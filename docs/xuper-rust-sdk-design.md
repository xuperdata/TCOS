## Motivation

This repo intents to provide a trusted SDK for XuperChain, and used in scenarios like confidential computing and data anchor.



## Design

Thanks to [xchain-rust-crypto](https://github.com/duanbing/xchain-rust-crypto), things become easy now. 

The main process of SDK includes:

* Load the private key from disk and keep it in memory
* Compose transaction
  * Select utxo from remote node by grpc
  * Check Compliance by grpc
  * Get RW set from PreExec by grpc
* Sign transaction locally
* Post transaction  to the remote node by grpc

This is also how the normal world SDKs do to send the transaction and call the smart contracts.  This this is not work for trusted SDK due to the unsafe loading and accessing of the private key. 

We consider protecting the private key accessing by TEE, like Intel SGX or ARM TrustZone. However grpc is still not migrated to trusted world.  So ocall will be used to do rpc. 



## Limit and TODO

* TODOs at [xchain-rust-crypto](https://github.com/duanbing/xchain-rust-crypto);
* Grpc migration



