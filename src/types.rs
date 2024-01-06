//! Custom module types

use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;

pub type EthRpcClientWithSigner = SignerMiddleware<Provider<Http>, LocalWallet>;
pub type EthRpcClient = Provider<Http>;
