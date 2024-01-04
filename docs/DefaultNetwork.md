# Default Network Parameters

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
  // prevent spin-up of addtional services
  "launch_additional_services": false,
  // empty for most minimal local deployment
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
