//!

/// Default local Ethereum chain identifier.
pub const DEFAULT_LOCAL_CHAIN_ID: u64 = 3151908;
/// File name for default network parameter.
pub const DEFAULT_NETWORK_PARAMS_FILE_NAME: &'static str = "default_network_params.json";
/// Default endpoint used by Kurtosis engine.
pub const DEFAULT_KURTOSIS_ENGINE_ENDPOINT: &'static str = "https://[::1]:9710";
/// Public key for funding account.
pub const PREFUNDING_ACCOUNT_PUB_KEY: &'static str = "0xf93Ee4Cf8c6c40b329b0c0626F28333c132CF241";
/// Private key for funding account.
pub const PREFUNDING_ACCOUNT_PRIV_KEY: &'static str =
    "ab63b23eb7941c1251757e24b3d2350d2bc05c3c388d06f8fe6feafefb1e8c70";
/// Default gas price for transfer transactions.
pub const ETH_TRANSFER_GAS_LIMIT: u64 = 21000;
