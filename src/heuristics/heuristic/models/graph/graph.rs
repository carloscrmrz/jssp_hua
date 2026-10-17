use petgraph::Direction;
use petgraph::algo::toposort;
use petgraph::graph::{Graph, NodeIndex};

use super::super::operation::Operation;
use super::super::problem::Problem;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeType {
    Conjunctive,
    Disjunctive,
}

pub struct DisjunctiveGraph {
    graph: Graph<Operation, EdgeType>,
    nodes: Vec<Vec<NodeIndex>>,
}

impl DisjunctiveGraph {
    pub fn new(problem: &Problem, machine_sequences: &[Vec<usize>]) -> Self {
        let mut graph = Graph::<Operation, EdgeType>::new();

        let nodes: Vec<Vec<NodeIndex>> = problem
            .jobs()
            .iter()
            .map(|job| {
                let chain: Vec<NodeIndex> = job
                    .operations()
                    .iter()
                    .map(|&operation| graph.add_node(operation))
                    .collect();
                for pair in chain.windows(2) {
                    graph.add_edge(pair[0], pair[1], EdgeType::Conjunctive);
                }
                chain
            })
            .collect();

        for (machine_id, sequence) in machine_sequences.iter().enumerate() {
            let on_machine: Vec<NodeIndex> = sequence
                .iter()
                .map(|&job_id| {
                    let position = problem
                        .job(job_id)
                        .position_on(machine_id)
                        .expect("every job visits every machine");
                    nodes[job_id][position]
                })
                .collect();
            for pair in on_machine.windows(2) {
                graph.add_edge(pair[0], pair[1], EdgeType::Disjunctive);
            }
        }

        DisjunctiveGraph { graph, nodes }
    }

    pub fn start_times(&self) -> Result<Vec<Vec<usize>>, String> {
        let (starts, _) = self.longest_paths()?;
        Ok(self
            .nodes
            .iter()
            .map(|chain| chain.iter().map(|node| starts[node.index()]).collect())
            .collect())
    }

    pub fn makespan(&self) -> Result<usize, String> {
        let (starts, _) = self.longest_paths()?;
        Ok(self
            .graph
            .node_indices()
            .map(|node| starts[node.index()] + self.graph[node].timespan())
            .max()
            .unwrap_or(0))
    }

    fn longest_paths(&self) -> Result<(Vec<usize>, Vec<Option<NodeIndex>>), String> {
        let order = toposort(&self.graph, None)
            .map_err(|_| String::from("Machine orders create a cycle: infeasible solution"))?;

        let mut starts = vec![0; self.graph.node_count()];
        let mut predecessors = vec![None; self.graph.node_count()];
        for node in order {
            let finish = starts[node.index()] + self.graph[node].timespan();
            for next in self.graph.neighbors_directed(node, Direction::Outgoing) {
                if finish > starts[next.index()] {
                    starts[next.index()] = finish;
                    predecessors[next.index()] = Some(node);
                }
            }
        }
        Ok((starts, predecessors))
    }
}
