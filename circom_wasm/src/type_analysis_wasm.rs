use program_structure::program_archive::ProgramArchive;
use type_analysis::check_types::check_types;

use crate::error_reporting_wasm::print_reports;

pub fn analyse_project(program_archive: &mut ProgramArchive) -> Result<Vec<String>, Vec<String>> {
    let analysis_result = check_types(program_archive);
    match analysis_result {
        Err(errs) => {
            let json_report = print_reports(&errs);
            Err(json_report)
        }
        Ok(warns) => {
            let json_report = print_reports(&warns);
            Ok(json_report)
        }
    }
}