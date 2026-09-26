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

}
