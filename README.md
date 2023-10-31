# execution-payload-builder

Builds an `ExecutionPayload` from a JSON `Block` obtained from RPC.

## Usage
For example, using raw `cast rpc` output:
```
cast rpc eth_getBlockByHash '["0x58fee1cac2a8ef87a84e6a77cef27b4935e1cf8ae8320afdd5c176ef17b5d94a", true]' --raw > ~/devnet-43510-1.json
```

Then run:
```
cargo run -- --path ~/devnet-43510-1.json
```
