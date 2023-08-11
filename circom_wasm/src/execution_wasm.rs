use std::rc::Rc;

use compiler::hir::very_concrete_program::VCP;
use constraint_generation::{BuildConfig, ConstraintWriter, FlagsExecution, instantiation, export, sync_dag_and_vcp};
use constraint_list::ConstraintList;
use constraint_writers::debug_writer::DebugWriter;
use dag::DAG;
use program_structure::program_archive::ProgramArchive;

use crate::error_reporting_wasm::print_reports;

pub struct ExecutionConfig {
    pub r1cs: String,
    pub sym: String,
    pub json_constraints: String,
    pub no_rounds: usize,
    pub flag_s: bool,
    pub flag_f: bool,
    pub flag_p: bool,
    pub flag_old_heuristics:bool,
    pub flag_verbose: bool,
    pub inspect_constraints_flag: bool,
    pub sym_flag: bool,
    pub r1cs_flag: bool,
    pub json_substitution_flag: bool,
    pub json_constraint_flag: bool,
    pub prime: String,
}

pub fn execute_project(
    program_archive: ProgramArchive,
    config: ExecutionConfig,
) -> Result<VCP, Vec<String>> {
    let debug = DebugWriter::new(config.json_constraints).unwrap();
    let build_config = BuildConfig {
        no_rounds: config.no_rounds,
        flag_json_sub: config.json_substitution_flag,
        flag_s: config.flag_s,
        flag_f: config.flag_f,
        flag_p: config.flag_p,
        flag_verbose: config.flag_verbose,
        inspect_constraints: config.inspect_constraints_flag,
        flag_old_heuristics: config.flag_old_heuristics,
        prime : config.prime,
    };
    let custom_gates = program_archive.custom_gates;
    let (exporter, vcp) = build_circuit_wasm(program_archive, build_config)?;
    // if config.r1cs_flag {
    //     generate_output_r1cs(&config.r1cs, exporter.as_ref(), custom_gates)?;
    // }
    // if config.sym_flag {
    //     generate_output_sym(&config.sym, exporter.as_ref())?;
    // }
    // if config.json_constraint_flag {
    //     generate_json_constraints(&debug, exporter.as_ref())?;
    // }
    Result::Ok(vcp)
}

fn build_circuit_wasm(program: ProgramArchive, config: BuildConfig) -> Result<(ConstraintWriter, VCP), Vec<String>> {
    // TODO: Return warnings to be displayed in the browser
    let flags = FlagsExecution {
        verbose: config.flag_verbose,
        inspect: config.inspect_constraints,
    };
    let instance = instantiation(&program, flags, &config.prime).map_err(|r| {
        print_reports(&r);
    });

    match instance {
        Result::Err(()) => {
            Result::Err(vec![String::from("Build Circuit Instantiation Failed")])
        },
        Result::Ok((exe, warnings)) => {
            print_reports(&warnings);
            let export_values = export(exe, program, flags).map_err(|r| {
                print_reports(&r);
            });

            match export_values {
                Result::Err(()) => {
                    Result::Err(vec![String::from("Exporting values from build circuit failed")])
                },
                Result::Ok((mut dag, mut vcp, warnings)) => {
                    if config.inspect_constraints {
                        print_reports(&warnings);
                    }
                    if config.flag_f {
                        sync_dag_and_vcp(&mut vcp, &mut dag);
                        Result::Ok((Box::new(dag), vcp))
                    } else {
                        let list = simplification_process_wasm(&mut vcp, dag, &config);
                        Result::Ok((Box::new(list), vcp))
                    }
                }
            }
        }
    }
}

fn simplification_process_wasm(vcp: &mut VCP, dag: DAG, config: &BuildConfig) -> ConstraintList {
    use dag::SimplificationFlags;
    let flags = SimplificationFlags {
        flag_s: config.flag_s,
        parallel_flag: config.flag_p,
        port_substitution: config.flag_json_sub,
        no_rounds: config.no_rounds,
        flag_old_heuristics: config.flag_old_heuristics,
        prime : config.prime.clone(),
    };
    let list: ConstraintList = crate::constraints_wasm::map(dag, flags);
    VCP::add_witness_list(vcp, Rc::new(list.get_witness_as_vec()));
    list
}

// fn generate_output_r1cs(file: &str, exporter: &dyn ConstraintExporter, custom_gates: bool) -> Result<(), ()> {
//     if let Result::Ok(()) = exporter.r1cs(file, custom_gates) {
//         println!("{} {}", Colour::Green.paint("Written successfully:"), file);
//         Result::Ok(())
//     } else {
//         eprintln!("{}", Colour::Red.paint("Could not write the output in the given path"));
//         Result::Err(())
//     }
// }

// fn generate_output_sym(file: &str, exporter: &dyn ConstraintExporter) -> Result<(), ()> {
//     if let Result::Ok(()) = exporter.sym(file) {
//         println!("{} {}", Colour::Green.paint("Written successfully:"), file);
//         Result::Ok(())
//     } else {
//         eprintln!("{}", Colour::Red.paint("Could not write the output in the given path"));
//         Result::Err(())
//     }
// }

// fn generate_json_constraints(
//     debug: &DebugWriter,
//     exporter: &dyn ConstraintExporter,
// ) -> Result<(), ()> {
//     if let Ok(()) = exporter.json_constraints(&debug) {
//         println!("{} {}", Colour::Green.paint("Constraints written in:"), debug.json_constraints);
//         Result::Ok(())
//     } else {
//         eprintln!("{}", Colour::Red.paint("Could not write the output in the given path"));
//         Result::Err(())
//     }
// }