#[derive(Clone)]
pub struct LogWasm {
    pub no_linear: usize,
    pub no_non_linear: usize,
    pub no_labels: usize,
    pub no_wires: usize,
    pub no_public_inputs: usize,
    pub no_private_inputs: usize,
    pub no_public_outputs: usize,
    pub no_private_outputs: usize,
    pub is_successful: bool
}

impl LogWasm {
    pub fn new() -> LogWasm {
        LogWasm {
            no_linear: 0,
            no_non_linear: 0,
            no_public_inputs: 0,
            no_private_inputs: 0,
            no_public_outputs: 0,
            no_private_outputs: 0,
            no_wires: 0,
            no_labels: 0,
            is_successful: false
        }
    }

    pub fn print_array(logs: &LogWasm) -> Vec<String> {
        let logs = if logs.is_successful {
            let mut output = Vec::new();

            if logs.no_non_linear > 0 { output.push(format!("non-linear constraints: {}", logs.no_non_linear)) }
            if logs.no_linear > 0 { output.push(format!("linear constraints: {}", logs.no_linear)) }
            if logs.no_public_inputs > 0 { output.push(format!("public inputs: {}", logs.no_public_inputs)) }
            if logs.no_public_outputs > 0 { output.push(format!("public outputs: {}", logs.no_public_outputs)) }
            if logs.no_private_inputs > 0 { output.push(format!("private inputs: {}", logs.no_private_inputs)) }
            if logs.no_private_outputs > 0 { output.push(format!("private outputs: {}", logs.no_private_outputs)) }
            if logs.no_wires > 0 { output.push(format!("wires: {}", logs.no_wires)) }
            if logs.no_labels > 0 { output.push(format!("labels: {}", logs.no_labels)) }

            output
        }
        else {
            vec!["previous errors were found".to_string()]
        };

        logs
    }
}
