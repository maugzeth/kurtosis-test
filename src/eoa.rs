//! Representation of a externally owned account (EOA) used for testing.

use crate::{constants, errors::KurtosisNetworkError, network::KurtosisTestNetwork, utils};
use ethers::{
    prelude::*, types::transaction::eip2718::TypedTransaction, types::TransactionRequest,
    utils::hex::ToHex,
};

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
    pub async fn new(
        network: &KurtosisTestNetwork,
        eth_amount: Option<U256>,
    ) -> Result<TestEOA, KurtosisNetworkError> {
        // fetch execution layer node rpc port
        let el_rpc_port = utils::get_el_rpc_port(&network)?;

        // create a new test account
        let wallet = LocalWallet::new(&mut rand::thread_rng());
        let mut new_eoa = TestEOA {
            nonce: 0,
            address: wallet.address(),
            private_key: wallet.signer().to_bytes().encode_hex::<String>(),
        };

        // get existing transaction count for account if any on network, else result is 0
        let rpc_client = network.rpc_client_for(&el_rpc_port, &new_eoa).await?;
        let eoa_tx_count = rpc_client
            .get_transaction_count(wallet.address(), None)
            .await
            .map_err(|e| KurtosisNetworkError::FailedToCreateNewEOA(e.to_string()))
            .unwrap()
            .as_u64();

        // set nonce to transaction count for new transaction
        new_eoa.nonce = eoa_tx_count;

        // fund account with eth amount if specified
        if let Some(amount) = eth_amount {
            let mut funding_eoa = TestEOA::funding_eoa(&network).await.unwrap();

            let funding_tx = TypedTransaction::Legacy(TransactionRequest {
                from: Some(funding_eoa.address()),
                to: Some(new_eoa.address().into()),
                // typical gas limit for a simple transfer
                gas: Some(constants::ETH_TRANSFER_GAS_LIMIT.into()),
                gas_price: None,
                value: Some(amount),
                // no data for a simple transfer
                data: None,
                nonce: Some(funding_eoa.nonce().into()),
                chain_id: Some(network.chain_id().into()),
            });

            network
                .send_transaction(&mut funding_eoa, &funding_tx, Some(el_rpc_port))
                .await
                .map_err(|e| KurtosisNetworkError::FundingTestEoa(e.to_string()))
                .unwrap();
        };

        Ok(new_eoa)
    }

    /// Get EOA prefunded with 1,000,000,000 ETH, used to prefund other EOAs.
    async fn funding_eoa(network: &KurtosisTestNetwork) -> Result<TestEOA, KurtosisNetworkError> {
        let address = constants::PREFUNDING_ACCOUNT_PUB_KEY
            .parse::<Address>()
            .unwrap();
        let mut funding_eoa = TestEOA {
            nonce: 0,
            address: address,
            private_key: constants::PREFUNDING_ACCOUNT_PRIV_KEY.to_string(),
        };

        // fetch execution layer node rpc port
        let el_rpc_port = utils::get_el_rpc_port(&network)?;

        // get existing transaction count for account if any on network, else result is 0
        let rpc_client = network.rpc_client_for(&el_rpc_port, &funding_eoa).await?;
        let eoa_tx_count = rpc_client
            .get_transaction_count(address, None)
            .await
            .map_err(|e| KurtosisNetworkError::FailedToCreateNewEOA(e.to_string()))
            .unwrap()
            .as_u64();

        // set nonce to transaction count + 1 for new transaction
        funding_eoa.nonce = eoa_tx_count;

        Ok(funding_eoa)
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
    pub(crate) fn increment_nonce(&mut self) {
        self.nonce += 1;
    }

    /// Increment nonce of EOA by one, used when sending transactions.
    pub(crate) fn set_nonce(&mut self, new_nonce: u64) {
        self.nonce = new_nonce;
    }
}
