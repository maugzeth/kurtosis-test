//!

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