use std::{
    env::set_var,
    io::Read,
    sync::{Arc, Mutex, RwLock},
    thread::{self, sleep},
    time::Duration,
};

use env_logger::Env;
use i2cdev::{core::I2CDevice, linux::LinuxI2CDevice};
use palantir::ping_thread;
use tasks::{PinStates, pin_states_thread};

mod db;
mod palantir;
mod states;
mod tasks;

use crate::db::db_init;
use log::info;

fn main() {
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    let mut atmega = LinuxI2CDevice::new("/dev/i2c-1", 0x26u16).unwrap();

    info!("I2c Read: {:?}", atmega.smbus_read_byte());
    let states = Arc::new(RwLock::new(PinStates::default()));

    let state_writer = Arc::clone(&states);
    thread::spawn(move || {
        pin_states_thread(atmega, state_writer);
    });

    db_init();

    thread::spawn(move || ping_thread());

    info!("Current Iteration: {}", db::current_iteration_num());

    loop {
        info!(
            "Current pin states (main thread): {}, {}",
            states.read().unwrap().gse_1_high(),
            states.read().unwrap().te_1_high()
        );
        sleep(Duration::from_millis(1000));
    }
}
