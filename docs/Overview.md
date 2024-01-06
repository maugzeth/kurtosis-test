# Documentation

## Usage

### Create Test Network

Import the kurtosis test network object into scope:

```rust
use kurtosis_test::KurtosisTestNetwork;
```

#### Default Test Network

Creates a custom test network, detailed [here](./DefaultNetwork.md).

```rust
let network = KurtosisTestNetwork::setup(None).await.unwrap();
```

#### Custom Test Network

within the root of your project directory define custom network parameter configuration file in:

`tests/configs/netparams/{custom_net_params.json}`

Then pass the name as a parameter when instantiating your test network:

```rust
let network_params = Some("custom_net_params.json");
let network = KurtosisTestNetwork::setup(network_params).await.unwrap();
```

### Create Test Account (EOA)

Import into scope:

```rust
use kurtosis_test::TestEOA;
```

#### Unfunded Account

Create a unfunded Externally Owned Account (EOA).

```rust
let mut unfunded_eoa = TestEOA::new(&network, None).await.unwrap();
```

#### Funded Account

Create Externally Owned Account (EOA) funded with 100 ETH.

```rust
use ethers::utils::parse_ether;

let fund_amount = parse_ether("100").unwrap();
let mut funded_eoa = TEstEOA::new(&network, Some(fund_amount)).await.unwrap();
```

#### Accessing EOA Properties

Common properties exposed by test EOA object:

```rust
let mut eoa = TestEOA::new(&network, None).await.unwrap();
eoa.address();      // aka public key
eoa.nonece();       // aka transaction count
eoa.private_key();
```

### Inspect Network Services

#### Manually Inspect Network Services

We can get a list of all running services on our test network by calling:

```rust
network.services();
```

#### Manually Inspect Network Service Ports

You can manually inspect network services and their respective ports, like so:

```rust
let el_service = network.services().iter().find(|service| service.is_exec_layer())?;
let rpc_port = el_service.ports.iter().find(|port| port.is_rpc_port())?
```

#### Get Execution Layer RPC Port

A common operation is to fetch the RPC port of an execution layer (EL) service within a network.

To do so you can use a utility function:

```rust
use kurtosis_test::utils;
let rpc_port = utils::get_el_rpc_port(&network).unwrap();
```

### Sending Transactions

#### Using RPC Client

Instantiate an `ethers-rs` JSON-RPC client with passed in test EOA used as transaction signer.

Transaction will be sent to node for which `el_rpc_port` (service port) belongs to.

```rust
let rpc_client = network.rpc_client_for(&el_rpc_port, &test_eoa).await?;
let eoa_tx_count = rpc_client.get_transaction_count(&test_eoa.address(), None).await?;
```

#### Using Network Utility

Send a `TypedTransaction` via network utility to any node on network.

```rust
use ethers::types::{transaction::eip2718::TypedTransaction, TransactionRequest};

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

network.send_transaction(&rpc_port, &mut sender, &tx).await?;
```

### Block Utilities

#### Waiting for New Block

Wait for new block to be mined before continuing.

```rust
network.wait_for_new_block().await.unwrap();
```

#### Wait for X Amount of Blocks

Waits for X number of blocks to be mined before continuing.

```rust
network.wait_for_x_block(3).await.unwrap();
```

### Assertions

TODO