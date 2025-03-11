use std::{env, sync::{Arc, Mutex}};

use bin_packets::ApplicationPacket;
use jupiter_fsw::{db::{open_current_powercycle_database, CachedPacket, PacketsCacheHandler}, tasks::{tasks::TelemetryLogger, Task}};


fn main() {
    let db = open_current_powercycle_database();
    let db = PacketsCacheHandler::new(&Arc::new(Mutex::new(db)));

    let mut logger = TelemetryLogger::new(&db);

    std::thread::spawn(move || {
        logger.task(&mut ());
    });

    loop {}
}
