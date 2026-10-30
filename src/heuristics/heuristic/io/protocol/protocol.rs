use serde::Serialize;

#[derive(Serialize)]
pub struct ResolvedParams {
    pub variant: u8,
    pub iterations: usize,
    pub k: f64,
    pub rimpk: f64,
    pub radius_scale: f64,
    pub adventurer_swaps: usize,
    pub local_rounds: usize,
}

#[derive(Serialize)]
pub struct SolutionPayload {
    pub machine_sequences: Vec<Vec<usize>>,
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProtocolEvent {
    RunStarted {
        v: u32,
        heuristic: &'static str,
        heuristic_version: String,
        instance_hash: String,
        instance_path: String,
        seed: String,
        params: ResolvedParams,
        objective_name: &'static str,
        objective_direction: &'static str,
    },
    Metric {
        v: u32,
        step: usize,
        name: &'static str,
        value: f64,
        elapsed_ms: u128,
    },
    Result {
        v: u32,
        objective_value: usize,
        feasible: bool,
        iterations_run: usize,
        elapsed_ms: u128,
        solution: SolutionPayload,
    },
    Error {
        v: u32,
        message: String,
    },
}
pub fn line(event: &ProtocolEvent) -> String {
    serde_json::to_string(event).expect("a ProtocolEvent always serializes") + "\n"
}
