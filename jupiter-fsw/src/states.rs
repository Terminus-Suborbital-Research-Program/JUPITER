use std::{
    sync::{Arc, RwLock},
    time::Instant,
};

use bin_packets::JupiterPhase;
use log::{info, warn};

use crate::tasks::PinStates;

pub struct JupiterStateMachine {
    phase: JupiterPhase,
    start_time: Instant,
    pin_struct: Arc<RwLock<PinStates>>,
}

impl JupiterStateMachine {
    pub fn new(pins: Arc<RwLock<PinStates>>) -> Self {
        JupiterStateMachine {
            phase: JupiterPhase::PowerOff,
            start_time: Instant::now(),//Should change this to be uninitalized
            pin_struct: pins,
        }
    }

    pub fn current_phase(&self) -> JupiterPhase {
        self.phase
    }

    pub fn update(&mut self) -> Option<JupiterPhase> {
        let old_phase = self.phase;
        let pins = self.pin_struct.read().unwrap();
        self.phase = match self.current_phase() {
            JupiterPhase::PowerOn => {
                if self.start_time.elapsed().as_secs() > 180 {
                    self.start_time = Instant::now();
                    JupiterPhase::Launch
                }
                else{
                    JupiterPhase::PowerOn
                }
            }

            JupiterPhase::Launch => {
                if pins.te_1_high(){
                    JupiterPhase::EjectionPhase
                } 
                else {
                    if self.start_time.elapsed().as_secs() > 200 {
                        JupiterPhase::EjectionPhase
                    }
                    else{
                        JupiterPhase::Launch
                    }
                }
            }

            JupiterPhase::SkirtEjection => JupiterPhase::SkirtEjection,

            JupiterPhase::PowerOff => {
                if pins.gse_1_high() {
                    self.start_time = Instant::now();
                    JupiterPhase::PowerOn
                }  else {
                    JupiterPhase::PowerOff
                }
            }
            _ => {
                unimplemented!("Undefined");
            }
        };

        if old_phase == self.phase {
            None
        } else {
            Some(self.current_phase())
        }
    }
}
