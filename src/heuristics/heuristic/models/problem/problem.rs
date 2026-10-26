use super::super::job::Job;
use super::super::operation::Operation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    jobs: Vec<Job>,
    num_machines: usize,
    positions_on_machines: Vec<Vec<usize>>,
    timespans: Vec<usize>,
    job_predecessors: Vec<usize>,
    job_successors: Vec<usize>,
    known_upper_bound: Option<usize>,
    known_lower_bound: Option<usize>,
}

impl Problem {
    pub fn new(routes: &[Vec<(usize, usize)>], num_machines: usize) -> Result<Problem, String> {
        if routes.is_empty() {
            return Err(String::from("An instance needs at least one job"));
        }
        if num_machines == 0 {
            return Err(String::from("An instance needs at least one machine"));
        }

        let mut jobs = Vec::with_capacity(routes.len());
        for (job_id, route) in routes.iter().enumerate() {
            if route.len() != num_machines {
                return Err(format!(
                    "Job {} has {} operations, expected {}",
                    job_id,
                    route.len(),
                    num_machines
                ));
            }

            let mut visited = vec![false; num_machines];
            let mut operations = Vec::with_capacity(num_machines);
            for &(machine_id, timespan) in route {
                if machine_id >= num_machines {
                    return Err(format!(
                        "Job {} uses machine {}, but machines go from 0 to {}",
                        job_id,
                        machine_id,
                        num_machines - 1
                    ));
                }
                if visited[machine_id] {
                    return Err(format!(
                        "Job {} visits machine {} more than once",
                        job_id, machine_id
                    ));
                }
                visited[machine_id] = true;
                operations.push(Operation::new(job_id, machine_id, timespan));
            }
            jobs.push(Job::new(job_id, operations));
        }

        let mut jobs = Vec::with_capacity(routes.len());
        let mut positions_on_machines = Vec::with_capacity(routes.len());
        for (job_id, route) in routes.iter().enumerate() {
            if route.len() != num_machines {
                return Err(format!(
                    "Job {} has {} operations, expected {}",
                    job_id,
                    route.len(),
                    num_machines
                ));
            }

            let mut visited = vec![false; num_machines];
            let mut operations = Vec::with_capacity(num_machines);
            let mut positions = vec![0; num_machines];

            for (position, &(machine_id, timespan)) in route.iter().enumerate() {
                if machine_id >= num_machines {
                    return Err(format!(
                        "Job {} uses machine {}, but machines go from 0 to {}",
                        job_id,
                        machine_id,
                        num_machines - 1
                    ));
                }
                if visited[machine_id] {
                    return Err(format!(
                        "Job {} visits machine {} more than once",
                        job_id, machine_id
                    ));
                }

                visited[machine_id] = true;
                positions[machine_id] = position;
                operations.push(Operation::new(job_id, machine_id, timespan));
            }
            jobs.push(Job::new(job_id, operations));
            positions_on_machines.push(positions);
        }

        let timespans: Vec<usize> = jobs
            .iter()
            .flat_map(|job| job.operations().iter().map(Operation::timespan))
            .collect();
        let num_operations = routes.len() * num_machines;
        let job_predecessors: Vec<usize> = (0..num_operations)
            .map(|operation| if operation % num_machines == 0 { num_operations } else {operation - 1})
            .collect();
        let job_successors: Vec<usize> = (0..num_operations)
            .map(|operation| if (operation + 1) % num_machines == 0 { num_operations } else {operation + 1})
            .collect();

        Ok(Problem {
            jobs,
            num_machines,
            positions_on_machines,
            timespans,
            job_successors,
            job_predecessors,
            known_upper_bound: None,
            known_lower_bound: None,
        })
    }

    pub fn with_known_bounds(mut self, upper_bound: usize, lower_bound: usize) -> Result<Problem, String> {
        if lower_bound > upper_bound {
            return Err(format!(
                "Lower bound {} is greater than upper bound {}",
                lower_bound, upper_bound
            ));
        }
        self.known_upper_bound = Some(upper_bound);
        self.known_lower_bound = Some(lower_bound);
        Ok(self)
    }

    pub fn known_upper_bound(&self) -> Option<usize> {
        self.known_upper_bound
    }

    pub fn known_lower_bound(&self) -> Option<usize> {
        self.known_lower_bound
    }

    pub fn num_jobs(&self) -> usize {
        self.jobs.len()
    }

    pub fn num_machines(&self) -> usize {
        self.num_machines
    }

    pub fn num_operations(&self) -> usize {
        self.num_jobs() * self.num_machines
    }

    pub fn jobs(&self) -> &[Job] {
        &self.jobs
    }

    pub fn job(&self, job_id: usize) -> &Job {
        &self.jobs[job_id]
    }

    pub fn timespans(&self) -> &[usize] {
        &self.timespans
    }

    pub fn lower_bound(&self) -> usize {
        let longest_job = self.jobs.iter().map(Job::total_timespan).max().unwrap_or(0);

        let mut machine_loads = vec![0; self.num_machines];
        for operation in self.jobs.iter().flat_map(Job::operations) {
            machine_loads[operation.machine_id()] += operation.timespan();
        }
        let busiest_machine = machine_loads.into_iter().max().unwrap_or(0);

        longest_job.max(busiest_machine)
    }

    pub fn upper_bound(&self) -> usize {
        let mut job_finishes = vec![0; self.num_jobs()];
        let mut machine_finishes = vec![0; self.num_machines];

        for pos in 0..self.num_machines {
            for job in &self.jobs {
                let operation = job.operations()[pos];
                let finish = job_finishes[job.id()].max(machine_finishes[operation.machine_id()]) + operation.timespan();
                job_finishes[job.id()] = finish;
                machine_finishes[operation.machine_id()] = finish;
            }
        }

        job_finishes.into_iter().max().unwrap_or(0)
    }

    pub fn position_on(&self, job_id: usize, machine_id: usize) -> usize {
        self.positions_on_machines[job_id][machine_id]
    }

    pub fn job_predecessors(&self) -> &[usize] {
        &self.job_predecessors
    }

    pub fn job_successors(&self) -> &[usize] {
        &self.job_successors
    }
}
