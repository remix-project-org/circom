use codespan_reporting::diagnostic::{ LabelStyle, Severity };
use program_structure::error_definition::Report;

pub fn print_reports(reports: &[Report]) -> Vec<String> {
    let mut json_report: Vec<String> = Vec::new();

    for report in reports.iter() {
        let diagnostic_report = report.to_diagnostic();
        let notes: Vec<String> = diagnostic_report.notes.into_iter().map(|note| format!(r#""{}""#, note.replace("\"", ""))).collect();
        let notes = notes.join(",");
        let mut labels: Vec<String> = Vec::new();

        for label in diagnostic_report.labels {
            let style = if label.style == LabelStyle::Primary {
                "Primary"
            } else if label.style == LabelStyle::Secondary {
                "Secondary"
            } else {
                "Unknown"
            };
            let range = format!(r#"{{ "start": "{}", "end": "{}" }}"#, label.range.start, label.range.end);
            let message = label.message.replace("\"", "");

            labels.push(format!(r#"{{ "style": "{}", "file_id": "{}", "range": {}, "message": "{}" }}"#, style, label.file_id, range, message));
        }
        
        let labels = labels.join(",");
        let severity = if diagnostic_report.severity == Severity::Bug {
            "Bug".to_string()
        } else if diagnostic_report.severity == Severity::Error {
            "Error".to_string()
        } else if diagnostic_report.severity == Severity::Help {
            "Help".to_string()
        } else if diagnostic_report.severity == Severity::Note {
            "Note".to_string()
        } else if diagnostic_report.severity == Severity::Warning {
            "Warning".to_string()
        } else {
            "Unknown".to_string()
        };
        let message = diagnostic_report.message.replace("\"", "");

        json_report.push(format!(r#"{{ "type": "{}", "message": "{}", "labels": [{}], "notes": [{}] }}"#, severity, message, labels, notes));
    }
    json_report
}