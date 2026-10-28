use std::f64::consts::PI;
use rand::rngs::StdRng;
use rand::{Rng, RngExt, SeedableRng};
use rayon::prelude::*;

use crate::heuristics::heuristic::models::solution::Solution;
use crate::heuristics::heuristic::models::solution::solution::Walker;
use super::super::models::problem::Problem;


#[derive(Debug, Clone, PartialEq)]
pub struct Parameters {
    pub iterations: usize,
    pub k: f64,
    pub rimpk: f64,
    pub populations: Vec<usize>,
    pub population_indices: Vec<i64>,
    pub radius_scale: f64,
    pub adventurer_swaps: usize,
    pub local_rounds: usize,
}

impl Parameters {
    pub fn variant(number: u8) -> Result<Parameters, String> {
        // Variants are defined in Ghasemian et al. (2020).
        let (populations, population_indices) = match number {
            1 => (vec![10, 9, 8, 7, 6, 5, 4, 3, 2, 1], vec![0; 10]),
            2 => (vec![1; 10], vec![1, 1, 1, 0, 0, 0, 0, 0, 0, 0]),
            3 => (vec![50, 40, 30, 2, 1, 1, 1, 1, 1, 1], vec![-1, -1, -1, 0, 0, 0, 0, 0, 0, 0]),
            4 => (vec![1; 10], vec![0, 0, 0, 0, 0, 0, 0, 1, 1, 1]),
            5 => (vec![1; 10], vec![0, 0, 0, 1, 1, 1, 1, 0, 0, 0]),
            _ => return Err(format!("invalid HUA variant: {}, expected 1-5", number)),
        };
        Ok(Parameters {  // Default params, can be overriden using the terminal.
            iterations: 500,
            k: 2.0,
            rimpk: 1.0,
            populations,
            population_indices,
            radius_scale: 0.05,
            adventurer_swaps: 2,
            local_rounds: 2,
        })
    }
}


pub struct Outcome {
    pub best: Solution,
    pub history: Vec<usize>,
    pub cancelled: bool,
}

pub fn run<R: Rng + ?Sized>(
    problem: &Problem,
    parameters: &Parameters,
    rng: &mut R,
) -> Outcome {
    let num_cities = parameters.populations.len();
    let target = problem
        .lower_bound()
        .max(problem.known_lower_bound().unwrap_or(0));

    let mut population = parameters.populations.clone();
    let mut adventurers: Vec<Solution> = (0..num_cities)
        .map(|_| Solution::random(&problem, rng))
        .collect();

    // First set of settlements
    let mut cities: Vec<Solution> = adventurers.clone();
    sort_by_makespan(&mut cities);

    let mut history: Vec<usize> = vec![cities[0].makespan()];
    let mut radii = city_radii(problem, parameters, &cities, &population, 1, rng);

    for iteration in 1..=parameters.iterations {
        if cities[0].makespan() <= target {
            break;
        }

        let seeds: Vec<u64> = (0..adventurers.len()).map(|_| rng.random()).collect();
        adventurers = adventurers
            .par_iter()
            .zip(seeds)
            .map(|(adventurer, seed)| {
                adventure(problem, parameters, adventurer, &cities[0], &mut StdRng::seed_from_u64(seed))
            })
            .collect();

        // Adventurers inside a city migrate into it
        sort_by_makespan(&mut adventurers);
        let mut founders: Vec<Solution> = vec![];
        for adventurer in &adventurers {
            let home = cities
                .iter()
                .zip(&radii)
                .position(|(city, &radius)| adventurer.within(city, radius));
            match home {
                Some(rank) => {
                    population[rank] = migrate(population[rank], parameters.population_indices[rank]);
                }
                None if !founders.contains(adventurer) => founders.push(adventurer.clone()),
                None => {}
            }
        }

        cities.extend(founders);
        sort_by_makespan(&mut cities);
        cities.truncate(num_cities);

        // citizens inside a city explore, each citizen is independent so they do their walk in parallel.
        for _ in 0..parameters.local_rounds {
            radii = city_radii(problem, parameters, &cities, &population, iteration, rng);
            let centers: Vec<Walker> = cities.par_iter().map(|city| Walker::new(problem,city)).collect();
            let walks: Vec<(usize, u64)> = population
                .iter()
                .enumerate()
                .flat_map(|(rank,&population)| std::iter::repeat_n(rank, population))
                .map(|rank| (rank, rng.random()))
            .collect();

            let citizens: Vec<(usize, Solution)> = walks
                .into_par_iter()
                .map(|(rank, seed)| {
                    let mut rng = StdRng::seed_from_u64(seed);
                    (rank, citizen(&centers[rank], radii[rank], &mut rng))
                })
                .collect();

            for (rank, citizen) in citizens {
                if citizen.makespan() < cities[rank].makespan() {
                    cities[rank] = citizen;
                }
            }

            sort_by_makespan(&mut cities);
        }

        history.push(cities[0].makespan());
    }

    Outcome {
        best: cities.swap_remove(0),
        history,
        cancelled: false
    }
}

fn sort_by_makespan(solutions: &mut [Solution]) {
    solutions.sort_by_key(Solution::makespan);
}

fn migrate(pop: usize, index: i64) -> usize { (pop as i64 + index).max(1) as usize }

fn adventure<R: Rng + ?Sized>(
    problem: &Problem,
    parameters: &Parameters,
    adventurer: &Solution,
    capital: &Solution,
    rng: &mut R
) -> Solution {
    let r = rng.random::<f64>();
    let towards_capital = (1.0 - r) / (parameters.k * r + 1.0 - r);

    let mut walker = Walker::new(problem, adventurer);
    for _ in 0..parameters.adventurer_swaps {
        let guided = rng.random::<f64>() < towards_capital && walker.guided_swap(capital, rng);
        if !guided {
            walker.random_swap(rng);
        }
    }

    walker.solution()
}

fn citizen<R: Rng + ?Sized>(center: &Walker, radius: usize, rng: &mut R) -> Solution {
    let steps = rng.random_range(1..=radius);
    center.clone().critical_walk(steps, rng)
}

fn city_radii<R: Rng + ?Sized>(
    problem: &Problem,
    parameters: &Parameters,
    cities: &[Solution],
    populations: &[usize],
    iteration: usize,
    rng: &mut R,
) -> Vec<usize> {
    let s = 500.0 * PI / (2.0 * parameters.iterations as f64);
    let t = iteration as f64;
    let wave = 50.0 * ((t + s).cos() + 1.0) / t.powf(parameters.rimpk);

    let capital = cities[0].makespan() as f64;
    let max_distance =
        (problem.num_machines() * problem.num_jobs() * problem.num_jobs().saturating_sub(1) / 2) as f64;

    cities
        .iter()
        .zip(populations)
        .map(|(city, &population)| {
            let diff = ((capital - city.makespan() as f64) / capital).abs().atan() + 0.5;
            let radius = wave * diff * rng.random::<f64>() / population as f64;
            ((radius / 100.0 * parameters.radius_scale * max_distance).round() as usize).max(1)
        })
        .collect()
}