use std::{
    sync::{Arc, RwLock},
    time::Duration,
};

use i2cdev::{core::I2CDevice as _, linux::LinuxI2CDevice};
use log::{info, warn};

pub struct PinStates {
    gse_1: bool,
    te_1: bool,
}

#[allow(dead_code)]
impl PinStates {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn gse_1_high(&self) -> bool {
        self.gse_1
    }

    pub fn te_1_high(&self) -> bool {
        self.te_1
    }

    pub fn set_pins(&mut self, gse_1: bool, te_1: bool) {
        self.gse_1 = gse_1;
        self.te_1 = te_1;
    }
}

impl Default for PinStates {
    fn default() -> Self {
        PinStates {
            gse_1: false,
            te_1: false,
        }
    }
}

// Divirging function to handle reading from the pins
pub fn pin_states_thread(mut atmega: LinuxI2CDevice, pin_states: Arc<RwLock<PinStates>>) -> ! {
    loop {
        let bytes = match atmega.smbus_read_byte() {
            Ok(b) => b,
            Err(e) => {
                warn!("Error reading ATMEGA: {:?}", e);
                0
            }
        };

        let gse = 1u8 | bytes == 1u8;
        let te = 0b10u8 | bytes == 0b10u8;
        info!("GSE: {}, TE: {}", gse, te);

        match pin_states.write() {
            Ok(mut writer) => {
                writer.set_pins(gse, te);
            }

            Err(e) => {
                warn!("Errer getting writer! Error: {:?}", e);
            }
        }

        std::thread::sleep(Duration::from_millis(1000));
    }
}
