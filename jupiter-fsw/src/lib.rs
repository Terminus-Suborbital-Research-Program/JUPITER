use std::time::{SystemTime, UNIX_EPOCH};

use bin_packets::UnixTimestampMillis;

pub mod db;
pub mod tasks;

pub fn now_millis() -> UnixTimestampMillis {
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as u64;
    UnixTimestampMillis::new(now)
}