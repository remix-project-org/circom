mod compilation_wasm;
mod execution_wasm;
mod parser_wasm;
mod type_analysis_wasm;
mod include_logic_wasm;
mod error_reporting_wasm;
mod constraints_wasm;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

use std::collections::HashMap;

use js_sys::Array;
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
    report: String
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
            let mut result = js_sys::Array::new_with_length(signals.len() as u32);

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
}

#[wasm_bindgen]
pub fn compile (file_name: String, sources: JsValue, config: JsValue) -> CompilationResult {
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
        }
        let result = start_compiler(file_name, link_libraries, link_libraries_sources, circuit_config);

        match result {
            Result::Err(report) => {
                // println!("{}", Colour::Red.paint("previous errors were found"));
                let report_string = format!("[{}]", report.join(","));
                let compilation_result = CompilationResult { program: Vec::new(), input_signals: HashMap::new(), report: report_string };

                return compilation_result;
            },
            Result::Ok((wasm_contents, templates_name_values)) => {
                // println!("{}", Colour::Green.paint("Everything went okay, circom safe"));
                let compilation_result = CompilationResult { program: wasm_contents, input_signals: templates_name_values, report: "".to_string() };

                return compilation_result;
            }
        }
    } else {
        CompilationResult { program: Vec::new(), input_signals: HashMap::new(), report: "Invalid config provided".to_string() }
    }
}

#[wasm_bindgen]
pub fn parse(file_name: String, sources: JsValue) -> String {
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
    let result = start_parser(file_name, link_libraries, link_libraries_sources);

    match result {
        Result::Err(report) => {
            // eprintln!("{}", Colour::Red.paint("previous errors were found"));
            let report_string = format!("[{}]", report.join(","));
            return report_string;
        },
        Result::Ok(warns) => {
            // println!("{}", Colour::Green.paint("Everything went okay, circom safe"));
            let report_string = format!("[{}]", warns.join(","));
            return report_string;
        }
    }
}

#[wasm_bindgen]
pub fn generate_r1cs(file_name: String, sources: JsValue, config: JsValue) -> Vec<u8> {
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
        }
        let result = start_r1cs(file_name, link_libraries, link_libraries_sources, circuit_config);

        match result {
            Result::Err(report) => {
                // println!("{}", Colour::Red.paint("previous errors were found"));
                return vec![];
            },
            Result::Ok(r1cs) => {
                // println!("{}", Colour::Green.paint("Everything went okay, circom safe"));
                return r1cs;
            }
        }
    } else {
        return vec![0];
    }
}

fn start_compiler(file_name: String, link_libraries: Vec<String>, link_libraries_sources: Vec<String>, config: CircuitConfig) -> Result<(Vec<u8>, HashMap<String, Vec<String>>), Vec<String>> {
    use execution_wasm::ExecutionConfig;
    let (mut program_archive, warnings) = parser_wasm::parse_project(file_name, link_libraries, link_libraries_sources)?;
    let parse_report = type_analysis_wasm::analyse_project(&mut program_archive)?;
    let execution_config = ExecutionConfig {
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
        prime: config.prime,
    };
    let (_, circuit) = execution_wasm::execute_project(program_archive.clone(), execution_config)?;
    let compilation_config = CompilerConfig {
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
    };
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

                template_names_values.insert(template_data.0.to_string(), input_signals.keys().cloned().collect());
            }
            
            return Result::Ok((wasm_contents, template_names_values));
        }
    }
}

fn start_parser(file_name: String, link_libraries: Vec<String>, link_libraries_sources: Vec<String>) -> Result<Vec<String>, Vec<String>> {
    let (mut program_archive, parse_warnings) = parser_wasm::parse_project(file_name, link_libraries, link_libraries_sources)?;
    let mut analysis_warnings = type_analysis_wasm::analyse_project(&mut program_archive)?;

    for warns in parse_warnings.iter() {
        analysis_warnings.push(warns.to_string());
    }
    Result::Ok(analysis_warnings)
}

fn start_r1cs(file_name: String, link_libraries: Vec<String>, link_libraries_sources: Vec<String>, config: CircuitConfig) -> Result<Vec<u8>, Vec<String>> {
    use execution_wasm::ExecutionConfig;
    let (mut program_archive, warnings) = parser_wasm::parse_project(file_name, link_libraries, link_libraries_sources)?;
    let parse_report = type_analysis_wasm::analyse_project(&mut program_archive)?;
    let execution_config = ExecutionConfig {
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
        prime: config.prime,
    };
    let (exporter, circuit) = execution_wasm::execute_project(program_archive.clone(), execution_config)?;
    let r1cs_details = generate_output_r1cs(exporter.as_ref(), program_archive.custom_gates)?;

    return Result::Ok(r1cs_details);
}
