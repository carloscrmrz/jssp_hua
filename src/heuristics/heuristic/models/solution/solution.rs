use rand::Rng;
use rand::seq::SliceRandom;

use super::super::graph::DisjunctiveGraph;
use super::super::problem::Problem;

#[derive(Debug, Clone, PartialEq, Eq)]
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
}
