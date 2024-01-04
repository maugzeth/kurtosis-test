//!

use ethers::types::{transaction::eip2718::TypedTransaction, TransactionRequest};
use ethers::utils::parse_ether;
use kurtosis_test::{eoa::TestEOA, KurtosisTestNetwork, utils};

async fn setup_network() -> KurtosisTestNetwork {
    KurtosisTestNetwork::setup(None).await.unwrap()
}

#[tokio::test]
async fn test_something() {
    // 1. Setup ethereum test network.
    let network = setup_network().await;

    let rpc_port = utils::get_el_rpc_port(&network).unwrap();

    // let funding_eth = parse_ether("100").unwrap();
    let mut sender = TestEOA::new(&network, None).await.unwrap();
    // let tx = TypedTransaction::Legacy(
    //     TransactionRequest {
    //         from: Some(sender.address()),
    //         to: Some(sender.address().into()),
    //         gas: Some(21000.into()), // typical gas limit for a simple transfer
    //         gas_price: Some(20_000_000_000u64.into()), // gas price in wei, adjust as needed
    //         value: Some(1_000_000_000_000_000u64.into()), // value in wei, adjust as needed
    //         data: None,              // no data for a simple transfer
    //         nonce: Some(sender.nonce().into()), // nonce, adjust as needed
    //         chain_id: Some(network.chain_id().into()), // chain id for mainnet, adjust as needed
    //     }
    // );
    // network.send_transaction(rpc_port, &mut sender, &tx).await.unwrap();
}
