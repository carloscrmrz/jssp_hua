use clap::Parser;
use crate::heuristics::heuristic::io::args::Args;
use crate::heuristics::heuristic::io::instance_reader::InstanceReader;

mod heuristics;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let instance_reader = InstanceReader::new(&args);

    instance_reader.get_parsed_instance();

    Ok(())
}
