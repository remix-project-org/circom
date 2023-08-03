use parser::parser_logic::{preprocess, produce_generic_report};
use parser::syntax_sugar_remover::apply_syntactic_sugar;
use program_structure::error_definition::{Report, ReportCollection};
use program_structure::file_definition::{FileLibrary, FileID};
use program_structure::program_archive::ProgramArchive;
use program_structure::error_code::ReportCode;
use parser::{check_number_version, parse_number_version, check_custom_gates_version, produce_report_with_main_components, lang};
use program_structure::ast::{produce_report, AST};
use crate::VERSION;
use crate::error_reporting_wasm::print_reports;
use crate::include_logic_wasm::{FileStack, IncludesGraph};

pub fn parse_project(file: String, link_libraries: Vec<String>, link_libraries_sources: Vec<String>) -> Result<(ProgramArchive, Vec<String>), Vec<String>> {
    let result_program_archive = run_parser_wasm(file, VERSION, link_libraries, link_libraries_sources);
    match result_program_archive {
        Result::Err((_, report_collection)) => {
            let report = print_reports(&report_collection);
            Result::Err(report)
        }
        Result::Ok((program_archive, warnings)) => {
            let warnings = print_reports(&warnings);
            Result::Ok((program_archive, warnings))
        }
    }
}

fn run_parser_wasm(
    file: String,
    version: &str,
    link_libraries: Vec<String>,
    link_libraries_sources: Vec<String>
) -> Result<(ProgramArchive, ReportCollection), (FileLibrary, ReportCollection)> {
    let mut file_library = FileLibrary::new();
    let mut definitions = Vec::new();
    let mut main_components = Vec::new();
    let mut file_stack = FileStack::new(file);
    let mut includes_graph = IncludesGraph::new();
    let mut warnings = Vec::new();
    let link_libraries2 = link_libraries.clone();
    let link_libraries_sources2 = link_libraries_sources.clone();

    while let Some(location) = FileStack::take_next(&mut file_stack) {
        if location < link_libraries2.len() {
            let file_name = (link_libraries2[location]).clone();
            let file_source = (link_libraries_sources2[location]).clone();
            let file_id = file_library.add_file(file_name.clone(), file_source.clone());
            let program = parse_file_wasm(&file_source, file_id).map_err(|e| (file_library.clone(), e))?;

            if let Some(main) = program.main_component {
                main_components.push((file_id, main, program.custom_gates));
            }
            includes_graph.add_node(file_name.clone(), program.custom_gates, program.custom_gates_declared);
            let includes = program.includes;
            definitions.push((file_id, program.definitions));
            for include in includes {
                let path_include =
                FileStack::add_include(&mut file_stack, include.clone(), &link_libraries2.clone())
                    .map_err(|e| (file_library.clone(), vec![e]))?;
                includes_graph.add_edge(path_include, &link_libraries2.clone()).map_err(|e| (file_library.clone(), vec![e]))?;
            }
            warnings.append(
                &mut check_number_version(
                    file_name.clone(),
                    program.compiler_version,
                    parse_number_version(version),
                )
                .map_err(|e| (file_library.clone(), vec![e]))?,
            );
            if program.custom_gates {
                check_custom_gates_version(
                    file_name.clone(),
                    program.compiler_version,
                    parse_number_version(version),
                )
                .map_err(|e| (file_library.clone(), vec![e]))?
            }
        } else {
            return Result::Err((file_library.clone(), Vec::new()));
        }
    }
    if main_components.len() == 0 {
        let report = produce_report(ReportCode::NoMainFoundInProject,0..0, 0);
        warnings.push(report);
        Err((file_library.clone(), warnings))
    } else if main_components.len() > 1 {
        let report = produce_report_with_main_components(main_components);
        warnings.push(report);
        Err((file_library.clone(), warnings))
    } else {
        let mut errors: ReportCollection = includes_graph.get_problematic_paths().iter().map(|path|
            Report::error(
                format!(
                    "Missing custom templates pragma in file {} because of the following chain of includes {}",
                    path[path.len() - 1],
                    IncludesGraph::display_path(path)
                ),
                ReportCode::CustomGatesPragmaError
            )
        ).collect();
        if errors.len() > 0 {
            warnings.append(& mut errors);
            Err((file_library.clone(), warnings))
        } else {
            let (main_id, main_component, custom_gates) = main_components.pop().unwrap();
            let result_program_archive = ProgramArchive::new(
                file_library,
                main_id,
                main_component,
                definitions,
                custom_gates,
            );
            match result_program_archive {
                Err((lib, mut rep)) => {
                    warnings.append(&mut rep);
                    Err((lib, warnings))
                }
                Ok(mut program_archive) => {
                    let lib: FileLibrary = program_archive.get_file_library().clone();
                    let program_archive_result = apply_syntactic_sugar( &mut program_archive);
                    match program_archive_result {
                        Result::Err(v) => {
                            warnings.push(v);
                            Result::Err((lib, warnings))},
                        Result::Ok(_) => Ok((program_archive, warnings)),
                    }
                }
            }
        }
    }
}

fn parse_file_wasm(src: &str, file_id: FileID) -> Result<AST, ReportCollection> {
    use lalrpop_util::ParseError::*;
    let mut errors = Vec::new();
    let preprocess = preprocess(src, file_id)?;

    let ast = lang::ParseAstParser::new()
        .parse(file_id, &mut errors, &preprocess)
        // TODO: is this always fatal?
        .map_err(|parse_error| match parse_error {
            InvalidToken { location } => 
                produce_generic_report(
                "InvalidToken: Circom parser encountered a token (or EOF) it did not expect".to_string(),
                 location..location, file_id
                ),
            UnrecognizedToken { ref token, .. } => 
                produce_generic_report(
                "UnrecognizedToken: Circom parser encountered a token it did not expect".to_string(),
                 token.0..token.2, file_id
                ),
            ExtraToken { ref token } => produce_generic_report(
                "ExtraToken: Cirom parser encountered additional, unexpected tokens".to_string(),
                 token.0..token.2, file_id
                ),
            UnrecognizedEOF { ref location, expected } => produce_generic_report(
                "UnrecognizedEOF: Circom parser encountered an end of file (EOF) it did not expect".to_string(),
                0..location.clone(), file_id
            ),
            User { error } => produce_generic_report(
                "User: Circom parser encountered an unexpected error".to_string(),
                 0..0, file_id
                )
        })
        .map_err(|e| vec![e])?;

    if !errors.is_empty() {
        return Err(errors.into_iter().collect());
    }

    Ok(ast)
}