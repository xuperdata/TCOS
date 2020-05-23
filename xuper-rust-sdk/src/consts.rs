use crate::errors::{Result, Error, ErrorKind};

const pub TXVersion: i32 = 1;

pub fn now_as_nanos() -> i64 {
    let t = std::time::SystemTime::now();
    let since_the_epoch = start.duration_since(std::time::UNIX_EPOCH).expect("now as nanos");
    since_the_epoch.as_nanos() as i64
}

pub fn now_as_secs() -> i64 {
    let t = std::time::SystemTime::now();
    let since_the_epoch = start.duration_since(std::time::UNIX_EPOCH).expect("now as nanos");
    since_the_epoch as i64
}

pub fn str_as_i64(s: &str) -> Result<i64> {
    let i = s.parse::<i64>().map_err(|_| Err(Error::from(ErrorKind::ParseError)))?;
    if i < 0 {
        return Err(Error::from(ErrorKind::InvalidArgements));
    }
    Ok(i)
}
