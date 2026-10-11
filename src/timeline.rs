use std::{error::Error, path::PathBuf, str::FromStr};

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Clone, Debug)]
pub struct Param {
    pub from: f32,
    pub to: f32,
}

impl Param {
    fn fixed(value: f32) -> Self {
        Self { from: value, to: value }
    }

    fn range(from: f32, to: f32) -> Self {
        Self { from, to }
    }
}

impl FromStr for Param {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let mut values = value.split("..");
        let parse = |part: Option<&str>| -> Result<f32, String> {
            let part = part.ok_or_else(|| format!("missing value in '{value}'"))?;
            part.trim()
                .parse::<f32>()
                .map_err(|error| format!("could not parse '{}' in '{value}': {error}", part.trim()))
        };
        let from = parse(values.next())?;
        match values.next() {
            Some(to) => {
                let to = parse(Some(to))?;
                if values.next().is_some() {
                    Err(format!("too many numbers in '{value}'"))
                } else {
                    Ok(Self::range(from, to))
                }
            }
            None => Ok(Self::fixed(from)),
        }
    }
}

impl Default for Param {
    fn default() -> Self {
        Self::fixed(1.0)
    }
}

#[derive(Subcommand, Debug)]
#[command(rename_all = "kebab-case")]
pub enum Op {
    Scale {
        #[arg(long, allow_hyphen_values = true)]
        x_factor: Param,
        #[arg(long, allow_hyphen_values = true)]
        y_factor: Param,
    },
    Shear {
        #[arg(long, allow_hyphen_values = true, default_value = "0")]
        horizontal: Param,
        #[arg(long, allow_hyphen_values = true, default_value = "0")]
        vertical: Param,
    },
    FlipHorizontal,
    FlipVertical,
    Rotate {
        #[arg(long, allow_hyphen_values = true)]
        angle: Param,
    },
    RawResize { x: u32, y: u32 },
    Resize { x: u32, y: u32 },
    Multiply {
        #[arg(long, allow_hyphen_values = true)]
        factor: Param,
    },
    MultiplyAll {
        #[arg(long, allow_hyphen_values = true, default_value = "1")]
        r: Param,
        #[arg(long, allow_hyphen_values = true, default_value = "1")]
        g: Param,
        #[arg(long, allow_hyphen_values = true, default_value = "1")]
        b: Param,
    },
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum Loop {
    #[default]
    Reset,
    Pingpong,
    Once,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
#[clap(rename_all = "kebab-case")]
pub enum Ease {
    #[default]
    Linear,
    Quad,
    Cubic,
    EaseInOut,
}

#[derive(Args, Clone, Debug)]
pub struct Timing {
    #[arg(long, global = true, default_value_t = 0.0)]
    pub start: f32,
    #[arg(long, global = true, default_value_t = 1.0)]
    pub end: f32,
    #[arg(long, global = true, default_value = "once")]
    pub looping: Loop,
    #[arg(long, global = true, default_value = "linear")]
    pub ease: Ease,
}

#[derive(Debug)]
pub struct Effect {
    pub op: Op,
    pub timing: Timing,
}

#[derive(Debug)]
pub enum Source {
    Script(PathBuf),
    Inline(Vec<Effect>),
}

#[derive(Subcommand)]
enum InputSource {
    File { path: PathBuf },
    #[command(external_subcommand)]
    Inline(Vec<String>),
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    input: Option<String>,
    #[arg(short, long, global = true)]
    output: Option<String>,
    #[arg(short, long)]
    time: Option<u32>,
    #[command(subcommand)]
    source: InputSource,
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
        Self {
            op: value.op,
            timing: value.timing,
        }
    }
}

#[derive(Debug)]
pub struct Arguments {
    pub input: Option<String>,
    pub output: Option<String>,
    pub time: Option<u32>,
    pub source: Source,
}

pub fn timeline() -> Result<Arguments, Box<dyn Error>> {
    let cli = Cli::parse();
    let source = match cli.source {
        InputSource::File { path } => Source::Script(path),
        InputSource::Inline(args) => {
            let effects = args
                .split(|arg| arg == "+")
                .map(|chunk| EffectCli::try_parse_from(chunk).map(Effect::from))
                .collect::<Result<Vec<_>, _>>()?;
            Source::Inline(effects)
        }
    };

    Ok(Arguments {
        input: cli.input,
        output: cli.output,
        time: cli.time,
        source,
    })
}
