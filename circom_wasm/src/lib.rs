mod compilation_wasm;
mod execution_wasm;
mod parser_wasm;
mod type_analysis_wasm;
mod include_logic_wasm;
mod error_reporting_wasm;
mod constraints_wasm;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsValue;

use crate::compilation_wasm::CompilerConfig;

struct CircuitConfig {
    prime: String
}

#[wasm_bindgen]
pub fn compile (file_name: String, sources: JsValue, config: JsValue) -> Vec<u8> {
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
                return vec![];
            },
            Result::Ok(wasm_contents) => {
                // println!("{}", Colour::Green.paint("Everything went okay, circom safe"));
                return wasm_contents;
            }
        }
    } else {
        return vec![0];
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
        Result::Ok(()) => {
            // println!("{}", Colour::Green.paint("Everything went okay, circom safe"));
            return String::from("Circuit parsing went okay, circom safe");
        }
    }
}

fn start_compiler(file_name: String, link_libraries: Vec<String>, link_libraries_sources: Vec<String>, config: CircuitConfig) -> Result<Vec<u8>, Vec<String>> {
    use execution_wasm::ExecutionConfig;
    let (mut program_archive, warnings) = parser_wasm::parse_project(file_name, link_libraries, link_libraries_sources)?;
    if warnings.len() > 0 {
        return Result::Err(warnings);
    }

    let parse_report = type_analysis_wasm::analyse_project(&mut program_archive)?;
    let config = ExecutionConfig {
        no_rounds: 0,
        flag_p: false,
        flag_s: false,
        flag_f: false,
        flag_old_heuristics: false,
        flag_verbose: false,
        inspect_constraints_flag: false,
        r1cs_flag: false,
        json_constraint_flag: false,
        json_substitution_flag: false,
        sym_flag: false,
        sym: "".to_string(),
        r1cs: "".to_string(),
        json_constraints: "".to_string(),
        prime: config.prime,        
    };
    let circuit = execution_wasm::execute_project(program_archive, config)?;
    let compilation_config = CompilerConfig {
        vcp: circuit,
        debug_output: false,
        // c_flag: user_input.c_flag(),
        c_flag: false,
        // wasm_flag: user_input.wasm_flag(),
        wasm_flag: true,
        // wat_flag: user_input.wat_flag(),
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
            return Err(report);
        }
        Result::Ok(wasm_contents) => {
            return Result::Ok(wasm_contents);
        }
    }
}

fn start_parser(file_name: String, link_libraries: Vec<String>, link_libraries_sources: Vec<String>) -> Result<(), Vec<String>> {
    let (mut program_archive, warnings) = parser_wasm::parse_project(file_name, link_libraries, link_libraries_sources)?;
    if warnings.len() > 0 {
        return Result::Err(warnings);
    }

    type_analysis_wasm::analyse_project(&mut program_archive)?;
    Result::Ok(())
}
