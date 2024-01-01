//!

use ethers::{prelude::*,  utils::hex::ToHex};

/// Representation of a test externally owned account (EOA).
pub struct TestEOA {
    /// Number of transactions
    nonce: u64,
    /// Generated address (public key)
    address: Address,
    /// Generated private key 
    private_key: String,
}

impl TestEOA {
    /// Create new test EOA with randomly generated private key.
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

    /// Get address of EOA.
    pub fn address(&self) -> Address {
        self.address
    }

    /// Get nonce of EOA, reflects number of transactions sent by EOA.
    pub fn nonce(&self) -> u64 {
        self.nonce
    }

    /// Get private key of EOA.
    pub fn private_key(&self) -> String {
        self.private_key.clone()
    }

    /// Increment nonce of EOA by one, used when sending transactions.
    pub fn increment_nonce(&mut self) {
        self.nonce += 1;
    }
}