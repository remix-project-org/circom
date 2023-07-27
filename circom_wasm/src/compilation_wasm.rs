use compiler::compiler_interface;
use compiler::compiler_interface::{Config, VCP};
use program_structure::error_definition::Report;
use program_structure::error_code::ReportCode;
use crate::VERSION;
use crate::error_reporting_wasm::print_reports;

pub struct CompilerConfig {
    pub js_folder: String,
    pub wasm_name: String,
    pub wat_file: String,
    pub wasm_file: String,
    pub c_folder: String,
    pub c_run_name: String,
    pub c_file: String,
    pub dat_file: String,
    pub wat_flag: bool,
    pub wasm_flag: bool,
    pub c_flag: bool,
    pub debug_output: bool,
    pub produce_input_log: bool,
    pub vcp: VCP,
}

pub fn compile (config: CompilerConfig) -> Result<(Vec<u8>), Vec<String>> {
    if config.wasm_flag {
        let circuit_result = compiler_interface::run_compiler(
            config.vcp,
            Config { debug_output: config.debug_output, produce_input_log: config.produce_input_log, wat_flag: config.wat_flag },
            VERSION
        );

        match circuit_result {
            Result::Err(()) => {
                return Err(Vec::new());
            },
            Result::Ok(circuit) => {
                match config.wasm_flag {
                    true => {
                        let wat_contents_result = circuit.generate_wasm();

                        match wat_contents_result {
                            Result::Err(()) => {
                                return Err(Vec::new());
                            },
                            Result::Ok(wat_contents) => {
                                let result = wat_to_wasm_for_browser(wat_contents);
                        
                                match result {
                                    Result::Err(report) => {
                                        let json_reports = print_reports(&[report]);
                                        return Err(json_reports)
                                    },
                                    Result::Ok(wasm_contents) => {
                                        return Ok(wasm_contents);
                                        // println!("{} {}", Colour::Green.paint("Written successfully:"), config.wasm_file);
                                    }
                                }
                            }
                        }
                    }
                    false => {}
                }
            }
        }
    }
    

    Result::Ok(Vec::new())
}

fn wat_to_wasm_for_browser(wat_contents: String) -> Result<Vec<u8>, Report> {
    use wast::Wat;
    use wast::parser::{self, ParseBuffer};

    let buf = ParseBuffer::new(&wat_contents).unwrap();
    let result_wasm_contents = parser::parse::<Wat>(&buf);
    match result_wasm_contents {
        Result::Err(error) => {
            Result::Err(Report::error(
                format!("Error translating the circuit from wat to wasm.\n\nException encountered when parsing WAT: {}", error),
                ReportCode::ErrorWat2Wasm,
            ))
        }
        Result::Ok(mut wat) => {
            let wasm_contents = wat.module.encode();
            match wasm_contents {
                Result::Err(error) => {
                    Result::Err(Report::error(
                        format!("Error translating the circuit from wat to wasm.\n\nException encountered when encoding WASM: {}", error),
                        ReportCode::ErrorWat2Wasm,
                    ))
                }
                Result::Ok(wasm_contents) => {
                    Ok(wasm_contents)
                }
            }
        }
    }
}
