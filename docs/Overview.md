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

Define custom network parameter configuration file in `tests/configs/netparams/{custom_net_params.json}`.

```rust
let network_params = Some("custom_net_params.json");
let network = KurtosisTestNetwork::setup(network_params).await.unwrap();
```

### Create Test Account (EOA)

```rust
use kurtosis_test::eoa::TestEOA;
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

### Fetch Network Nodes / Participants

#### Manually Inspect Network Services

```rust
network.services.iter().find(|service| ...);
```

#### Manually Inspect Network Service Ports

You can manually inspect network services and their respective ports like so:

```rust
let el_service = network.services.iter().find(|service| service.is_exec_layer())?;
let rpc_port = el_service.ports.iter().find(|port| ...)?;
```

#### Get Execution Layer RPC Port

It's common to want to fetch RPC port of execution layer (EL) service within network.

To do so you can use a utility function:

```rust
use kurtosis_test::utils;
let rpc_port = utils::get_el_rpc_port(&network).unwrap();
```

### Sending Transactions

#### Using RPC Client

Instantiate an `ethers-rs` JSON-RPC client with passed in test EOA used as transaction signer.

Transaction will be sent to node belonging for which `el_rpc_port` belongs to.

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

### Assertions

TODO