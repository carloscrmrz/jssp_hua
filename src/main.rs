use clap::Parser;
use rand::SeedableRng;
use rand::rngs::StdRng;
use crate::heuristics::heuristic::hua::hua;
use crate::heuristics::heuristic::hua::hua::Parameters;
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

    let mut parameters = match Parameters::variant(args.variant) {
        Ok(parameters) => parameters,
        Err(message) => std::process::exit(1)
    };
    if let Some(iterations) = args.iterations {
        parameters.iterations = iterations;
    }
    if let Some(k) = args.k {
        parameters.k = k;
    }
    if let Some(rimpk) = args.rimpk {
        parameters.rimpk = rimpk;
    }
    if let Some(radius_scale) = args.radius_scale {
        parameters.radius_scale = radius_scale;
    }
    if let Some(adventurer_swaps) = args.adventurer_swaps {
        parameters.adventurer_swaps = adventurer_swaps;
    }
    if let Some(local_rounds) = args.local_rounds {
        parameters.local_rounds = local_rounds;
    }

    let outcome = hua::run(&problem, &parameters, &mut rng);
    let solution = outcome.best;
    let iterations_run = outcome.history.len();

    let solution = Solution::random(&problem, &mut rng);

    println!(
        "jobs: {}, machines: {}, seed: {}, HUS variant: {}",
        problem.num_jobs(),
        problem.num_machines(),
        seed,
        args.variant
    );
    println!("computed lower bound: {}", problem.lower_bound());
    println!("computed upper bound: {}", problem.upper_bound());
    println!("makespan: {}", solution.makespan());
    // for (machine_id, sequence) in solution.machine_sequences().iter().enumerate() {
    //     // println!("M{}: {:?}", machine_id, sequence);
    // }
    Ok(())
}
