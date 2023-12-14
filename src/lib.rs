//! Testing utility for managing local Kurtosis Ethereum network.

use std::process::Command;

use ethers::{prelude::*, types::transaction::eip2718::TypedTransaction, utils::hex::ToHex};
use kurtosis_sdk::engine_api::engine_service_client::EngineServiceClient;
use regex::Regex;
use serde_json::json;

type EthRpcClient = SignerMiddleware<Provider<ethers::providers::Http>, LocalWallet>;

#[derive(thiserror::Error, Debug)]
pub enum KurtosisNetworkError {
    #[error("failed to connect to kurtosis engine")]
    EngineConnect,
    #[error("kurtosis cli is not installed locally")]
    CliNotInstalled,
    #[error("failed to start kurtosis engine locally, check if docker installed")]
    FailedToStartKurtosisEngine,
    #[error("failed to check kurtosis engine status")]
    FailedToCheckEngineStatus,
    #[error("failed to add enclave: {0}")]
    FailedToAddEnclave(String),
    #[error("failed to destroy enclave: {0}")]
    FailedToRemoveEnclave(String),
    #[error("enclave id is not unique, try a different one")]
    NonUniqueEnclaveName,
    #[error("enclave doesn't exist for network")]
    EnclaveDoesNotExist,
    #[error("failed to fetch and parse enclave services")]
    FailedToGetEnclaveServices,
    #[error("failed to destroy enclave")]
    FailedToDestroyEnclave,
    #[error("failed to instantiate RPC client: {0}")]
    FailedToCreateRpcClient(String),
    #[error("http call failed: {0}")]
    HttpCallError(String),
}

pub struct TestEOA {
    nonce: u64,
    address: Address,
    private_key: String,
}

impl TestEOA {
    pub fn new() -> TestEOA {
        let wallet = LocalWallet::new(&mut rand::thread_rng());
        // TODO: Prefund the account with some ETH, by sending ETH to it from another account.
        // This will cause issues with transaction that are not the users being on the network.
        // Is there another way to prefund the account?
        TestEOA {
            nonce: 0,
            address: wallet.address(),
            private_key: wallet.signer().to_bytes().encode_hex::<String>(),
        }
    }

    pub fn address(&self) -> Address {
        self.address
    }

    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    pub fn increment_nonce(&mut self) {
        self.nonce += 1;
    }
}

/// Representation of a HTTP service call.
#[derive(Debug)]
pub struct EnclaveServiceEthCall {
    /// Service port to send request to
    pub service_port: EnclaveServicePortInfo,
    /// Request method e.g. GET, POST, etc
    pub http_method: reqwest::Method,
    /// Ethereum method e.g. eth_call, eth_senfTransaction
    pub eth_method: &'static str,
    /// Body/payload for HTTP service call
    pub payload: serde_json::Value,
}

/// Enclave service port info structure.
#[derive(Debug, Clone)]
pub struct EnclaveServicePortInfo {
    /// Port name e.g. "http", "metrics", "rpc", etc
    pub name: String,
    /// Port protocol description e.g. "8080/tcp"
    pub protocol: String,
    /// URL to connect to service e.g. "127.0.0.1:56766".
    pub url: String,
}

impl EnclaveServicePortInfo {
    /// Check if port is JSON-RPC port.
    pub fn is_rpc_port(&self) -> bool {
        self.name.eq("rpc")
    }

    /// Check if port is a engine RPC.
    pub fn is_engine_rpc_port(&self) -> bool {
        self.name.eq("engine-rpc")
    }
}

/// Enclave service structure.
#[derive(Debug)]
pub struct EnclaveService {
    /// Unique identifier for service
    pub uuid: String,
    /// Human readable name of service
    pub name: String,
    /// Status of the service e.g. "RUNNING"
    pub status: String,
    /// List of service ports
    pub ports: Vec<EnclaveServicePortInfo>,
}

impl EnclaveService {
    /// Check if service is execution layer service, name is prefixed with "el-" and has RPC service port.
    pub fn is_exec_layer(&self) -> bool {
        self.name.contains("el-") && self.ports.iter().find(|port| port.is_rpc_port()).is_some()
    }
}

// TODO: Implement `Drop` trait to automatically destroy/clean all enclaves of engine, calls self.teardown().
// TODO: refresh/restart?
// TODO: Mine utility for mining blocks: "mine(x)", "mine_every(sec)"

/// Kurtosis Ethereum test network.
pub struct KurtosisTestNetwork {
    /// Kurtosis engine client
    pub engine: EngineServiceClient<tonic::transport::channel::Channel>,
    /// Active enclaves active for Kurtosis engine
    pub enclave_id: String,
    /// Services running on enclaves
    pub services: Vec<EnclaveService>,
}

impl KurtosisTestNetwork {
    /// Setup local Kurtosis network for testing.
    pub async fn setup(
        network_params_file_name: Option<&str>,
    ) -> Result<Self, KurtosisNetworkError> {
        is_kurtosis_cli_installed()?;
        println!("Kurtosis installed.");

        // start kurtosis engine (in docker), if no engine context is found
        if !is_kurtosis_engine_running()? {
            println!("Starting kurtosis engine locally...");
            start_kurtosis_engine(
                network_params_file_name.unwrap_or("default_network_params.json"),
            )?;
        }
        println!("Kurtosis engine running locally.");

        // connect to local kurtosis engine
        let mut engine = EngineServiceClient::connect("https://[::1]:9710")
            .await
            .map_err(|_| KurtosisNetworkError::EngineConnect)
            .unwrap();
        println!("Connected to engine.");

        // get an existing enclave id, can only be empty or a single ethereum-package enclave is deployed.
        // empty state is only acheived when we have a running engine but no etherereum-package enclave deployed.
        // if we have a single enclave, ethereum-package enclave is deployed within running engine.
        let existing_enclaves = engine.get_enclaves(()).await.unwrap().into_inner();
        let mut enclave_id: String = existing_enclaves
            .enclave_info
            .keys()
            .map(|id| id.to_string())
            .collect();

        // if no enclave found, create ethereum-package
        if enclave_id.is_empty() {
            println!("No existing enclave found on startup, creating ethereum-package enclave.");
            start_kurtosis_engine(
                network_params_file_name.unwrap_or("default_network_params.json"),
            )?;

            // Fetch newly created ethereum-package enclave uuid
            let existing_enclaves = engine.get_enclaves(()).await.unwrap().into_inner();
            enclave_id = existing_enclaves
                .enclave_info
                .keys()
                .map(|id| id.to_string())
                .collect();
        } else {
            println!("Existing enclaves on startup: {:?}", enclave_id);
        }

        // get and parse all services of enclave
        let services = get_kurtosis_running_services(enclave_id.as_str())?;
        // DEBUG: services.iter().for_each(|s| println!("{:?}", s));

        Ok(Self {
            engine,
            enclave_id,
            services,
        })
    }

    /// Default chain network ID for kurtosis test networks.
    pub fn chain_id(&self) -> u64 {
        3151908
    }

    /// Destroy enclave containing eithereum test network, engine continues running.
    pub fn destroy(&self) -> Result<(), KurtosisNetworkError> {
        delete_kurtosis_enclave(self.enclave_id.as_str())
    }

    /// Send transaction to network node, must be execution layer (EL).
    pub async fn send_transaction(
        &self,
        el_rpc_port: &EnclaveServicePortInfo,
        sender: &TestEOA,
        tx: &TypedTransaction,
    ) -> Result<TxHash, KurtosisNetworkError> {
        let rpc_client = self.rpc_client_for(el_rpc_port, sender).await?;

        let block_num = rpc_client.get_block_number().await.unwrap();
        println!("BLOCK NUM: {:?}", block_num);

        let sent_tx = rpc_client
            .send_transaction(tx.clone(), Some(BlockId::from(block_num)))
            .await
            .unwrap();
        println!("SENT TX: {:?}", sent_tx);

        Ok(sent_tx.tx_hash())
    }

    /// Generic utility for directly calling/interacting with a enclave service endpoint.
    pub async fn call(
        &self,
        call: &EnclaveServiceEthCall,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let client = reqwest::Client::new();

        let request: reqwest::RequestBuilder = match call.http_method {
            reqwest::Method::GET => client.get(format!("http//{}", call.service_port.url)),
            reqwest::Method::POST => client.post(format!("http://{}", call.service_port.url)),
            reqwest::Method::PUT => client.put(format!("http://{}", call.service_port.url)),
            reqwest::Method::DELETE => client.delete(format!("http://{}", call.service_port.url)),
            _ => panic!("Unsupported service call method."),
        };

        let payload = json!({ "id": 1, "jsonrpc": "2.0", "method": call.eth_method, "params": [&call.payload]});
        println!("PAYLOAD: {:?}", payload);

        request.json(&payload).send().await
    }

    /// Instantiate and return RPC client for RPC service port with signer middleware.
    pub async fn rpc_client_for(
        &self,
        service_port: &EnclaveServicePortInfo,
        signer: &TestEOA,
    ) -> Result<EthRpcClient, KurtosisNetworkError> {
        if !service_port.is_rpc_port() {
            return Err(KurtosisNetworkError::FailedToCreateRpcClient(
                "Port provided is not an RPC port.".to_string(),
            ));
        }

        // Create client with signer middleware
        let rpc_url = format!("http://{}", service_port.url);
        let provider = Provider::<Http>::try_from(rpc_url)
            .map_err(|e| KurtosisNetworkError::FailedToCreateRpcClient(e.to_string()))?;
        let wallet = signer
            .private_key
            .parse::<LocalWallet>()
            .map_err(|e| KurtosisNetworkError::FailedToCreateRpcClient(e.to_string()))?;
        let client = SignerMiddleware::new_with_provider_chain(provider.clone(), wallet)
            .await
            .map_err(|e| KurtosisNetworkError::FailedToCreateRpcClient(e.to_string()))?;

        Ok(client)
    }
}

// fn format_tx

/// Start Kurtosis engine locally in docker using ethereum-package.
///
/// Command:
/// `kurtosis run github.com/kurtosis-tech/ethereum-package --args-file {network_params_file_name}`
///
/// Launch `ethereum-package` enclave, this also spins up engine if not already done so.
fn start_kurtosis_engine(network_params_file_name: &str) -> eyre::Result<(), KurtosisNetworkError> {
    let mut net_param_conf_path = "tests/configs/".to_string();
    net_param_conf_path.push_str(network_params_file_name);

    let cmd_result = Command::new("kurtosis")
        .arg("run")
        .arg("github.com/kurtosis-tech/ethereum-package")
        .arg("--args-file")
        .arg(net_param_conf_path)
        .output();
    match cmd_result {
        Ok(out) => {
            if !out.status.success() {
                return Err(KurtosisNetworkError::FailedToStartKurtosisEngine);
            }
            Ok(())
        }
        Err(_) => Err(KurtosisNetworkError::FailedToStartKurtosisEngine),
    }
}

/// Check if Kurtosis CLI is installed locally.
///
/// Command:
/// `kurtosis version`
///
/// If getting version fails, we know Kurtosis is not installed, else it is.
fn is_kurtosis_cli_installed() -> eyre::Result<(), KurtosisNetworkError> {
    let cmd_result = Command::new("kurtosis").arg("version").output();
    match cmd_result {
        Ok(out) => {
            if !out.status.success() {
                return Err(KurtosisNetworkError::FailedToCheckEngineStatus);
            }
            Ok(())
        }
        Err(_) => Err(KurtosisNetworkError::CliNotInstalled),
    }
}

/// Check if Kurtosis engine is running locally in docker.
///
/// Command:
/// `kurtosis engine status`
///
/// Check if kurtosis engine is running by checking for presence of string:
///
/// `"Kurtosis engine is running with the following info"`
///
/// If present in standard output of command, return `true` if so, else `false`.
fn is_kurtosis_engine_running() -> eyre::Result<bool, KurtosisNetworkError> {
    let cmd_out = Command::new("kurtosis")
        .arg("engine")
        .arg("status")
        .output();
    match cmd_out {
        Ok(out) => {
            if !out.status.success() {
                return Err(KurtosisNetworkError::FailedToCheckEngineStatus);
            }
            let command_stdout = String::from_utf8_lossy(&out.stdout);
            Ok(command_stdout.contains("Kurtosis engine is running with the following info"))
        }
        Err(_) => Err(KurtosisNetworkError::FailedToCheckEngineStatus),
    }
}

/// Fetch all active/running services in enclave.
///
/// Command:
/// `kurtosis enclave inspect {enclave_uuid}`
///
/// Returns a list of enclave services parsed from standard output of enclave inspect.
fn get_kurtosis_running_services(
    enclave_uuid: &str,
) -> Result<Vec<EnclaveService>, KurtosisNetworkError> {
    let cmd_out = Command::new("kurtosis")
        .arg("enclave")
        .arg("inspect")
        .arg(enclave_uuid)
        .output();
    match cmd_out {
        Ok(out) => {
            if !out.status.success() {
                return Err(KurtosisNetworkError::FailedToGetEnclaveServices);
            }
            let command_stdout = String::from_utf8_lossy(&out.stdout);
            let enclave_services = parse_services_from_enclave_inspect(&command_stdout.to_string());
            Ok(enclave_services)
        }
        Err(_) => Err(KurtosisNetworkError::FailedToGetEnclaveServices),
    }
}

/// Deletes an enclave.
///
/// Command:
/// `kurtosis enclave rm {enclave_uuid} --force`
///
/// We force removal using `--force` to prevent having to stop enclave, then removing.
/// Instead this does it all within a single command.
fn delete_kurtosis_enclave(enclave_uuid: &str) -> Result<(), KurtosisNetworkError> {
    let cmd_out = Command::new("kurtosis")
        .arg("enclave")
        .arg("rm")
        .arg(enclave_uuid)
        .arg("--force")
        .output();
    match cmd_out {
        Ok(out) => {
            if !out.status.success() {
                return Err(KurtosisNetworkError::FailedToDestroyEnclave);
            }
            Ok(())
        }
        Err(_) => Err(KurtosisNetworkError::FailedToDestroyEnclave),
    }
}

/// Parses raw services output from kurtosis enclave inspect command output to [`EnclaveService`] type.
///
/// Example of all service line edge cases handled:
///
/// ```text
/// 0: ========================================== User Services ==========================================
/// 1: 7d28bc07285f   beacon-metrics-gazer                             http: 8080/tcp -> http://127.0.0.1:56766      RUNNING
/// 2: 93e319e73408   cl-1-lighthouse-reth                             http: 4000/tcp -> http://127.0.0.1:56741      RUNNING
/// 3:                                                                 metrics: 5054/tcp -> http://127.0.0.1:56742
/// 4: cd490f70070c   blob-spammer                                     <none>                                        RUNNING
/// ````
///
/// Parse normal service lines (lines 1, 2), some services have multiple ports (line 3) or no ports (line 4).
fn parse_services_from_enclave_inspect(raw_output: &String) -> Vec<EnclaveService> {
    let none_port_service_line_re =
        Regex::new(r"^([a-f0-9]{12})\s+(\S+)\s+(<none>)\s+(\S+)").unwrap();
    let continue_service_line_re = Regex::new(r"^\s+(\S+)(:)\s+(\S+)\s+(\S+)\s+(\S+)\s+").unwrap();
    let new_service_line_re =
        Regex::new(r"^([a-f0-9]{12})\s+(\S+)\s+(\S+)(:)\s(\d+\S+)\s(->)\s(\S+)(\s+)(\S+)$")
            .unwrap();

    let mut services: Vec<EnclaveService> = Vec::new();
    raw_output.split("\n").for_each(|line| {
        // DEBUG: println!("{:?}", line);

        // if we match a new service line, return new enclave service entry
        if let Some(caps) = new_service_line_re.captures(line) {
            let port_info = EnclaveServicePortInfo {
                name: caps.get(3).unwrap().as_str().to_string(),
                protocol: caps.get(5).unwrap().as_str().to_string(),
                url: caps.get(7).unwrap().as_str().to_string(),
            };
            services.push(EnclaveService {
                uuid: caps.get(1).unwrap().as_str().to_string(),
                name: caps.get(2).unwrap().as_str().to_string(),
                status: caps.get(9).unwrap().as_str().to_string(),
                ports: vec![port_info],
            });
            return;
        }

        // if we match a none port service, return new enclave service entry with no ports
        if let Some(caps) = none_port_service_line_re.captures(line) {
            services.push(EnclaveService {
                uuid: caps.get(1).unwrap().as_str().to_string(),
                name: caps.get(2).unwrap().as_str().to_string(),
                status: caps.get(4).unwrap().as_str().to_string(),
                ports: vec![],
            });
            return;
        }

        // if we match a continued service port line, update last service by appending to ports
        if let Some(caps) = continue_service_line_re.captures(line) {
            let mut last_service = services.pop().unwrap();
            let mut updated_service_ports = last_service.ports;
            updated_service_ports.push(EnclaveServicePortInfo {
                name: caps.get(1).unwrap().as_str().to_string(),
                protocol: caps.get(3).unwrap().as_str().to_string(),
                url: caps.get(5).unwrap().as_str().to_string(),
            });
            last_service.ports = updated_service_ports;
            services.push(last_service);
            return;
        }
    });

    services
}
