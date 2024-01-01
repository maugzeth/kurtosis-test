//!

use ethers::middleware::SignerMiddleware;
use ethers::providers::{Http, Provider};
use ethers::signers::LocalWallet;

pub type EthRpcClient = SignerMiddleware<Provider<Http>, LocalWallet>;