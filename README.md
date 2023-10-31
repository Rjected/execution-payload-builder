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

It will spit out a `cast rpc engine_newPayloadV3` command:
```
cargo run -- --path ~/devnet-43511-2.json --jwt-secret blah --rpc-url foo
   Compiling execution-payload-builder v0.1.0 (/Users/dan/projects/execution-payload-builder)
    Finished dev [unoptimized + debuginfo] target(s) in 1.50s
     Running `target/debug/execution-payload-builder --path /Users/dan/devnet-43511-2.json --jwt-secret blah --rpc-url foo`
cast rpc --rpc-url foo --jwt-secret blah engine_newPayloadV3 --raw '[{"parentHash":"0x58fee1cac2a8ef87a84e6a77cef27b4935e1cf8ae8320afdd5c176ef17b5d94a","feeRecipient":"0xf97e180c050e5ab072211ad2c213eb5aee4df134","stateRoot":"0x110ab4c2a60046b0495821b7205e9779e5c82e272578dcf6da2f99e151d232be","receiptsRoot":"0xba987831fa678a1548ce8a6accab6f97cf8018f408e2ae2db73d119fdb4ac4e1","logsBloom":"0x002000000000000000000000800000020000000000001000080000800000000000000800
... response is huge
```
