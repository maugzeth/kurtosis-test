//!

use kurtosis_test::{KurtosisTestNetwork, TestEOA};
use ethers::types::{transaction::eip2718::TypedTransaction, TransactionRequest};

async fn setup_network() -> KurtosisTestNetwork {
    KurtosisTestNetwork::setup(None).await.unwrap()
}

fn teardown_network(network: KurtosisTestNetwork) {
    network.destroy().unwrap();
}

#[tokio::test]
async fn test_something() {
    // 1. Setup ethereum test network.
    let network = setup_network().await;

    // 2. Fetch required info from ethereum test network.
    // Ex: Find EL node service and port it exposes for JSON-RPC endpoint.
    // TODO: Add a way to filter for specific client type e.g. is_reth, is_geth, etc.
    let el_service = network
        .services
        .iter()
        .find(|service| service.is_exec_layer())
        .unwrap();
    let rpc_port = el_service
        .ports
        .iter()
        .find(|port| port.is_rpc_port())
        .unwrap();

    // 3. Setup your application which is dependant on network info.
    // Ex: Setup a mock database and indexer workflow (application specific).
    // let database = MyDatabase::new();
    // let indexer = MyIndexer::new(&database, rpc_service_port.url);

    // TODO: Optionally pass in rpc port or else default choose one for them.
    // 4: interact with network e.g. sending transactions.
    // Ex: sending two test transactions to test network.
    let mut sender = TestEOA::new();
    let tx = TypedTransaction::Legacy(TransactionRequest {
        from: Some(sender.address()),
        to: Some(sender.address().into()),
        gas: Some(21000.into()), // typical gas limit for a simple transfer
        gas_price: Some(20_000_000_000u64.into()), // gas price in wei, adjust as needed
        value: Some(1_000_000_000_000_000u64.into()), // value in wei, adjust as needed
        data: None,              // no data for a simple transfer
        nonce: Some(sender.nonce().into()), // nonce, adjust as needed
        chain_id: Some(network.chain_id().into()), // chain id for mainnet, adjust as needed
    });
    network.send_transaction(rpc_port, &mut sender, &tx).await.unwrap();

    // 5: Assert your application state changed as expected.
    // Ex: database has indexed the two transactions sent to test network.
    // let indexed_tx_count = database.count("transaction").await.unwrap();
    // assert_eq!(indexed_tx_count, 2);

    // 6. (optional) Teardown/destroy network, optionally destroy with_engine.
    // teardown_network(network)
}
