use super::super::operation::Operation;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Job {
    id: usize,
    operations: Vec<Operation>,
}

impl Job {
    pub fn new(id: usize, operations: Vec<Operation>) -> Job {
        Job { id, operations }
    }

    pub fn id(&self) -> usize {
        self.id
    }

    pub fn operations(&self) -> &[Operation] {
        &self.operations
    }

    pub fn position_on(&self, machine_id: usize) -> Option<usize> {
        self.operations
            .iter()
            .position(|operation| operation.machine_id() == machine_id)
    }

    pub fn total_timespan(&self) -> usize {
        self.operations.iter().map(Operation::timespan).sum()
    }
}
