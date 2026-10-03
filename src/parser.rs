use clap::Parser;

use crate::{timeline::Op};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub input: String,

    #[arg(short, long, global = true)]
    pub output: Option<String>,

    #[arg(short, long)]
    pub duration: Option<f32>,

    #[arg(required = true, num_args = 1.., allow_hyphen_values = true)]
    ops: Vec<String>,
}

#[derive(Parser)]
#[command(no_binary_name = true)]
struct Step {
    #[command(subcommand)]
    op: Op,
}

pub fn validate_angle(s: &str) -> Result<f32, String> {
    let val: f32 = s
        .parse()
        .map_err(|_| format!("{} isn't a valid floating-point number", s))?;

    if (0.0..=70.0).contains(&val) {
        Ok(val)
    } else {
        Err(String::from("value must be between 0.0 and 70.0"))
    }
}

pub fn get_args() -> (Args, Vec<Op>) {
    let args = Args::parse();

    let steps: Vec<Step> = args.ops
        .split(|a| a == "+")
        .map(|chunk| Step::try_parse_from(chunk).unwrap_or_else(|e| e.exit()))
        .collect();

    let ops: Vec<Op> = steps.into_iter().map(|s| s.op).collect();

    (args, ops)
}
