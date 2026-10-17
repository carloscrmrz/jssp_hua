use super::super::args::Args;
use crate::heuristics::heuristic::models::problem::Problem;

use std::fs;

pub struct InstanceReader {
    raw_instance: Option<String>,
    path_to_instance: Option<String>,
}
impl InstanceReader {
    pub fn new(args: &Args) -> InstanceReader {
        if args.instance.is_some() {
            return InstanceReader {
                raw_instance: Some(args.instance.as_ref().unwrap().to_string()),
                path_to_instance: None,
            };
        } else if args.instance_path.is_some() {
            return InstanceReader {
                path_to_instance: Some(args.instance_path.as_ref().unwrap().to_string()),
                raw_instance: None,
            };
        }
        panic!("No instance specified");
    }


    pub fn get_parsed_instance(&self) -> Result<Problem, String> {
        match (self.path_to_instance.as_ref(), self.raw_instance.as_ref()) {
            (Some(_), Some(_)) => Err(String::from(
                "Give either an instance or a path to one, not both",
            )),
            (Some(path_to_instance), None) => {
                let raw_str = fs::read_to_string(path_to_instance)
                    .map_err(|e| format!("Failed to read '{}': {}", path_to_instance, e))?;
                InstanceReader::parse_instance(&raw_str)
            }
            (None, Some(raw_instance)) => InstanceReader::parse_instance(raw_instance),
            (None, None) => Err(String::from("No instance specified")),
        }
    }

    fn parse_numbers(line: &str) -> Result<Vec<usize>, String> {
        line.split_whitespace()
            .map(|s| s.parse::<usize>().map_err(|e| format!("Failed to parse '{}': {}", s, e)))
            .collect()
    }

    fn get_pairs(lines: &[&str]) -> Result<Vec<Vec<(usize, usize)>>, String> {
        let mut result: Vec<Vec<(usize, usize)>> = vec![];
        for (job_id, line) in lines.iter().enumerate() {
            let numbers = Self::parse_numbers(line)?;
            if numbers.len() % 2 != 0 {
                return Err(format!(
                    "Job {} has an odd number of values; expected 'machine timespan' pairs",
                    job_id
                ));
            }

            let pairs: Vec<(usize, usize)> = numbers
                .chunks_exact(2)
                .map(|chunk| (chunk[0], chunk[1]))
                .collect();

            result.push(pairs);
        }
        Ok(result)
    }

    fn parse_instance(raw: &str) -> Result<Problem, String> {
        let lines: Vec<&str> = raw.lines().filter(|line| !line.trim().is_empty()).collect();

        let (jobs, machines, bounds) = match Self::parse_numbers(lines.first().ok_or("Empty instance")?)?[..] {
            [n, m] => (n, m, None),
            [n, m, ub, lb] => (n, m, Some((ub, lb))),
            _ => {
                return Err(String::from(
                    "Invalid first line, expected 'jobs machines' or 'jobs machines upper_bound lower_bound'",
                ));
            }
        };

        let pairs = Self::get_pairs(&lines[1..])?;

        if pairs.len() != jobs {
            return Err(format!("Invalid number of jobs: {}, expected {}", pairs.len(), jobs));
        }

        let problem = Problem::new(&pairs, machines)?;
        match bounds {
            Some((upper_bound, lower_bound)) => problem.with_known_bounds(upper_bound, lower_bound),
            None => Ok(problem),
        }
    }
}
