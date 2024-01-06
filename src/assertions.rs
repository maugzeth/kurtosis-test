//!

use ethers::prelude::*;

use crate::{KurtosisTestNetwork, TestEOA, utils};

// TODO: assert_eoa_balance
pub async fn assert_eoa_balance(network: &KurtosisTestNetwork, eoa: &TestEOA, expected_balance: U256) {
    let el_rpc_port = utils::get_el_rpc_port(&network).unwrap();
    let rpc_client = network.rpc_client_for(&el_rpc_port, &eoa).await.unwrap();
    let balance = rpc_client.get_balance(eoa.address(), None).await.unwrap().as_u64();
    assert_eq!(U256::from(balance), expected_balance);
    
}

// TODO: assert_eoa_nonce

// TODO: assert_block_number
// TODO: assert_block_tx_count
// TODO: assert_block_difficulty
// TODO: assert_block_gas_used

// TODO: assert_tx_status
// TODO: assert_tx_gas_used