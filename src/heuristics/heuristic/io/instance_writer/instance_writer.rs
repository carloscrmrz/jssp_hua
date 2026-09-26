use std::fs;
use std::path::Path;

use chrono::{DateTime, Local};

pub struct InstanceWriter {
    path_to_output: Option<String>,
}

impl InstanceWriter {
    pub fn new(path_to_output: Option<String>) -> InstanceWriter {
        InstanceWriter { path_to_output }
    }

    pub fn output_file_name(index: usize, at: &DateTime<Local>) -> String {
        format!(
            "./results/output-{:04}-{}{:02}.out",
            index,
            at.format("%d%m-%Y-%H%M-%S"),
            at.timestamp_subsec_millis() / 10
        )
    }

    pub fn solutions_file_name(path_to_output: &str) -> String {
        Path::new(path_to_output)
            .with_extension("sol")
            .to_string_lossy()
            .into_owned()
    }

    pub fn format_report(&self) -> Result<String, String> {
        todo!("format the JSSP report")
    }
    pub fn format_solutions(&self) -> String {
        todo!("format the accepted solutions")
    }

    pub fn write_instance(&self) -> Result<(), String> {
        let report = self.format_report()?;
        print!("{report}");

        match self.path_to_output.as_ref() {
            Some(path_to_output) => {
                if let Some(parent) = Path::new(path_to_output).parent() {
                    fs::create_dir_all(parent).map_err(|err| err.to_string())?;
                }
                fs::write(path_to_output, report).map_err(|err| err.to_string())?;
                fs::write(
                    Self::solutions_file_name(path_to_output),
                    self.format_solutions(),
                )
                .map_err(|err| err.to_string())
            }
            None => Ok(()),
        }
    }
}
