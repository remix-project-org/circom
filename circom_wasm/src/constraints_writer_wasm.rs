use constraint_list::ConstraintList;

use crate::r1cs_porting_wasm::port_r1cs_wasm;

pub trait R1csExporter {
    fn r1cs(&self, custom_gates: bool) -> Result<Vec<u8>, ()>;
}

impl R1csExporter for ConstraintList {
    fn r1cs(&self, custom_gates: bool) -> Result<Vec<u8>, ()> {
        port_r1cs_wasm(self, custom_gates)
        // Result::Ok(Vec::new())
    }
}

pub type R1csConstraintWriter = Box<dyn R1csExporter>;