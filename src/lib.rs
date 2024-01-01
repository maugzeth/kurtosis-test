//! Testing utility for managing local Kurtosis Ethereum network.

// TODO: Implement `Drop` trait to automatically destroy/clean all enclaves of engine, calls self.teardown().
// TODO: refresh/restart?
// TODO: Mine utility for mining blocks: "mine(x)", "mine_every(sec)"

use ethers::{prelude::*, types::transaction::eip2718::TypedTransaction};
use kurtosis_sdk::engine_api::engine_service_client::EngineServiceClient;
use serde_json::json;

pub mod constants;
pub mod eoa;
pub mod types;
mod kurtosis;
mod errors;
mod utils;

use crate::eoa::TestEOA;
use crate::errors::KurtosisNetworkError;
use crate::types::EthRpcClient;

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
        kurtosis::is_cli_installed()?;
        println!("Kurtosis installed.");

        // start kurtosis engine (in docker), if no engine context is found
        if !kurtosis::is_engine_running()? {
            println!("Starting kurtosis engine locally...");
            kurtosis::start_engine(
                network_params_file_name.unwrap_or(constants::DEFAULT_NETWORK_PARAMS_FILE_NAME),
            )?;
        }
        println!("Kurtosis engine running locally.");

        // connect to local kurtosis engine
        let mut engine = EngineServiceClient::connect(constants::DEFAULT_KURTOSIS_ENGINE_ENDPOINT)
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
            kurtosis::start_engine(
                network_params_file_name.unwrap_or(constants::DEFAULT_NETWORK_PARAMS_FILE_NAME),
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
        let services = kurtosis::get_running_services(enclave_id.as_str())?;
        // DEBUG: 
        // println!("Indexed services within enclave: ");
        // services.iter().for_each(|s| {
        //     println!("+ {}", s.name);
        //     s.ports.iter().for_each(|p| {
        //         println!("    + {}@{}, ", p.name, p.url);
        //     });
        // });

        Ok(Self {
            engine,
            enclave_id,
            services,
        })
    }

    /// Default chain network ID for kurtosis test networks.
    pub fn chain_id(&self) -> u64 {
        constants::DEFAULT_LOCAL_CHAIN_ID
    }

    /// Destroy enclave containing eithereum test network, engine continues running.
    pub fn destroy(&self) -> Result<(), KurtosisNetworkError> {
        println!("Destroying enclave: {}", self.enclave_id);
        kurtosis::delete_enclave(self.enclave_id.as_str())
    }

    /// Send transaction to network node, must be execution layer (EL).
    pub async fn send_transaction(
        &self,
        el_rpc_port: &EnclaveServicePortInfo,
        sender: &mut TestEOA,
        tx: &TypedTransaction,
    ) -> Result<TxHash, KurtosisNetworkError> {
        // define RPC client for execution layer node, with sender as signer
        let rpc_client = self.rpc_client_for(el_rpc_port, sender).await?;

        // fetch current block number to use as block id for transaction
        let block_num = rpc_client.get_block_number().await.unwrap();
        println!("BLOCK NUM: {:?}", block_num);

        // send transaction to execution layer node
        let sent_tx = rpc_client
            .send_transaction(tx.clone(), Some(BlockId::from(block_num)))
            .await
            .unwrap();
        println!("SENT TX: {:?}", sent_tx);

        // increment sender nonce, on successful transaction send 
        sender.increment_nonce();

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
            .private_key()
            .parse::<LocalWallet>()
            .map_err(|e| KurtosisNetworkError::FailedToCreateRpcClient(e.to_string()))?;
        let client = SignerMiddleware::new_with_provider_chain(provider.clone(), wallet)
            .await
            .map_err(|e| KurtosisNetworkError::FailedToCreateRpcClient(e.to_string()))?;

        Ok(client)
    }
}
