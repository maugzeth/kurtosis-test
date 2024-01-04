# Kurtosis Test

Testing utility for setting up configurable Ethereum test network(s) programmatically in Rust using Kurtosis.

## Dependency

Make sure you have the following installed.

1. `Kurtosis CLI` - required for interfacing with kurtosis locally.
2. `Docker` - required to run containerizwd kurtosis engine and enclaves.

## Getting Started

Steps to getting started using Kurtosis in your Rust test suites.

### 1) Define Network Parameters/Configuration

Define your network configuration, an example of how a simple network parameter file would look like:

```json
{
  "participants": [
    {
      "el_client_type": "reth",
      "el_client_image": "ghcr.io/paradigmxyz/reth",
      "cl_client_type": "lighthouse",
      "cl_client_image": "sigp/lighthouse:latest",
      "count": 1
    },
    {
      "el_client_type": "reth",
      "el_client_image": "ghcr.io/paradigmxyz/reth",
      "cl_client_type": "teku",
      "cl_client_image": "consensys/teku:latest",
      "count": 1
    }
  ],
  "launch_additional_services": false,
  "additional_services": []
}
```

This is a simple network configuration which contains two node participants.

- **Participant 1** is a node running `Reth` (execution layer) and `Lighthouse` (consensus layer).
- **Participant 2** is a node running `Reth` (execution layer) and `Teku` (conensus layer).

Global flags like `launch_additional_services` can be passed to further configure the network.

In our case we don't need any but using this flag we could launch additional services like:

- A Grafana + Prometheus instance
- A transaction spammer called `tx-fuzz`
- Flashbot's `mev-boost` implementation of PBS (to test/simulate MEV workflows)

For more options check out `ethereum-package` documentation, [here](https://github.com/kurtosis-tech/ethereum-package/#configuration).

### 2) Setup Your Test

Here is a basic example of integration test for a transaction indexing program structure utilising `kurtosis-test` to launch ethereum test network:

```rust
use kurtosis_test::{KurtosisTestNetwork, eoa::TestEOA};
use ethers::types::{transaction::eip2718::TypedTransaction, TransactionRequest};
use ethers::utils::parse_ether;


/// Setup Ethereum test network using `network_params.json`.
///
/// Network params file will be searched for within:
///    `tests/configs/netparams`
/// Directory relative to project root.
async fn setup_network() -> KurtosisTestNetwork {
    KurtosisTestNetwork::setup("network_params.json").await.unwrap()
}

#[tokio::test]
async fn test_something() {
    // 1. Setup ethereum test network.
    let network = setup_network().await;

    // 2. Fetch required info from ethereum test network.
    // Ex: Find EL node service and port it exposes for JSON-RPC endpoint.
    let rpc_port = network.get_el_rpc_port().unwrap();

    // 3. Setup your application which is dependant on network info.
    // Ex: Setup a mock database and indexer workflow (application specific).
    let database = MyDatabase::new();
    let indexer = MyIndexer::new(&database, &rpc_port.url);
    
    // 4: interact with network e.g. define EOA and send transactions.
    // Ex: sending test transactions to test network.
    let eth_amount = parse_ether("10").unwrap();
    let sender = TestEOA::new(&network, eth_amount).await.unwrap();
    let tx = TypedTransaction::Legacy(
        TransactionRequest {
            from: Some(sender.address()),
            to: Some(sender.address().into()),
            gas: Some(21000.into()),
            gas_price: Some(20_000_000_000u64.into()),
            value: Some(1_000_000_000_000_000u64.into()),
            data: None,
            nonce: Some(sender.nonce().into()),
            chain_id: Some(network.chain_id().into()),
        }
    );
    network.send_transaction(rpc_port, &sender, &tx).await.unwrap();
    
    // 5: Assert your application state changed as expected.
    // Ex: database has indexed the two transactions sent to test network.
    let indexed_tx_count = database.count("transaction").await.unwrap();
    assert_eq!(indexed_tx_count, 2);
}
```

### 3) Run Your Test

You run your tests the same way you normally would, nothing special needs to be done.

The `kurtosis-test` create will handle spinning up the kurtosis engine, running the `ethereum-package` to setup our Ethereum test network and cleaning up.

### 4) Debug Test Network

To debug your test network you need two things:

#### Kurtosis CLI
  
Use command line interface to directly interact with kurtosis engine and respective enclaves.

#### Docker

View, manage and debug running docker images (engine, services).

## Contibutors

Authored by [@maugzeth](https://github.com/maugzeth) and maintained with 💚 by **Dedsol Team**
