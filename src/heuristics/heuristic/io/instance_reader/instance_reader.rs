use super::super::args::Args;

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

    pub fn get_parsed_instance(&self) -> Vec<usize> {
        match (self.path_to_instance.as_ref(), self.raw_instance.as_ref()) {
            (Some(_path_to_instance), Some(_raw_instance)) => {
                vec![]
            } // temp, while I create a better error system
            (Some(path_to_instance), None) => {
                let raw_str = fs::read_to_string(path_to_instance);
                if raw_str.is_err() {
                    return vec![];
                }
                let parsed_result = InstanceReader::parse_instance(&raw_str.unwrap());
                return match parsed_result {
                    Ok(parsed_result) => parsed_result,
                    _ => {
                        vec![]
                    }
                };
            }
            (None, Some(raw_instance)) => {
                let parsed_result = InstanceReader::parse_instance(&raw_instance);
                return match parsed_result {
                    Ok(parsed_result) => parsed_result,
                    _ => {
                        vec![]
                    }
                };
            }
            (None, None) => {
                vec![]
            } // temp, while I create a better error system
        }
    }

    fn parse_instance(_arg: &str) -> Result<Vec<usize>, String> {
        Ok(vec![]) // TODO – implement this l8r
    }
}
