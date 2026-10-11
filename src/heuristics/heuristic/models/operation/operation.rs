#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Operation {
    job_id: usize,
    machine_id: usize,
    timespan: usize,
}

impl Operation {
    pub fn new(job_id: usize, machine_id: usize, timespan: usize) -> Operation {
        Operation {
            job_id,
            machine_id,
            timespan,
        }
    }

    pub fn job_id(&self) -> usize {
        self.job_id
    }

    pub fn machine_id(&self) -> usize {
        self.machine_id
    }

    pub fn timespan(&self) -> usize {
        self.timespan
    }
}
