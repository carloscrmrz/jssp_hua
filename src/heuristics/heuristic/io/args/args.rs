use clap::Parser;

#[derive(Parser)]
#[command(version, about)]
pub struct Args {
    #[arg(short, long)]
    pub instance: Option<String>,

    #[clap(short('p'), long)]
    pub instance_path: Option<String>,

    #[arg(short, long)]
    pub seed: Option<u64>,

    #[arg(short('t'), long)]
    pub threads: Option<usize>,

    #[arg(long, default_value_t = 1, value_parser = clap::value_parser!(u8).range(1..=5))]
    pub variant: u8,

    #[arg(long)]
    pub iterations: Option<usize>,

    #[arg(long)]
    pub k: Option<f64>,

    #[arg(long)]
    pub rimpk: Option<f64>,

    #[arg(long)]
    pub radius_scale: Option<f64>,

    #[arg(long)]
    pub adventurer_swaps: Option<usize>,

    #[arg(long)]
    pub local_rounds: Option<usize>,
}
