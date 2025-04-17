use std::{
    env::set_var,
    io::Read,
    sync::{Arc, Mutex, RwLock},
    thread::{self, sleep},
    time::Duration,
};

use bin_packets::JupiterPhase;
use env_logger::Env;
use gpio::GpioOut;
use i2cdev::{core::I2CDevice, linux::LinuxI2CDevice};
use palantir::ping_thread;
use rppal::gpio::Gpio;
use serialport::{SerialPortBuilder, available_ports};
use states::JupiterStateMachine;
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

    let serial_port_info = available_ports().expect("Serial device error!");
    let mut tty_s_found = false;
    let mut tty_a_found = false;
    for port in serial_port_info.iter() {
        info!(
            "Serial port info found: {}, {:?}",
            port.port_name, port.port_type
        );
        if port.port_name == "/dev/ttyS0" {
            tty_s_found = true;
        }
        if port.port_name == "/dev/ttyAMA0" {
            tty_a_found = true;
        }
    }

    let gpio = Gpio::new().expect("Failed to open GPIO");
    let mut gpio_pin = gpio.get(5).unwrap().into_output();

    let port = if tty_a_found {
        serialport::new("/dev/ttyAMA0", 115200).open()
    } else if tty_s_found {
        serialport::new("/dev/ttyAMA0", 115200).open()
    } else {
        panic!("Serial open error!")
    };
    let mut port = port.expect("Couldn't open serial port!");

    let mut atmega = LinuxI2CDevice::new("/dev/i2c-1", 0x26u16).unwrap();

    info!("I2c Read: {:?}", atmega.smbus_read_byte());
    let states = Arc::new(RwLock::new(PinStates::default()));

    let state_writer = Arc::clone(&states);
    let pin_state_state_machine = Arc::clone(&states);

    let mut state_machine = JupiterStateMachine::new(pin_state_state_machine);

    thread::spawn(move || {
        pin_states_thread(atmega, state_writer);
    });

    db_init();

    thread::spawn(move || ping_thread());

    info!("Current Iteration: {}", db::current_iteration_num());

    loop {
        let buf = "hello world!".as_bytes();
        let w = port.write(buf);
        info!("Serial write: {:?}", w);

        let transition = state_machine.update();

        if state_machine.current_phase() == JupiterPhase::EjectionPhase {
            gpio_pin.set_high();
        } else {
            gpio_pin.set_low();
        }

        if transition.is_some() {
            let new_state = transition.unwrap();
            info!("New State: {:?}", new_state);

            match new_state {
                JupiterPhase::PowerOn => {
                    info!("Yippee!");
                }
                JupiterPhase::Launch => {
                    info!("Hold on to your hats!");
                }
                JupiterPhase::EjectionPhase => {
                    info!("Get OUTTA HERE!");
                }

                // JupiterPhase::SkirtEjection => {
                //     info!("Skirt ejection, deploying ejector payload");
                // }
                _ => {
                    info!("No actions to take.");
                }
            }
        }

        sleep(Duration::from_millis(1000));
    }
}
