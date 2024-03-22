mod compilation_wasm;
mod execution_wasm;
mod parser_wasm;
mod type_analysis_wasm;
mod include_logic_wasm;
mod error_reporting_wasm;
mod constraints_wasm;
mod constraints_writer_wasm;
mod r1cs_porting_wasm;
mod r1cs_writer_wasm;
mod log_writer_wasm;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

use std::collections::HashMap;

use execution_wasm::ExecutionConfig;
use compiler::hir::very_concrete_program::VCP;
use js_sys::Array;
use log_writer_wasm::LogWasm;
use program_structure::file_definition::FileLibrary;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use crate::compilation_wasm::CompilerConfig;
use crate::execution_wasm::generate_output_r1cs;

struct CircuitConfig {
    prime: String
}
#[wasm_bindgen]
pub struct CompilationResult {
    program: Vec<u8>,
    input_signals: HashMap<String, Vec<String>>,
    report: String,
    log: LogWasm
}


#[wasm_bindgen]
impl CompilationResult {
    pub fn program(&self) -> js_sys::Uint8Array {
        let result = js_sys::Uint8Array::new_with_length(self.program.len() as u32);
        result.copy_from(&self.program);
        result
    }

    pub fn input_signals(&self, name: &str) -> Array {
        if let Some(signals) = self.input_signals.get(name) {
            let result = js_sys::Array::new_with_length(signals.len() as u32);

            for signal in signals {
                result.push(&JsValue::from(signal));
            }
            result
        } else {
            Array::new()
        }
    }

    pub fn report(&self) -> JsValue {
        JsValue::from_str(&self.report)
    }

    pub fn log (&self) -> Array {
        let logs_vec = LogWasm::print_array(&self.log);
        let result = js_sys::Array::new_with_length(logs_vec.len() as u32);

        for log in logs_vec {
            result.push(&JsValue::from(log));
        }
        result
    }
}

#[wasm_bindgen]
pub struct R1csResult {
    program: Vec<u8>,
    report: String,
    log: LogWasm
}

#[wasm_bindgen]
impl R1csResult {
    pub fn program(&self) -> js_sys::Uint8Array {
        let result = js_sys::Uint8Array::new_with_length(self.program.len() as u32);

        result.copy_from(&self.program);
        result
    }

    pub fn report (&self) -> JsValue {
        JsValue::from_str(&self.report)
    }

    pub fn log (&self) -> Array {
        let logs_vec = LogWasm::print_array(&self.log);
        let result = js_sys::Array::new_with_length(logs_vec.len() as u32);

        for log in logs_vec {
            result.push(&JsValue::from(log));
        }
        result
    }
}

#[wasm_bindgen]
pub struct ParseResult {
    report: String,
    file_library: FileLibrary
}

#[wasm_bindgen]
impl ParseResult {
    pub fn report(&self) -> JsValue {
        JsValue::from_str(&self.report)
    }

    pub fn get_report_name(&self, report_id: usize) -> JsValue {
        let file_storage = self.file_library.to_storage();
        if let Some(file_name) = file_storage.get(report_id) {
            JsValue::from_str(file_name.name())
        } else {
            JsValue::from_str("")
        }
    }
}

#[wasm_bindgen]
pub fn compile (file_name: String, sources: JsValue, config: JsValue) -> CompilationResult {
    let processed_input = process_sources(sources, config);

    match processed_input {
        Result::Err(report) => {
            CompilationResult { program: Vec::new(), input_signals: HashMap::new(), report, log: LogWasm::new() }
        },
        Result:: Ok((link_libraries, link_libraries_sources, circuit_config)) => {
            let result = start_compiler(file_name, link_libraries, link_libraries_sources, circuit_config);

            match result {
                Result::Err(report) => {
                    // println!("{}", Colour::Red.paint("previous errors were found"));
                    let report_string = format!("[{}]", report.join(","));
                    let compilation_result = CompilationResult { program: Vec::new(), input_signals: HashMap::new(), report: report_string, log: LogWasm::new() };
    
                    return compilation_result;
                },
                Result::Ok((wasm_contents, templates_name_values, mut log)) => {
                    // println!("{}", Colour::Green.paint("Everything went okay"));
                    log.is_successful = true;
                    let compilation_result = CompilationResult { program: wasm_contents, input_signals: templates_name_values, report: "".to_string(), log };
    
                    return compilation_result;
                }
            }
        }
    }
}

#[wasm_bindgen]
pub fn parse(file_name: String, sources: JsValue) -> ParseResult {
    let mut link_libraries = Vec::new();
    let mut link_libraries_sources = Vec::new();

    if let Ok(file_sources) = sources.dyn_into::<js_sys::Object>() {
        let source_keys = js_sys::Object::keys(&(file_sources));
        let source_keys = source_keys.iter().map(|key| key.as_string().unwrap());
        let source_values = js_sys::Object::values(&file_sources);
        let source_values = source_values.iter().map(|value| value.as_string().unwrap_or_else(|| String::from("")));

        link_libraries.extend(source_keys);
        link_libraries_sources.extend(source_values);
    }
    let result = start_parser(file_name, link_libraries.clone(), link_libraries_sources);

    match result {
        Result::Err((file_library, report)) => {
            // eprintln!("{}", Colour::Red.paint("previous errors were found"));
            let report_string = format!("[{}]", report.join(","));
            return ParseResult { report: report_string, file_library };
        },
        Result::Ok((file_library, warns)) => {
            // println!("{}", Colour::Green.paint("Everything went okay"));
            let report_string = format!("[{}]", warns.join(","));
            return ParseResult { report: report_string, file_library: file_library }
        }
    }
}

#[wasm_bindgen]
pub fn generate_r1cs(file_name: String, sources: JsValue, config: JsValue) -> R1csResult {
    let processed_input = process_sources(sources, config);

    match processed_input {
        Result::Err(report) => {
            R1csResult { program: Vec::new(), report, log: LogWasm::new() }
        },
        Result::Ok((link_libraries, link_libraries_sources, circuit_config)) => {
            let result = start_r1cs(file_name, link_libraries, link_libraries_sources, circuit_config);

            match result {
                Result::Err(report) => {
                    // println!("{}", Colour::Red.paint("previous errors were found"));
                    let report_string = format!("[{}]", report.join(","));

                    R1csResult { program: Vec::new(), report: report_string, log: LogWasm::new() }
                },
                Result::Ok((r1cs, mut log)) => {
                    log.is_successful = true;
                    // println!("{}", Colour::Green.paint("Everything went okay"));
                    R1csResult { program: r1cs, report: "".to_string(), log }
                }
            }
        }
    }
}

fn start_compiler(file_name: String, link_libraries: Vec<String>, link_libraries_sources: Vec<String>, config: CircuitConfig) -> Result<(Vec<u8>, HashMap<String, Vec<String>>, LogWasm), Vec<String>> {
    let program = parser_wasm::parse_project(file_name, link_libraries, link_libraries_sources);

    match program {
        Result::Err((_, report)) => {
            Result::Err(report)
        }, Result::Ok((mut program_archive, _, warnings)) => {
            let parse_report = type_analysis_wasm::analyse_project(&mut program_archive)?;
            let execution_config = get_execution_config(config);
            let (_, circuit, log) = execution_wasm::execute_project(program_archive.clone(), execution_config)?;
            let compilation_config = get_compiler_config(circuit);
            let compilation_details = compilation_wasm::compile(compilation_config);
            match compilation_details {
                Result::Err(mut report) => {
                    for rp in parse_report.iter() {
                        report.push(rp.to_string());
                    }
                    for warns in warnings.iter()  {
                        report.push(warns.to_string());
                    }
                    return Err(report);
                }
                Result::Ok(wasm_contents) => {
                    let circuit_templates = program_archive.get_templates();
                    let mut template_names_values = HashMap::new();
        
                    for template_data in circuit_templates.iter() {
                        let input_signals = template_data.1.get_inputs();
        
                        template_names_values.insert(template_data.0.to_string(), input_signals.keys().filter(|key| !key.is_empty()).cloned().collect());
                    }
                    
                    return Result::Ok((wasm_contents, template_names_values, log));
                }
            }
        }
    }
}

fn start_parser(file_name: String, link_libraries: Vec<String>, link_libraries_sources: Vec<String>) -> Result<(FileLibrary, Vec<String>), (FileLibrary, Vec<String>)> {
    let program = parser_wasm::parse_project(file_name, link_libraries, link_libraries_sources);

    match program {
        Result::Err((file_library, report)) => {
            Result::Err((file_library, report))
        },
        Result::Ok((mut program_archive, file_library, parse_warnings)) => {
            let analysis = type_analysis_wasm::analyse_project(&mut program_archive);

            match analysis {
                Result::Err(report) => {
                    Result::Err((file_library, report))
                },
                Result::Ok(mut analysis_warnings) => {
                    for warns in parse_warnings.iter() {
                        analysis_warnings.push(warns.to_string());
                    }
                    Result::Ok((file_library, analysis_warnings))
                }
            }
        }
    }
}

fn start_r1cs(file_name: String, link_libraries: Vec<String>, link_libraries_sources: Vec<String>, config: CircuitConfig) -> Result<(Vec<u8>, LogWasm), Vec<String>> {
    let program = parser_wasm::parse_project(file_name, link_libraries, link_libraries_sources);
   
   match program {
        Result::Err((_, report)) => {
            Result::Err(report)
        },
        Result::Ok((mut program_archive, _, _)) => {
            type_analysis_wasm::analyse_project(&mut program_archive)?;
            let execution_config = get_execution_config(config);
            let (exporter, _, _) = execution_wasm::execute_project(program_archive.clone(), execution_config)?;
            let r1cs_details = generate_output_r1cs(exporter.as_ref(), program_archive.custom_gates)?;
        
            Result::Ok(r1cs_details)
        }
   }
}

fn process_sources(sources: JsValue, config: JsValue) -> Result<(Vec<String>, Vec<String>, CircuitConfig), String> {
    if let Some(config) = config.dyn_into::<js_sys::Object>().ok() {
        let prime: JsValue = js_sys::Reflect::get(&config, &"prime".into()).unwrap();
        let prime = prime.as_string().unwrap();
        let circuit_config = CircuitConfig {
            prime
        };
        let mut link_libraries = Vec::new();
        let mut link_libraries_sources = Vec::new();

        if let Ok(file_sources) = sources.dyn_into::<js_sys::Object>() {
            let source_keys = js_sys::Object::keys(&(file_sources));
            let source_keys = source_keys.iter().map(|key| key.as_string().unwrap());
            let source_values = js_sys::Object::values(&file_sources);
            let source_values = source_values.iter().map(|value| value.as_string().unwrap_or_else(|| String::from("")));

            link_libraries.extend(source_keys);
            link_libraries_sources.extend(source_values);
            Result::Ok((link_libraries, link_libraries_sources, circuit_config))
        } else {
            Result::Err("File sources not found".to_string())
        }
    } else {
        Result::Err("Invalid config".to_string())
    }
}

fn get_execution_config(circuit_config: CircuitConfig) -> ExecutionConfig {
    ExecutionConfig {
        no_rounds: 0,
        flag_p: false,
        flag_s: false,
        flag_f: false,
        flag_old_heuristics: false,
        flag_verbose: false,
        inspect_constraints_flag: false,
        json_constraint_flag: false,
        json_substitution_flag: false,
        sym_flag: false,
        sym: "".to_string(),
        json_constraints: "".to_string(),
        prime: circuit_config.prime,
    }
}

fn get_compiler_config(circuit: VCP) -> CompilerConfig {
    CompilerConfig {
        vcp: circuit,
        debug_output: false,
        c_flag: false,
        wasm_flag: true,
        wat_flag: false,
        js_folder: "".to_string(),
        wasm_name: "".to_string(),
        c_folder: "".to_string(),
        c_run_name: "".to_string(),
        c_file: "".to_string(),
        dat_file: "".to_string(),
        wat_file: "".to_string(),
        wasm_file: "".to_string(),
        produce_input_log: false,
    }
}
