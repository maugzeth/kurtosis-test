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
  "launch_additional_services": false
}
```

This is a simple network configuration which contains two node participants.

- **Participant 1** is a node running `Reth` (execution layer) and `Lighthouse` (consensus layer).
- **Participant 2** is a node running `Reth` (execution layer) and `Teku` (conensus layer).

Global flags like `launch_additional_services` can be passed to further configure the network.

In this case this flag could launch additional services like:

- A Grafana + Prometheus instance
- A transaction spammer called `tx-fuzz`
- Flashbot's `mev-boost` implementation of PBS (to test/simulate MEV workflows)

For more options check out `ethereum-package` documentation, [here](https://github.com/kurtosis-tech/ethereum-package/#configuration).

### 2) Setup Your Test

Here is a basic example of integration test for a transaction indexing program structure utilising `kurtosis-test` to launch ethereum test network:

```rust
use kurtosis_test::KurtosisTestNetwork;

/// Setup Ethereum test network using `network_params.json`.
///
/// Network params file will be searched for within:
///    `tests/configs/netparams`
/// Directory relative to project root.
async fn setup_network() -> KurtosisTestNetwork {
    KurtosisTestNetwork::setup("network_params.json").await.unwrap()
}

/// Teardown/destroy kurtosis testing enclaves.
fn teardown_network(network: KurtosisTestNetwork) {
    network.destroy().unwrap();
}

#[tokio::test]
async fn test_something() {
    // 1. Setup ethereum test network.
    let network = setup_network().await;

    // 2. Fetch required info from ethereum test network.
    // Ex: Find EL node service and port it exposes for JSON-RPC endpoint.
    let el_service = network.services.iter()
      .find(|service| service.is_exec_layer()).unwrap();
    let rpc_service_port = el_service.ports.iter()
      .find(|port| port.is_rpc_port()).unwrap();

    // 3. Setup your application which is dependant on network info.
    // Ex: Setup a mock database and indexer workflow (application specific).
    let database = MyDatabase::new();
    let indexer = MyIndexer::new(&database, rpc_service_port.url);
    
    // 4: interact with network e.g. sending transactions.
    // Ex: sending two test transactions to test network.
      network.send_transaction(rpc_service_port, tx_object1);
      network.send_transaction(rpc_service_port, tx_object2); 
    
    // 5: Assert your application state changed as expected.
    // Ex: database has indexed the two transactions sent to test network.
    let indexed_tx_count = database.count("transaction").await.unwrap();
    assert_eq!(indexed_tx_count, 2);

    // 6. (optional) Teardown/destroy network, optionally destroy with_engine.
    teardown_network(network, with_engine=false)
}
```

### 3) Run Your Test

You run your tests the same way you normally would, nothing special needs to be done.

The `kurtosis-test` create will handle spinning up the kurtosis engine, running the `ethereum-package` to setup our Ethereum test network and cleaning up.

### 4) Debug Test Network

To debug your test network you need two things:

#### **Kurtosis CLI**

Use command line interface to directly interact with kurtosis engine and respective enclaves.

#### **Docker**

View, manage and debug running docker images (engine, services).
