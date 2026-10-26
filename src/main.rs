use clap::Parser;
use rand::SeedableRng;
use rand::rngs::StdRng;

use crate::heuristics::heuristic::io::args::Args;
use crate::heuristics::heuristic::io::instance_reader::InstanceReader;
use crate::heuristics::heuristic::models::solution::Solution;

mod heuristics;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let instance_reader = InstanceReader::new(&args);
    let problem = instance_reader.get_parsed_instance()?;

    let seed = args.seed.unwrap_or_else(rand::random);
    let mut rng = StdRng::seed_from_u64(seed);

    let solution = Solution::random(&problem, &mut rng);

    println!(
        "jobs: {}, machines: {}, seed: {}",
        problem.num_jobs(),
        problem.num_machines(),
        seed
    );
    println!("computed lower bound: {}", problem.lower_bound());
    println!("computed upper bound: {}", problem.upper_bound());
    if let (Some(upper_bound), Some(lower_bound)) =
        (problem.known_upper_bound(), problem.known_lower_bound())
    {
        println!("known bounds: [{}, {}]", lower_bound, upper_bound);
    }
    println!("makespan: {}", solution.makespan());
    println!("cost: {}", solution.current_cost());
    for (machine_id, sequence) in solution.machine_sequences().iter().enumerate() {
        // println!("M{}: {:?}", machine_id, sequence);
    }
    Ok(())
}
