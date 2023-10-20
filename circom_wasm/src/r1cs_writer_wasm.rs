use std::collections::HashMap;

use compiler::num_bigint::BigInt;
use constraint_writers::r1cs_writer::{CustomGatesAppliedData, obtain_linear_combination_block, bigint_as_bytes, HeaderData, into_format, CustomGatesUsedData};

const SECTIONS: u8 = 5;
const MAGIC: &[u8] = b"r1cs";
const VERSION: &[u8] = &[1, 0, 0, 0];
const HEADER_TYPE: &[u8] = &[1, 0, 0, 0];
const CONSTRAINT_TYPE: &[u8] = &[2, 0, 0, 0];
const WIRE2LABEL_TYPE: &[u8] = &[3, 0, 0, 0];
const CUSTOM_GATES_USED_TYPE: &[u8] = &[4, 0, 0, 0];
const CUSTOM_GATES_APPLIED_TYPE: &[u8] = &[5, 0, 0, 0];
const PLACE_HOLDER: &[u8] = &[3, 3, 3, 3, 3, 3, 3, 3];

fn initialize_r1cs_section(output: &mut Vec<u8>, header: &[u8], start_pointer: usize) -> Result<(usize), ()> {
    let remaining_bytes = if start_pointer > 0 {
         output.split_off(start_pointer)
    } else {
        vec![]
    };
    output.extend_from_slice(header);
    let go_back = output.len();
    output.extend_from_slice(PLACE_HOLDER);
    output.extend_from_slice(&remaining_bytes);
    // writer.write_all(header).map_err(|_err| {})?;
    //writer.flush().map_err(|_err| {})?;
    //writer.flush().map_err(|_err| {})?;
    Result::Ok(go_back)
}

fn end_r1cs_section(output: &mut Vec<u8>, go_back: usize, size: usize) -> Result<usize, ()> {
    let go_back_1= output.len();
    let remaining_bytes = output.split_off(go_back);
    let (stream, _) = bigint_as_bytes(&BigInt::from(size), 8);

    output.extend_from_slice(&stream);
    output.extend_from_slice(&remaining_bytes);
    Result::Ok(go_back_1)
}

fn write_constraint_wasm<T>(
    output: &mut Vec<u8>,
    a: &HashMap<T, BigInt>,
    b: &HashMap<T, BigInt>,
    c: &HashMap<T, BigInt>,
    field_size: usize,
    go_back: usize
) -> Result<(usize, usize), ()> where T: AsRef<[u8]> + std::cmp::Ord + std::hash::Hash {
    let (block_a, size_a) = obtain_linear_combination_block(a, field_size);
    let (block_b, size_b) = obtain_linear_combination_block(b, field_size);
    let (block_c, size_c) = obtain_linear_combination_block(c, field_size);
    let remaining_bytes = output.split_off(go_back);
    output.extend_from_slice(&block_a);
    //file.flush().map_err(|_err| {})?;
    output.extend_from_slice(&block_b);
    //file.flush().map_err(|_err| {})?;
    output.extend_from_slice(&block_c);
    //file.flush().map_err(|_err| {})?;
    let go_back = output.len();

    output.extend_from_slice(&remaining_bytes);
    Result::Ok((size_a + size_b + size_c, go_back))
}

fn initialize_r1cs_out(num_sections: u8) -> Result<Vec<u8>, ()> {
    let mut output = Vec::new();
    
    output.extend_from_slice(MAGIC);
    // writer.write_all(MAGIC).map_err(|_err| {})?;
    //writer.flush().map_err(|_err| {})?;
    output.extend_from_slice(VERSION);
    // writer.write_all(VERSION).map_err(|_err| {})?;
    //writer.flush().map_err(|_err| {})?;
    output.extend_from_slice(&[num_sections, 0, 0, 0]);
    // writer.write_all(&[num_sections, 0, 0, 0]).map_err(|_err| {})?;
    //writer.flush().map_err(|_err| {})?;
    Result::Ok(output)
}

pub struct R1CSWriterWasm {
    field_size: usize,
    pub output: Vec<u8>,
    sections: [bool; SECTIONS as usize]
}

pub struct HeaderSectionWasm {
    output: Vec<u8>,
    go_back: usize,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize]
}

pub struct ConstraintSectionWasm {
    output: Vec<u8>,
    number_of_constraints: usize,
    pub go_back: usize,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize]
}

pub struct SignalSectionWasm {
    output: Vec<u8>,
    go_back: usize,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize]
}

pub struct CustomGatesUsedSectionWasm {
    output: Vec<u8>,
    go_back: usize,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize]
}

pub struct CustomGatesAppliedSectionWasm {
    output: Vec<u8>,
    go_back: usize,
    size: usize,
    index: usize,
    field_size: usize,
    sections: [bool; SECTIONS as usize]
}

impl R1CSWriterWasm {
    pub fn new(
        field_size: usize,
        custom_gates: bool
    ) -> Result<R1CSWriterWasm, ()> {
        let sections = [false; SECTIONS as usize];
        let num_sections: u8 = if custom_gates { 5 } else { 3 };
        let output = initialize_r1cs_out(num_sections)?;
        Result::Ok(R1CSWriterWasm { output, sections, field_size })
    }

    pub fn start_header_section(mut r1cs: R1CSWriterWasm, start_pointer: usize) -> Result<HeaderSectionWasm, ()> {
        let start = initialize_r1cs_section(&mut r1cs.output, HEADER_TYPE, start_pointer)?;
        Result::Ok(HeaderSectionWasm {
            output: r1cs.output,
            go_back: start,
            size: 0,
            index: 0,
            field_size: r1cs.field_size,
            sections: r1cs.sections,
        })
    }

    pub fn start_constraints_section(mut r1cs: R1CSWriterWasm, start_pointer: usize) -> Result<ConstraintSectionWasm, ()> {
        let start = initialize_r1cs_section(&mut r1cs.output, CONSTRAINT_TYPE, start_pointer)?;
        Result::Ok(ConstraintSectionWasm {
            number_of_constraints: 0,
            output: r1cs.output,
            go_back: start,
            size: 0,
            index: 1,
            field_size: r1cs.field_size,
            sections: r1cs.sections,
        })
    }

    pub fn start_signal_section(mut r1cs: R1CSWriterWasm, start_pointer: usize) -> Result<SignalSectionWasm, ()> {
        let start = initialize_r1cs_section(&mut r1cs.output, WIRE2LABEL_TYPE, start_pointer)?;
        Result::Ok(SignalSectionWasm {
            output: r1cs.output,
            go_back: start,
            size: 0,
            index: 2,
            field_size: r1cs.field_size,
            sections: r1cs.sections,
        })
    }

    pub fn start_custom_gates_used_section(mut r1cs: R1CSWriterWasm, start_pointer: usize) -> Result<CustomGatesUsedSectionWasm, ()> {
        let start = initialize_r1cs_section(&mut r1cs.output, CUSTOM_GATES_USED_TYPE, start_pointer)?;
        Result::Ok(CustomGatesUsedSectionWasm {
            output: r1cs.output,
            go_back: start,
            size: 0,
            index: 3,
            field_size: r1cs.field_size,
            sections: r1cs.sections
        })
    }

    pub fn start_custom_gates_applied_section(mut r1cs: R1CSWriterWasm, start_pointer: usize) -> Result<CustomGatesAppliedSectionWasm, ()> {
        let start = initialize_r1cs_section(&mut r1cs.output, CUSTOM_GATES_APPLIED_TYPE, start_pointer)?;
        Result::Ok(CustomGatesAppliedSectionWasm {
            output: r1cs.output,
            go_back: start,
            size: 0,
            index: 4,
            field_size: r1cs.field_size,
            sections: r1cs.sections
        })
    }

    pub fn finish_writing(mut r1cs: R1CSWriterWasm) -> Result<(), ()> {
	// r1cs.writer.flush().map_err(|_err| {})
        Result::Ok(())
    }
}

impl HeaderSectionWasm {
    pub fn write_section(&mut self, data: HeaderData) -> Result<(), ()> {
        let (field_stream, bytes_field) = bigint_as_bytes(&data.field, self.field_size);
        let (length_stream, bytes_size) = bigint_as_bytes(&BigInt::from(self.field_size), 4);
        let mut output = self.output.clone();
        let remaining_bytes = output.split_off(self.go_back);

        output.extend_from_slice(&length_stream);
        output.extend_from_slice(&field_stream);
        self.size += bytes_field + bytes_size;

        let data_stream = [
            [data.total_wires, 4],
            [data.public_outputs, 4],
            [data.public_inputs, 4],
            [data.private_inputs, 4],
            [data.number_of_labels, 8],
            [data.number_of_constraints, 4],
        ];
        for data in &data_stream {
            let (stream, size) = bigint_as_bytes(&BigInt::from(data[0]), data[1]);
            self.size += size;
            output.extend_from_slice(&stream);
        }
        // self.go_back = output.len();
        output.extend_from_slice(&remaining_bytes);
        self.output = output;
        Result::Ok(())
    }

    pub fn end_section(mut self) -> Result<(R1CSWriterWasm, usize), ()> {
        let go_back = end_r1cs_section(&mut self.output, self.go_back, self.size)?;
        let mut sections = self.sections;
        let index = self.index;
        sections[index] = true;
        Result::Ok((R1CSWriterWasm { output: self.output, field_size: self.field_size, sections }, go_back))
    }
}

type Constraint = HashMap<usize, BigInt>;
impl ConstraintSectionWasm {
    pub fn write_constraint_usize(
        &mut self,
        a: &Constraint,
        b: &Constraint,
        c: &Constraint,
    ) -> Result<(), ()> {
        let field_size = self.field_size;
        let mut r1cs_a = HashMap::new();
        for (k, v) in a {
            let (_, bytes) = BigInt::from(*k).to_bytes_le();
            r1cs_a.insert(bytes, v.clone());
        }
        let mut r1cs_b = HashMap::new();
        for (k, v) in b {
            let (_, bytes) = BigInt::from(*k).to_bytes_le();
            r1cs_b.insert(bytes, v.clone());
        }
        let mut r1cs_c = HashMap::new();
        for (k, v) in c {
            let (_, bytes) = BigInt::from(*k).to_bytes_le();
            r1cs_c.insert(bytes, v.clone());
        }
        let (size, go_back) = write_constraint_wasm(&mut self.output, &r1cs_a, &r1cs_b, &r1cs_c, field_size, self.go_back)?;
        self.size += size;
        self.number_of_constraints += 1;
        // self.go_back = go_back;
        Result::Ok(())
    }

    pub fn end_section(mut self) -> Result<(R1CSWriterWasm, usize), ()> {
        let go_back = end_r1cs_section(&mut self.output, self.go_back, self.size)?;
        let mut sections = self.sections;
        let index = self.index;
        sections[index] = true;
        Result::Ok((R1CSWriterWasm {
            output: self.output,
            field_size: self.field_size,
            sections
        }, go_back))
    }

    pub fn constraints_written(&self) -> usize {
        self.number_of_constraints
    }
}

impl SignalSectionWasm {
    pub fn write_signal<T>(
        &mut self,
        bytes: &T
    ) -> Result<(), ()> where T: AsRef<[u8]> {
        let (bytes, size) = into_format(bytes.as_ref(), 8);
        self.size += size;
        let mut output = self.output.clone();
        let remaining_bytes = output.split_off(self.go_back);

        output.extend_from_slice(&bytes);
        output.extend_from_slice(&remaining_bytes);
        self.output = output;
        Result::Ok(())
        //self.writer.flush().map_err(|_err| {})
    }

    pub fn write_signal_usize(&mut self, signal: usize) -> Result<(), ()> {
        let (_, as_bytes) = BigInt::from(signal).to_bytes_le();
        SignalSectionWasm::write_signal(self, &as_bytes)
    }

    pub fn end_section(mut self) -> Result<(R1CSWriterWasm, usize), ()> {
        let go_back = end_r1cs_section(&mut self.output, self.go_back, self.size)?;
        let mut sections = self.sections;
        let index = self.index;
        sections[index] = true;
        Result::Ok((R1CSWriterWasm {
            output: self.output,
            field_size: self.field_size,
            sections
        }, go_back))
    }
}

impl CustomGatesUsedSectionWasm {
    pub fn write_custom_gates_usages(&mut self, data: CustomGatesUsedData) -> Result<(), ()> {
        let no_custom_gates = data.len();
        let (no_custom_gates_stream, no_custom_gates_size) =
            bigint_as_bytes(&BigInt::from(no_custom_gates), 4);
        self.size += no_custom_gates_size;
        let mut output = self.output.clone();
        let remaining_bytes = output.split_off(self.go_back);

        output.extend_from_slice(&no_custom_gates_stream);
        //self.writer.flush().map_err(|_err| {})?;

        for custom_gate in data {
            let custom_gate_name = custom_gate.0;
            let custom_gate_name_stream = custom_gate_name.as_bytes();
            self.size += custom_gate_name_stream.len() + 1;
            output.extend_from_slice(custom_gate_name_stream);
            output.extend_from_slice(&[0]);
            //self.writer.flush().map_err(|_err| {})?;

            let custom_gate_parameters = custom_gate.1;
            let no_custom_gate_parameters = custom_gate_parameters.len();
            let (no_custom_gate_parameters_stream, no_custom_gate_parameters_size) =
                bigint_as_bytes(&BigInt::from(no_custom_gate_parameters), 4);
            self.size += no_custom_gate_parameters_size;
            output.extend_from_slice(&no_custom_gate_parameters_stream);
            //self.writer.flush().map_err(|_err| {})?;

            for parameter in custom_gate_parameters {
                let (parameter_stream, parameter_size) = bigint_as_bytes(&parameter, self.field_size);
                self.size += parameter_size;
                output.extend_from_slice(&parameter_stream);
                //self.writer.flush().map_err(|_err| {})?;
            }
        }
        output.extend_from_slice(&remaining_bytes);
        self.output = output;

        Result::Ok(())
    }

    pub fn end_section(mut self) -> Result<(R1CSWriterWasm, usize), ()> {
        let go_back = end_r1cs_section(&mut self.output, self.go_back, self.size)?;
        let mut sections = self.sections;
        let index = self.index;
        sections[index] = true;
        Result::Ok((R1CSWriterWasm {
            output: self.output,
            field_size: self.field_size,
            sections
        }, go_back))
    }
}

impl CustomGatesAppliedSectionWasm {
    pub fn write_custom_gates_applications(&mut self, data: CustomGatesAppliedData) -> Result<(), ()> {
        let no_custom_gate_applications = data.len();
        let (no_custom_gate_applications_stream, no_custom_gate_applications_size) =
            bigint_as_bytes(&BigInt::from(no_custom_gate_applications), 4);
        self.size += no_custom_gate_applications_size;
        let mut output = self.output.clone();
        let remaining_bytes = output.split_off(self.go_back);

        output.extend_from_slice(&no_custom_gate_applications_stream);
        //self.writer.flush().map_err(|_err| {})?;

        for custom_gate_application in data {
            let custom_gate_index = custom_gate_application.0;
            let (custom_gate_index_stream, custom_gate_index_size) =
                bigint_as_bytes(&BigInt::from(custom_gate_index), 4);
            self.size += custom_gate_index_size;
            output.extend_from_slice(&custom_gate_index_stream);
            //self.writer.flush().map_err(|_err| {})?;

            let custom_gate_signals = custom_gate_application.1;
            let no_custom_gate_signals = custom_gate_signals.len();
            let (no_custom_gate_signals_stream, no_custom_gate_signals_size) =
                bigint_as_bytes(&BigInt::from(no_custom_gate_signals), 4);
            self.size += no_custom_gate_signals_size;
            output.extend_from_slice(&no_custom_gate_signals_stream);
            //self.writer.flush().map_err(|_err| {})?;

            for signal in custom_gate_signals {
                let (signal_stream, signal_size) = bigint_as_bytes(&BigInt::from(signal), 8);
                self.size += signal_size;
                output.extend_from_slice(&signal_stream);
                //self.writer.flush().map_err(|_err| {})?;
            }
        }
	//self.writer.flush().map_err(|_err| {})?;
        output.extend_from_slice(&remaining_bytes);
        self.output = output;
        Result::Ok(())
    }

    pub fn end_section(mut self) -> Result<(R1CSWriterWasm, usize), ()> {
        let go_back = end_r1cs_section(&mut self.output, self.go_back, self.size)?;
        let mut sections = self.sections;
        let index = self.index;
        sections[index] = true;
        Result::Ok((R1CSWriterWasm {
            output: self.output,
            field_size: self.field_size,
            sections
        }, go_back))
    }
}