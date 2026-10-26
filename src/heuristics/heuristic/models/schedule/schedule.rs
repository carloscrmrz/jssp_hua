use super::super::operation::Operation;
use super::super::problem::Problem;


#[derive(Clone)]
pub struct Schedule<'a> {
    problem: &'a Problem,
    machine_predecessors: Vec<usize>,
    machine_successors: Vec<usize>,
    machine_positions: Vec<usize>,

    order: Vec<usize>,
    order_positions: Vec<usize>,

    finishes: Vec<usize>,
    last: usize,

    pending: Vec<usize>,
    ready: Vec<usize>,
    reordered: Vec<usize>,
}

impl<'a> Schedule<'a> {
    pub fn new(problem: &'a Problem, machine_sequences: &[Vec<usize>]) -> Result<Schedule<'a>, String> {
        let num_machines = problem.num_machines();
        let num_operations = problem.num_operations();

        let none = num_operations;
        let mut machine_predecessors: Vec<usize> = vec![none; num_operations];
        let mut machine_successors: Vec<usize> = vec![none; num_operations];
        let mut machine_positions: Vec<usize> = vec![none; num_operations];

        for (machine_id, sequence) in machine_sequences.iter().enumerate() {
            let mut previous = none;
            for (position, &job_id) in sequence.iter().enumerate() {
                let operation = job_id * num_machines + problem.position_on(job_id, machine_id);
                machine_positions[operation] = position;
                machine_predecessors[operation] = previous;
                if previous != none {
                    machine_successors[previous] = operation;
                }
                previous = operation;
            }
        }

        let mut schedule = Schedule {
            problem,
            machine_predecessors,
            machine_successors,
            machine_positions,
            order: (0..num_operations).collect(),

            order_positions: (0..num_operations).chain([usize::MAX]).collect(),
            finishes: vec![0; num_operations + 1],
            last: 0,
            pending: vec![0; num_operations],
            ready: Vec::with_capacity(num_operations),
            reordered: Vec::with_capacity(num_operations),
        };
        if !schedule.reorder(0, num_operations.saturating_sub(1)) {
            return Err(String::from("Machine orders create a cycle: infeasible solution"));
        }
        schedule.update_finishes(0);
        Ok(schedule)
    }

    pub fn problem(&self) -> &'a Problem { self.problem }

    pub fn makespan(&self) -> usize { self.finishes[self.last] }

    pub fn operation(&self, operation: usize) -> Operation {
        let num_machines = self.problem.num_machines();
        self.problem.job(operation / num_machines).operations()[operation % num_machines]
    }

    pub fn machine_position(&self, operation: usize) -> usize { self.machine_positions[operation] }

    // Toposorts order[first..=last], using only the arcs on that segment. The rest of the order
    // is still valid. Will return false on a cycle.
    fn reorder(&mut self, first: usize, last: usize) -> bool {
        if self.order.is_empty() {
            return true;
        }

        let inside = |order_pos: &[usize], operation: usize| (first..=last).contains(&order_pos[operation]);
        self.ready.clear();
        for i in first..=last {
            let operation = self.order[i];
            let pending = [self.problem.job_predecessors()[operation], self.machine_predecessors[operation]]
                .into_iter()
                .filter(|&p| inside(&self.order, p))
                .count();
            self.pending[operation] = pending;
            if pending == 0 {
                self.ready.push(operation);
            }
        }

        self.reordered.clear();
        while let Some(operation) = self.ready.pop() {
            self.reordered.push(operation);
            for next in [self.problem.job_successors()[operation], self.machine_successors[operation]] {
                if inside(&self.order_positions, next) {
                    self.pending[next] -= 1;
                    if self.pending[next] == 0 {
                        self.ready.push(next);
                    }
                }
            }
        }

        if self.reordered.len() < last - first + 1 {
            return false;
        }

        for (offset, &operation) in self.reordered.iter().enumerate() {
            self.order[first + offset] = operation;
            self.order_positions[operation] = first + offset;
        }
        true
    }

    fn update_finishes(&mut self, from: usize) {
        let timespans = self.problem.timespans();
        let job_predecessors = self.problem.job_predecessors();
        let none = self.order.len();
        for &operation in &self.order[from..] {
            let job = self.finishes[job_predecessors[operation]];
            let machine = self.finishes[self.machine_predecessors[operation]];
            self.finishes[operation] = job.max(machine) + timespans[operation];
        }

        let num_machines = self.problem.num_machines();
        self.last = (num_machines - 1..none)
            .step_by(num_machines)
            .max_by_key(|&operation| (self.finishes[operation], std::cmp::Reverse(self.order_positions[operation])))
            .unwrap_or(0);
    }

    pub fn exchange(&mut self, first: usize, second: usize) -> Result<(), String> {
        if self.operation(first).machine_id() != self.operation(second).machine_id() {
            return Err(String::from("Only operations on the same machine can be swapped"))
        }

        if first == second { return Ok(()); }
        let (first, second) = if self.machine_positions[first] < self.machine_positions[second] {
            (first, second)
        } else {
            (second, first)
        };

        let (start, end) = (self.order_positions[first], self.order_positions[second]);
        self.exchange_links(start, end);
        if !self.reorder(start, end) {
            self.exchange_links(second, first);
            return Err(String::from("Cycle detected: infeasible solution"));
        }

        self.update_finishes(start);
        Ok(())
    }

    fn exchange_links(&mut self, first: usize, second: usize) {
        let none = self.order.len();
        let before_first = self.machine_predecessors[first];
        let after_second = self.machine_successors[second];
        if before_first != none {
            self.machine_successors[before_first] = second;
        }
        if after_second != none {
            self.machine_predecessors[after_second] = first;
        }

        if self.machine_successors[first] == second {
            self.machine_predecessors[second] = before_first;
            self.machine_successors[second] = first;
            self.machine_predecessors[first] = second;
            self.machine_successors[first] = after_second;
        } else {
            let after_first = self.machine_successors[first];
            let before_second = self.machine_predecessors[second];
            self.machine_predecessors[after_first] = second;
            self.machine_successors[before_second] = first;
            self.machine_predecessors[second] = before_first;
            self.machine_successors[second] = after_first;
            self.machine_predecessors[first] = before_second;
            self.machine_successors[first] = after_second;
        }
        self.machine_positions.swap(first, second);
    }
}