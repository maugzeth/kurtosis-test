//! Testing utility for managing local Kurtosis Ethereum network.

use ethers::{prelude::*, types::transaction::eip2718::TypedTransaction};
use kurtosis_sdk::engine_api::engine_service_client::EngineServiceClient;

pub mod constants;
pub mod eoa;
mod errors;
mod kurtosis;
pub mod types;
pub mod utils;

use crate::eoa::TestEOA;
use crate::errors::KurtosisNetworkError;
use crate::kurtosis::{EnclaveService, EnclaveServicePort};
use crate::types::EthRpcClient;

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
        // check kurtosis cli is installed
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

        // fetch existing enclaves for engine
        let existing_enclaves = engine.get_enclaves(()).await.unwrap().into_inner();
        let mut enclave_id: String = existing_enclaves
            .enclave_info
            .keys()
            .map(|id| id.to_string())
            .collect();

        // if no enclave is found, create ethereum-package enclave
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
            println!("Existing enclave found on startup: {:?}", enclave_id);
        }

        // fetch and parse all services of enclave
        let services = kurtosis::get_running_services(enclave_id.as_str())?;
        utils::pprint_services(&services);

        Ok(Self {
            engine,
            enclave_id,
            services,
        })
    }

    /// Default chain network ID for kurtosis test network.
    pub fn chain_id(&self) -> u64 {
        constants::DEFAULT_LOCAL_CHAIN_ID
    }

    /// Destroy enclave containing eithereum test network, engine continues running.
    pub fn destroy(&self) -> Result<(), KurtosisNetworkError> {
        println!("Destroying enclave: {}", self.enclave_id);
        kurtosis::delete_enclave(self.enclave_id.as_str())
    }

    /// Send transaction to network node (via given execution layer RPC port).
    pub async fn send_transaction(
        &self,
        el_rpc_port: &EnclaveServicePort,
        sender: &mut TestEOA,
        tx: &TypedTransaction,
    ) -> Result<TxHash, KurtosisNetworkError> {
        // define RPC client for execution layer node, with sender as signer
        let rpc_client = self.rpc_client_for(&el_rpc_port, &sender).await?;

        // fetch current block number to use as block id for transaction
        let block_num = rpc_client.get_block_number().await.unwrap();
        println!("BLOCK NUM: {:?}", block_num);

        // send transaction to execution layer node
        let sent_tx = rpc_client
            .send_transaction(tx.clone(), Some(BlockId::from(block_num)))
            .await
            .map_err(|e| KurtosisNetworkError::FailedToSendTransaction(e.to_string()))
            .unwrap();
        println!("SENT TX: {:?}", sent_tx);

        // increment sender nonce, on successful transaction send
        sender.increment_nonce();

        Ok(sent_tx.tx_hash())
    }

    /// Instantiate and return RPC client for RPC service port with signer middleware.
    pub async fn rpc_client_for(
        &self,
        service_port: &EnclaveServicePort,
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

impl Drop for KurtosisTestNetwork {
    fn drop(&mut self) {
        println!("Shutting down kurtosis test network.");
        self.destroy().unwrap();
    }
}
