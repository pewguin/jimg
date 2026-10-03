use std::{error::Error, fs, path::PathBuf, str::FromStr};
use serde::Deserialize;
use clap::{Args, ValueEnum, Subcommand, Parser};

#[derive(Deserialize)]
#[serde(untagged)]
enum ParamRepr {
    Fixed(f32),
    Text(String),
    Range { from: f32, to: f32 },
}

#[derive(Deserialize, Clone, Debug)]
#[serde(try_from = "ParamRepr")]
pub struct Param {
    pub from: f32,
    pub to: f32,
}

impl Param {
    fn fixed(val: f32) -> Self {
        Self {
            from: val,
            to: val
        }
    }

    fn range(from: f32, to: f32) -> Self {
        Self {
            from,
            to,
        }
    }
}

impl FromStr for Param {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut nums = s.split("..");
        let from = nums.next().ok_or("no 'from' value")?.parse::<f32>().map_err(|e| e.to_string())?;
        let to = nums.next().ok_or("no 'to' value")?.parse::<f32>().map_err(|e| e.to_string())?;
        match nums.next() {
            Some(_) => Err("too many numbers".to_string()),
            None => Ok(Self::range(from, to)),
        }
    }
}

impl TryFrom<ParamRepr> for Param {
    type Error = String;
    fn try_from(r: ParamRepr) -> Result<Self, String> {
        match r {
            ParamRepr::Fixed(v) => Ok(Param::fixed(v)),
            ParamRepr::Text(s) => s.parse(),
            ParamRepr::Range { from, to } => Ok(Param::range(from, to)),
        }
    }
}


#[derive(Subcommand, Deserialize, Debug)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum Op {
    Squish { amount: Param },
    Slant {
        angle: Param
    },
    FlipHorizontal,
    FlipVertical,
    Rotate {
        #[arg(long, allow_hyphen_values = true)]
        angle: Param
    },
    RawResize { x: u32, y: u32 },
    Resize { x: u32, y: u32 },
}

#[derive(Clone, Copy, Debug, Default, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Loop {
    #[default]
    Reset,
    Pingpong,
    Once,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, ValueEnum)]
#[serde(rename_all = "kebab-case")]
pub enum Ease {
    #[default]
    Linear,
    Quad,
    Cubic,
    EaseInOut,
}

#[derive(Args, Deserialize, Clone, Debug)]
pub struct Timing {
    #[arg(long, global = true, default_value_t = 0.0)]
    #[serde(default)]
    pub start: f32,
    #[arg(long, global = true, default_value_t = 1.0)]
    #[serde(default = "one")]
    pub end: f32,
    #[arg(long, global = true, default_value = "once")]
    #[serde(default)]
    pub looping: Loop,
    #[arg(long, global = true, default_value = "linear")]
    #[serde(default, rename = "loop")]
    pub ease: Ease,
}
fn one() -> f32 { 1.0 }

#[derive(Deserialize, Debug)]
pub struct Effect {
    pub op: Op,
    pub timing: Timing,
}

#[derive(Deserialize, Debug)]
pub struct Timeline {
    #[serde(default)]
    pub input: Option<String>,
    #[serde(default)]
    pub output: Option<String>,
    #[serde(default)]
    pub effects: Vec<Effect>
}

#[derive(Subcommand)]
enum Source {
    File { path: PathBuf },
    #[command(external_subcommand)]
    Inline(Vec<String>)
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    pub input: Option<String>,
    #[arg(short, long, global = true)]
    pub output: Option<String>,

    #[command(subcommand)]
    source: Source,
}

#[derive(Parser)]
#[command(no_binary_name = true)]
struct EffectCli {
    #[command(subcommand)]
    op: Op,
    #[command(flatten)]
    timing: Timing,
}

impl From<EffectCli> for Effect {
    fn from(value: EffectCli) -> Self {
        Effect { op: value.op, timing: value.timing }
    }
}

fn parse_effects(args: &[String]) -> Result<Vec<Effect>, clap::Error> {
    args.split(|a| a == "+")
        .map(|chunk| EffectCli::try_parse_from(chunk).map(|c| c).map(Effect::from))
        .collect()
}

pub fn timeline() -> Result<Timeline, Box<dyn Error>> {
    let cli = Cli::parse();
    Ok(match cli.source {
        Source::File { path } => toml::from_str(&fs::read_to_string(path)?)?,
        Source::Inline(args) => Timeline {
            input: cli.input,
            output: cli.output,
            effects: parse_effects(&args)?
        },
    })
}
