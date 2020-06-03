use crate::errors::{Error, ErrorKind, Result};
use num_traits::identities::Zero;
use std::str::FromStr;

#[allow(non_upper_case_globals)]
pub const TXVersion: i32 = 1;

pub fn now_as_nanos() -> i64 {
    let t = std::time::SystemTime::now();
    let since_the_epoch = t
        .duration_since(std::time::UNIX_EPOCH)
        .expect("now as nanos");
    since_the_epoch.as_nanos() as i64
}

pub fn now_as_secs() -> i64 {
    let t = std::time::SystemTime::now();
    let since_the_epoch = t
        .duration_since(std::time::UNIX_EPOCH)
        .expect("now as nanos");
    since_the_epoch.as_secs() as i64
}

pub fn str_as_i64(s: &str) -> Result<i64> {
    let i = s
        .parse::<i64>()
        .map_err(|_| Error::from(ErrorKind::ParseError))?;
    if i < 0 {
        return Err(Error::from(ErrorKind::InvalidArguments));
    }
    Ok(i)
}

pub fn str_as_bigint(s: &str) -> Result<num_bigint::BigInt> {
    if s == "0" || s.len() == 0 {
        return Ok(num_bigint::BigInt::zero());
    }
    num_bigint::BigInt::from_str(s).map_err(|_| Error::from(ErrorKind::ParseError))
}

pub fn print_bytes_num(s: &Vec<u8>) {
    println!(
        "print_bytes_num: {:?}",
        num_bigint::BigInt::from_bytes_be(num_bigint::Sign::Plus, s).to_str_radix(10)
    );
}
