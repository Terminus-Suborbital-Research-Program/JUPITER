use std::{
    env::set_var,
    sync::{Arc, Mutex},
    thread::sleep,
    time::Duration,
};

use env_logger::Env;

mod db;
mod tasks;

use crate::db::db_init;
use log::info;

fn main() {
    let env = Env::default().filter_or("LOG_LEVEL", "info");
    env_logger::init_from_env(env);

    db_init();

    info!("Current Iteration: {}", db::current_iteration_num());

    loop {
        sleep(Duration::from_millis(1000));
    }
}
