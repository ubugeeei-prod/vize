//! `vize dump` validates a level dump or captures one product compilation.
//!
//! L1 checks lossless Vue-template source, including recoverable holes.
//! L2/L3 check canonical Full dump bytes, not semantic IR validity.
//! The all-level feed observes the selected product backend's executed levels.

mod all_levels;
mod roundtrip;
mod script;

use std::path::PathBuf;

use clap::{ArgGroup, Args, ValueEnum};
use vize_l1::embed::Lang;
use vize_l1::embed::syntax::{ProgramGoal, ProgramOptions};

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum ScriptLanguage {
    Js,
    Ts,
    Jsx,
    Tsx,
}

#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum ScriptGoal {
    #[default]
    Module,
    Script,
}

impl ScriptLanguage {
    fn options(self, goal: ScriptGoal) -> ProgramOptions {
        ProgramOptions {
            lang: match self {
                Self::Js | Self::Jsx => Lang::Js,
                Self::Ts | Self::Tsx => Lang::Ts,
            },
            jsx: matches!(self, Self::Jsx | Self::Tsx),
            goal: match goal {
                ScriptGoal::Module => ProgramGoal::Module,
                ScriptGoal::Script => ProgramGoal::Script,
            },
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum Level {
    L1,
    L2,
    L3,
}

impl Level {
    fn id(self) -> &'static str {
        match self {
            Self::L1 => "l1",
            Self::L2 => "l2",
            Self::L3 => "l3",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, ValueEnum)]
pub enum Pipeline {
    #[default]
    Dom,
    Ssr,
    Vapor,
}

#[derive(Args)]
#[command(group(ArgGroup::new("mode").required(true).args(["roundtrip", "all_levels"])))]
pub struct DumpArgs {
    /// L1 Vue-template source fidelity, or L2/L3 canonical Full dump bytes
    #[arg(long, value_enum, value_name = "LEVEL", requires = "roundtrip")]
    pub level: Option<Level>,

    /// Read FILE and require byte identity after the selected level's roundtrip
    #[arg(long, value_name = "FILE", requires = "level")]
    pub roundtrip: Option<PathBuf>,

    /// Inspect a native JS/TS/JSX/TSX Program while checking L1 source bytes
    #[arg(long, value_enum, requires = "roundtrip")]
    pub script: Option<ScriptLanguage>,

    /// Select the script parse goal explicitly (default: module)
    #[arg(long, value_enum, requires = "script")]
    pub script_goal: Option<ScriptGoal>,

    /// Capture only the levels executed by one product compile
    #[arg(long, requires_all = ["json", "source"])]
    pub all_levels: bool,

    /// Write the stage feed as JSON
    #[arg(long, requires = "all_levels")]
    pub json: bool,

    /// Compile FILE (.html, .vue or .pug) and capture its executed levels
    #[arg(value_name = "FILE", requires = "all_levels")]
    pub source: Option<PathBuf>,

    /// Select a real product backend; named no-op passes are unsupported
    #[arg(long, value_enum, requires = "all_levels")]
    pub pipeline: Option<Pipeline>,

    /// Write captured pages and the versioned product feed into DIR
    #[arg(long, value_name = "DIR", requires = "all_levels")]
    pub dump_dir: Option<PathBuf>,

    /// Write a page only when its bytes differ from the preceding page
    #[arg(long, requires = "dump_dir")]
    pub dump_after_change: bool,

    /// Write observed timing data, including availability, as JSON
    #[arg(long, value_name = "FILE", requires = "all_levels")]
    pub timing_json: Option<PathBuf>,

    /// Write observed remarks, including availability, as JSON
    #[arg(long, value_name = "FILE", requires = "all_levels")]
    pub remarks: Option<PathBuf>,
}

pub fn run(args: DumpArgs) {
    if args.source.is_some() {
        all_levels::run(&args);
        return;
    }
    let (Some(path), Some(level)) = (args.roundtrip.as_deref(), args.level) else {
        eprintln!("dump: choose --level and --roundtrip together");
        std::process::exit(2);
    };
    if args.script.is_some() && !matches!(level, Level::L1) {
        eprintln!("dump: --script requires --level l1");
        std::process::exit(2);
    }
    let input = match std::fs::read_to_string(path) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("dump: cannot read {}: {error}", path.display());
            std::process::exit(1);
        }
    };
    let (printed, script_facts) = if let Some(language) = args.script {
        match script::inspect(
            &input,
            language.options(args.script_goal.unwrap_or_default()),
        ) {
            Ok(inspected) => (inspected.printed, Some(inspected.facts)),
            Err(error) => {
                eprintln!("dump: {}: {error:?}", path.display());
                std::process::exit(1);
            }
        }
    } else {
        (
            match roundtrip::reprint(level, &input) {
                Ok(printed) => printed,
                Err(error) => {
                    eprintln!("dump: {}: {error}", path.display());
                    std::process::exit(1);
                }
            },
            None,
        )
    };
    if printed.as_str() != input.as_str() {
        let line = roundtrip::first_divergent_line(&input, printed.as_str());
        eprintln!(
            "dump: {}: {} roundtrip mismatch starting at line {line} \
             (input {} bytes, printed {} bytes)",
            path.display(),
            level.id(),
            input.len(),
            printed.len(),
        );
        std::process::exit(1);
    }
    if let Some(facts) = script_facts {
        print!("{facts}");
    }
    println!(
        "dump: {} roundtrip OK: {} ({} bytes)",
        level.id(),
        path.display(),
        input.len(),
    );
}
