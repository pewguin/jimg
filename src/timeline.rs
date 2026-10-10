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
        let parse_num = |st: Option<&str>| -> Result<f32, Self::Err> {
            let string = st.ok_or(format!("missing value in '{}'", s))?;
            string.trim().parse::<f32>().map_err(|e| format!("could not parse {} in '{}': {}", string.trim(), s, e.to_string()))
        };
        let mut nums = s.split("..");
        let from = parse_num(nums.next())?;
        let to = parse_num(nums.next())?;
        match nums.next() {
            Some(_) => Err(format!("too many numbers in '{}'", s)),
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

impl Default for Param {
    fn default() -> Self {
        Self::fixed(1.0)
    }
}

fn param_one() -> Param { Param::fixed(1.0) }
fn param_zero() -> Param { Param::fixed(0.0) }

fn one() -> f32 { 1.0 }
fn zero() -> f32 { 0.0 }

#[derive(Subcommand, Deserialize, Debug)]
#[serde(tag = "op", rename_all = "kebab-case")]
pub enum Op {
    Scale {
        #[arg(long, allow_hyphen_values = true)]
        #[serde(default)]
        x_factor: Param, 
        #[arg(long, allow_hyphen_values = true)]
        #[serde(default)]
        y_factor: Param,
    },
    Shear {
        #[arg(long, allow_hyphen_values = true)]
        #[serde(default = "param_zero")]
        horizontal: Param,
        #[arg(long, allow_hyphen_values = true)]
        #[serde(default = "param_zero")]
        vertical: Param,
    },
    FlipHorizontal,
    FlipVertical,
    Rotate {
        #[arg(long, allow_hyphen_values = true)]
        angle: Param
    },
    RawResize { x: u32, y: u32 },
    Resize { x: u32, y: u32 },
    Multiply { 
        #[arg(long, allow_hyphen_values = true)]
        factor: Param
    },
    MultiplyAll {
        #[arg(long, allow_hyphen_values = true)]
        #[serde(default = "param_one")]
        r: Param,
        #[arg(long, allow_hyphen_values = true)]
        #[serde(default = "param_one")]
        g: Param,
        #[arg(long, allow_hyphen_values = true)]
        #[serde(default = "param_one")]
        b: Param,
    }
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
    #[serde(default, rename = "loop")]
    pub looping: Loop,
    #[arg(long, global = true, default_value = "linear")]
    #[serde(default)]
    pub ease: Ease,
}

#[derive(Deserialize, Debug)]
pub struct Effect {
    #[serde(flatten)]
    pub op: Op,
    #[serde(flatten)]
    pub timing: Timing,
}

#[derive(Deserialize, Debug)]
pub struct Timeline {
    #[serde(default)]
    pub input: Option<String>,
    #[serde(default)]
    pub output: Option<String>,
    pub length: u32,
    #[serde(default)]
    pub effects: Vec<Effect>
}

impl Timeline {
    fn with_input_output(mut self, input: Option<String>, output: Option<String>) -> Timeline{
        self.input = Some(input.unwrap_or_else(|| self.input.expect("no input file")));
        self.output = output.or(self.output);
        self
    }
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
    #[arg(short, long)]
    pub time: Option<u32>,

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
        Source::File { path } => {
            let tl: Timeline = toml::from_str(&fs::read_to_string(path)?)?;
            tl.with_input_output(cli.input, cli.output)
        },
        Source::Inline(args) => Timeline {
            input: cli.input,
            output: cli.output,
            length: cli.time.expect("specify length of gif with -t"),
            effects: parse_effects(&args)?
        },
    })
}
