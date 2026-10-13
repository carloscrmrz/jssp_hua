use super::super::job::Job;
use super::super::operation::Operation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Problem {
    jobs: Vec<Job>,
    num_machines: usize,
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

        Ok(Problem {
            jobs,
            num_machines,
        })
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

    pub fn lower_bound(&self) -> usize {
        let longest_job = self.jobs.iter().map(Job::total_timespan).max().unwrap_or(0);

        let mut machine_loads = vec![0; self.num_machines];
        for operation in self.jobs.iter().flat_map(Job::operations) {
            machine_loads[operation.machine_id()] += operation.timespan();
        }
        let busiest_machine = machine_loads.into_iter().max().unwrap_or(0);

        longest_job.max(busiest_machine)
    }
}
