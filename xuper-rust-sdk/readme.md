## xuper-rust-sdk

A Xuperchain SDK by rust, especially for TEE(Intel SGX/ARM TZ) application.


## Notices

* Serialize enum as number: https://serde.rs/enum-number.html
* #[serde(default)]  for header.from_node
* crate::wallet::* 
* InvokeResponse:  skip original reuquest


## Test
```
cargo test -- --test-threads 1
```
