use std::{
    env::set_var,
    io::Read,
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

use env_logger::Env;
use i2cdev::{core::I2CDevice, linux::LinuxI2CDevice};
use palantir::ping_thread;

mod db;
mod palantir;
mod tasks;

use crate::db::db_init;
use log::info;

fn main() {
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    let mut dev = LinuxI2CDevice::new("/dev/i2c-1", 0x26u16).unwrap();

    info!("I2c Read: {:?}", dev.smbus_read_byte());

    db_init();

    thread::spawn(move || ping_thread());

    info!("Current Iteration: {}", db::current_iteration_num());

    loop {
        sleep(Duration::from_millis(1000));
    }
}
