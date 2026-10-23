use super::InstanceReader;
use crate::heuristics::heuristic::io::args::Args;

fn args_with_raw_instance(raw: &str) -> Args {
    Args {
        instance: Some(raw.to_string()),
        instance_path: None,
        seed: None,
        threads: None,
    }
}

fn args_with_path(path: &str) -> Args {
    let mut args = args_with_raw_instance("");
    args.instance = None;
    args.instance_path = Some(path.to_string());
    args
}

#[test]
#[should_panic(expected = "No instance specified")]
fn new_panics_when_neither_instance_nor_path_is_given() {
    let mut args = args_with_raw_instance("");
    args.instance = None;
    InstanceReader::new(&args);
}

// TODO: parsing tests (raw instance, file path, malformed input) once the JSSP input is defined.
