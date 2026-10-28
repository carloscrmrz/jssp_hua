use rand::{Rng, RngExt};
use rand::seq::SliceRandom;
use crate::heuristics::heuristic::models::schedule::schedule::Schedule;
use super::super::graph::DisjunctiveGraph;
use super::super::problem::Problem;

#[derive(Debug, Clone, PartialEq)]
pub struct Solution {
    machine_sequences: Vec<Vec<usize>>,
    makespan: usize,
}

impl Solution {
    pub fn from_machine_sequences(
        problem: &Problem,
        machine_sequences: Vec<Vec<usize>>,
    ) -> Result<Solution, String> {
        if machine_sequences.len() != problem.num_machines() {
            return Err(format!(
                "Got {} machine sequences, expected {}",
                machine_sequences.len(),
                problem.num_machines()
            ));
        }
        for (machine_id, sequence) in machine_sequences.iter().enumerate() {
            let mut seen = vec![false; problem.num_jobs()];
            for &job_id in sequence {
                if job_id >= problem.num_jobs() || seen[job_id] {
                    return Err(format!(
                        "Machine {} sequence {:?} is not a permutation of the jobs",
                        machine_id, sequence
                    ));
                }
                seen[job_id] = true;
            }
            if sequence.len() != problem.num_jobs() {
                return Err(format!(
                    "Machine {} sequence {:?} is not a permutation of the jobs",
                    machine_id, sequence
                ));
            }
        }

        let makespan = DisjunctiveGraph::new(problem, &machine_sequences).makespan()?;
        let upper_bound = problem.upper_bound();

        let cost = makespan as f64 / upper_bound as f64;
        Ok(Solution {
            machine_sequences,
            makespan,
        })
    }

    pub fn from_operation_sequence(
        problem: &Problem,
        operation_sequence: &[usize],
    ) -> Result<Solution, String> {
        if operation_sequence.len() != problem.num_operations() {
            return Err(format!(
                "Operation sequence has {} entries, expected {}",
                operation_sequence.len(),
                problem.num_operations()
            ));
        }

        let mut next_position = vec![0; problem.num_jobs()];
        let mut machine_sequences = vec![Vec::with_capacity(problem.num_jobs()); problem.num_machines()];
        for &job_id in operation_sequence {
            if job_id >= problem.num_jobs() {
                return Err(format!("Unknown job {} in operation sequence", job_id));
            }
            let Some(operation) = problem.job(job_id).operations().get(next_position[job_id]) else {
                return Err(format!(
                    "Job {} appears more than {} times in operation sequence",
                    job_id,
                    problem.num_machines()
                ));
            };
            machine_sequences[operation.machine_id()].push(job_id);
            next_position[job_id] += 1;
        }

        Solution::from_machine_sequences(problem, machine_sequences)
    }

    pub fn random<R: Rng + ?Sized>(problem: &Problem, rng: &mut R) -> Solution {
        let mut operation_sequence: Vec<usize> = (0..problem.num_jobs())
            .flat_map(|job_id| std::iter::repeat_n(job_id, problem.num_machines()))
            .collect();
        operation_sequence.shuffle(rng);

        Solution::from_operation_sequence(problem, &operation_sequence)
            .expect("operation-based encodings are always feasible")
    }

    pub fn machine_sequences(&self) -> &[Vec<usize>] {
        &self.machine_sequences
    }

    pub fn makespan(&self) -> usize {
        self.makespan
    }

    pub fn graph(&self, problem: &Problem) -> DisjunctiveGraph {
        DisjunctiveGraph::new(problem, &self.machine_sequences)
    }

    pub fn start_times(&self, problem: &Problem) -> Vec<Vec<usize>> {
        self.graph(problem)
            .start_times()
            .expect("a Solution is always feasible")
    }

    pub fn within(&self, other: &Solution, radius: usize) -> bool {
        let mut distance = 0;
        for (this, other) in self.machine_sequences.iter().zip(&other.machine_sequences) {
            distance += kendall_tau(this, other);
            if distance > radius {
                return false;
            }
        }
        true
    }
}

const MAX_SWAP_ATTEMPTS: usize = 100;

#[derive(Clone)]
pub struct Walker<'a> {
    machine_sequences: Vec<Vec<usize>>,
    schedule: Schedule<'a>,
}

impl<'a> Walker<'a> {
    pub fn new(problem: &'a Problem, solution: &Solution) -> Walker<'a> {
        Walker {
            machine_sequences: solution.machine_sequences.clone(),
            schedule: Schedule::new(problem, &solution.machine_sequences).expect("a solution is always feasible"),
        }
    }

    pub fn makespan(&self) -> usize {
        self.schedule.makespan()
    }

    pub fn solution(&self) -> Solution {
        Solution {
            machine_sequences: self.machine_sequences.clone(),
            makespan: self.schedule.makespan(),
        }
    }

    pub fn swap(&mut self, machine_id: usize, first: usize, second: usize) -> Result<(), String> {
        let sequence = &self.machine_sequences[machine_id];
        let operation_idx = |job_id: usize| {
            job_id * self.schedule.problem().num_machines() + self.schedule.problem().position_on(job_id, machine_id)
        };
        self.schedule
            .exchange(operation_idx(sequence[first]), operation_idx(sequence[second]))?;
        self.machine_sequences[machine_id].swap(first, second);
        Ok(())
    }

    pub fn guided_swap<R: Rng + ?Sized>(&mut self, guide: &Solution, rng: &mut R) -> bool {
        let guide_positions: Vec<Vec<usize>> = guide.machine_sequences
            .iter()
            .map(|seq| positions(seq))
            .collect();

        let disagreeing: Vec<usize> = self
            .machine_sequences
            .iter()
            .zip(&guide.machine_sequences)
            .map(|(this, other)| kendall_tau(this, other))
            .collect();

        let total: usize = disagreeing.iter().sum();
        if total == 0 {
            return false;
        }

        (0..MAX_SWAP_ATTEMPTS).any(|_| {
            let mut pick = rng.random_range(0..total);
            let machine_id = disagreeing
                .iter()
                .position(|&count| {
                    if pick < count {
                        return true;
                    }
                    pick -= count;
                    false
                })
                .expect("the pick is below the total number of disagreeing pairs");
            let (first, second) =
               nth_disagreeing_pair(&self.machine_sequences[machine_id], &guide_positions[machine_id], pick);
            self.swap(machine_id, first, second).is_ok()
        })
    }

    pub fn random_swap<R: Rng + ?Sized>(&mut self, rng: &mut R) -> bool {
        let num_jobs = self.machine_sequences[0].len();
        if num_jobs < 2 { return false; }

        (0..MAX_SWAP_ATTEMPTS).any(|_| {
            let machine_id = rng.random_range(0..self.machine_sequences.len());
            let first = rng.random_range(0..num_jobs);
            let second = (first +rng.random_range(1..num_jobs)) % num_jobs;
            self.swap(machine_id, first, second).is_ok()
        })
    }

    pub fn critical_walk<R: Rng + ?Sized>(mut self, steps: usize, rng: &mut R) -> Solution {
        let mut best = self.solution();
        for _ in 0..steps {
            let swaps = critical_swaps(&self.schedule);
            if swaps.is_empty() {
                break;
            }
            let operation = swaps[rng.random_range(0..swaps.len())];
            let machine_id = self.schedule.operation(operation).machine_id();
            let position = self.schedule.machine_position(operation);

            self.schedule
                .swap_with_next(operation)
                .expect("critical swaps never create a cycle");
            self.machine_sequences[machine_id].swap(position, position + 1);

            if self.makespan() < best.makespan {
                best = self.solution();
            }
        }
        best
    }
}

fn critical_swaps(schedule: &Schedule) -> Vec<usize> {
    let path = schedule.critical_path();
    let blocks: Vec<&[usize]> = path
        .chunk_by(|&a, &b| schedule.operation(a).machine_id() == schedule.operation(b).machine_id())
        .collect();

    let last_block = blocks.len() - 1;
    let mut swaps = vec![];
    for (index, block) in blocks.iter().enumerate() {
        if block.len() < 2 {
            continue;
        }
        if index > 0 {
            swaps.push(block[0]);
        }
        if index < last_block && !swaps.contains(&block[block.len() - 2]) {
            swaps.push(block[block.len() - 2]);
        }
    }
    swaps
}

fn positions(sequence: &[usize]) -> Vec<usize> {
    let mut position = vec![0; sequence.len()];
    for (index, &job_id) in sequence.iter().enumerate() {
        position[job_id] = index;
    }
    position
}

fn count_inversions(values: &mut [usize], buffer: &mut [usize]) -> usize {
    let len = values.len();
    if len < 2 {
        return 0;
    }
    let middle = len / 2;
    let mut inversions = count_inversions(&mut values[..middle], &mut buffer[..middle])
        + count_inversions(&mut values[middle..], &mut buffer[middle..len]);

    let (mut left, mut right) = (0, middle);
    for slot in buffer[..len].iter_mut() {
        if right == len || (left < middle && values[left] <= values[right]) {
            *slot = values[left];
            left += 1;
        } else {
            *slot = values[right];
            right += 1;
            inversions += middle - left;
        }
    }
    values.copy_from_slice(&buffer[..len]);
    inversions
}

fn nth_disagreeing_pair(sequence: &[usize], guide_position: &[usize], n: usize) -> (usize, usize) {
    let mut remaining = n;
    for first in 0..sequence.len() {
        for second in first + 1..sequence.len() {
            if guide_position[sequence[first]] > guide_position[sequence[second]] {
                if remaining == 0 {
                    return (first, second);
                }
                remaining -= 1;
            }
        }
    }
    unreachable!("n is below the number of disagreeing pairs")
}

fn kendall_tau(first: &[usize], second: &[usize]) -> usize {
    let position = positions(second);
    let mut ranks: Vec<usize> = first.iter().map(|&job_id| position[job_id]).collect();
    let mut buffer = vec![0; ranks.len()];
    count_inversions(&mut ranks, &mut buffer)
}