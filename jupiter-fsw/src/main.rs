use std::{env, sync::{Arc, Mutex}};

use bin_packets::ApplicationPacket;
use jupiter_fsw::db::{CachedPacket, PacketsCacheHandler};

fn main() {
    let packets_db_loc = env::var("PACKETS_DATABASE").expect("Couldn't find packet database location!");
    let connection = sqlite::open(packets_db_loc).expect("Database open failure!");
    let connection = Arc::new(Mutex::new(connection));
    let handler = PacketsCacheHandler::new(&connection);

    let demo_packet = ApplicationPacket::Command(bin_packets::CommandPacket::Ping);
    let demo_packet = CachedPacket::new(demo_packet);
    println!("Last packet: {:?}", handler.most_recent_packet());
    handler.insert_cached_packet(demo_packet);

    println!("Hello, world!");
}
