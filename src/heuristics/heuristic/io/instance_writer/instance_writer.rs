use std::fs;
use std::path::Path;

use chrono::{DateTime, Local};

use crate::heuristics::heuristic::models::problem::Problem;
use crate::heuristics::heuristic::models::solution::Solution;


pub struct Report<'a> {
    pub problem: &'a Problem,
    pub solution: &'a Solution,
    pub seed: u64,
    pub variant: u8,
    pub iterations: usize,
    pub iterations_run: usize,
    pub initial_makespan: usize,
    pub cancelled: bool,
}

pub struct InstanceWriter {
    path_to_output: Option<String>,
}

impl InstanceWriter {
    pub fn new(path_to_output: Option<String>) -> InstanceWriter {
        InstanceWriter { path_to_output }
    }

    pub fn format_solution(problem: &Problem, solution: &Solution) -> String {
        solution
            .machine_sequences()
            .iter()
            .enumerate()
            .map(|(machine_id, sequence)| {
                let operations: Vec<String> = sequence
                    .iter()
                    .map(|&job_id| {
                        let position = problem.position_on(job_id, machine_id);
                        let timespan = problem.job(job_id).operations()[position].timespan();
                        format!("[{}, {}, {}]", job_id, position, timespan)
                    })
                    .collect();
                format!("[{}]\n", operations.join(", "))
            })
            .collect()
    }

    fn format_human(report: &Report) -> String {
        let problem = report.problem;
        let mut text = format!(
            "jobs: {}, machines: {}, seed: {}, variant: HUS{}\n",
            problem.num_jobs(),
            problem.num_machines(),
            report.seed,
            report.variant
        );
        text += &format!("computed lower bound: {}\n", problem.lower_bound());
        text += &format!("computed upper bound: {}\n", problem.upper_bound());
        text += &format!("makespan: {}\n", report.solution.makespan());
        // text + &Self::format_solution(problem, report.solution)
        text
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

    pub fn format_report(&self, report: &Report) -> String {
        Self::format_human(report)
    }
    pub fn format_solutions(&self, report: &Report) -> String {
        Self::format_solution(report.problem, report.solution)
    }

    pub fn write_instance(&self, report: &Report) -> Result<(), String> {
        let formatted = self.format_report(report);
        print!("{formatted}");

        match self.path_to_output.as_ref() {
            Some(path_to_output) => {
                if let Some(parent) = Path::new(path_to_output).parent() {
                    fs::create_dir_all(parent).map_err(|err| err.to_string())?;
                }
                fs::write(path_to_output, formatted).map_err(|err| err.to_string())?;
                fs::write(
                    Self::solutions_file_name(path_to_output),
                    self.format_solutions(report),
                )
                    .map_err(|err| err.to_string())
            }
            None => Ok(()),
        }
    }
}
