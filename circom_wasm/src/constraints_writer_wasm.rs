use constraint_list::ConstraintList;

use crate::{r1cs_porting_wasm::port_r1cs_wasm, log_writer_wasm::LogWasm};

pub trait R1csExporter {
    fn r1cs(&self, custom_gates: bool) -> Result<(Vec<u8>, LogWasm), ()>;
}

impl R1csExporter for ConstraintList {
    fn r1cs(&self, custom_gates: bool) -> Result<(Vec<u8>, LogWasm), ()> {
        port_r1cs_wasm(self, custom_gates)
        // Result::Ok(Vec::new())
    }
}

pub type R1csConstraintWriter = Box<dyn R1csExporter>;