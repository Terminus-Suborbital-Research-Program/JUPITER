use std::{env, process::Command, sync::{Arc, Mutex}, time::SystemTime};

use bin_packets::{ApplicationPacket, UnixTimestampMillis};
use bincode::{config::standard, decode_from_slice, encode_to_vec, error::DecodeError};
use log::info;
use sqlite::Connection;

fn packet_db_loc() -> String {
    env::var("PACKETS_DATABASE").expect("PACKETS_DATABASE not set!")
}

fn template_db_loc() -> String {
    env::var("TEMPLATE_DB").expect("TEMPLATE_DB not set!")
}

pub fn open_current_powercycle_database() -> Connection {
    match sqlite::open(packet_db_loc()) {
        Ok(db) => db,
        Err(_) => {
            // Probably the db doesn't exist
            create_db();
            sqlite::open(packet_db_loc()).unwrap()
        }
    }
}

fn create_db() {
    Command::new("cp").arg(template_db_loc()).arg(packet_db_loc()).output().unwrap();
    info!("Created new database for current powercycle");
}

#[derive(Debug)]
pub struct CachedPacket {
    data: Vec<u8>,
    timestamp: UnixTimestampMillis,
}

impl CachedPacket {
    pub fn new(packet: ApplicationPacket) -> Self {
        Self::from(packet)
    }

    pub(super) fn raw(packet: &[u8], time: u64) -> Self {
        CachedPacket {
            data: Vec::from(packet),
            timestamp: UnixTimestampMillis::new(time),
        }
    }

    pub fn inner(self) -> (Vec<u8>, UnixTimestampMillis) {
        (self.data, self.timestamp)
    }
}

impl From<ApplicationPacket> for CachedPacket {
    fn from(value: ApplicationPacket) -> Self {
        let now = SystemTime::now();
        let timestamp = now.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis() as u64;
        let timestamp = UnixTimestampMillis::new(timestamp);

        let data = encode_to_vec(value, standard()).unwrap();

        CachedPacket {
            data,
            timestamp,
        }
    }
}

impl TryFrom<CachedPacket> for ApplicationPacket {
    type Error = DecodeError;

    fn try_from(value: CachedPacket) -> Result<Self, Self::Error> {
        let res: (ApplicationPacket, usize) = decode_from_slice(&value.data, standard())?;
        Ok(res.0)
    }
}

pub struct PacketsCacheHandler {
    db: Arc<Mutex<Connection>>,
}

impl PacketsCacheHandler {
    pub fn new(database: &Arc<Mutex<Connection>>) -> Self {
        PacketsCacheHandler {
            db: Arc::clone(database)
        }
    }

    pub fn duplicate(&self) -> Self {
        PacketsCacheHandler {
            db: Arc::clone(&self.db)
        }
    }

    pub fn insert_cached_packet(&self, packet: CachedPacket) {
        let connection = self.db.lock().unwrap();
        
        let statement = "INSERT INTO current_powercycle (timestamp, data) VALUES (?, ?);";
        let mut statement = connection.prepare(statement).unwrap();
        let (data, time) = packet.inner();
        statement.bind((1, time.timestamp as i64)).unwrap();
        statement.bind((2, data.as_slice())).unwrap();
        statement.iter();
        for _ in statement.iter() {}
    }

    pub fn most_recent_packet(&self) -> Option<CachedPacket> {
        let connection = self.db.lock().unwrap();

        let statement = "SELECT * FROM current_powercycle ORDER BY timestamp DESC;";
        let mut statement = connection.prepare(statement).unwrap();
        match statement.next() {
            Ok(_) => {
                Some(CachedPacket::raw(
                    &statement.read::<Vec<u8>, _>("data").unwrap(),
                    statement.read::<i64, _>("timestamp").unwrap() as u64,
                ))
            }

            Err(_) => None,
        }
    }
}