#![cfg_attr(all(feature = "mesalock_sgx",
                not(target_env = "sgx")), no_std)]
#![cfg_attr(all(target_env = "sgx", target_vendor = "mesalock"), feature(rustc_private))]

#[cfg(all(feature = "mesalock_sgx", not(target_env = "sgx")))]
#[macro_use]
extern crate sgx_tstd as std;

use std::prelude::v1::*;

extern crate sgx_types;
use sgx_types::*;

use std::path::PathBuf;

#[macro_use]
extern crate lazy_static;

use hex;
use mesatee_sdk::{Mesatee, MesateeEnclaveInfo};
use std::net::SocketAddr;

lazy_static! {
    static ref USER_ID: String = String::from("user1");
    static ref USER_TOKEN: String = String::from("token1");
    static ref FNS_ADDR: SocketAddr = "localhost:30007".parse().unwrap();
    static ref PUBKEY_PATH: String = String::from("auditors/godzilla/godzilla.public.der");
    static ref SIG_PATH: String = String::from("auditors/godzilla/godzilla.sign.sha256");
    static ref ENCLAVE_PATH: String = String::from("enclave_info.toml");
}

#[no_mangle]
pub extern "C" fn ecall_run_tests() {
    let mut auditors: Vec<(&str, &str)> = Vec::new();
    auditors.push((&PUBKEY_PATH, &SIG_PATH));
    let enclave_info: MesateeEnclaveInfo =
        MesateeEnclaveInfo::load(auditors, &ENCLAVE_PATH).unwrap();
    let mesatee: Mesatee = Mesatee::new(&enclave_info, &USER_ID, &USER_TOKEN, *FNS_ADDR).unwrap();
    let msg: String = String::from("hello, from Occlum");
    let task = mesatee.create_task("echo").unwrap();
    let _res = task.invoke_with_payload(&msg);

    println!("done");
}
