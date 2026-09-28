//! `vize dump` validates a level dump or exports the native stage ladder.
//!
//! L1 checks lossless Vue-template source, including recoverable holes.
//! L2/L3 check canonical Full dump bytes, not semantic IR validity.
//! The all-level feed captures the native stage ladder; it does not replace
//! product compilation.

mod all_levels;
mod roundtrip;

use std::path::PathBuf;

use clap::{ArgGroup, Args, ValueEnum};

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

#[derive(Args)]
#[command(group(ArgGroup::new("mode").required(true).args(["roundtrip", "all_levels"])))]
pub struct DumpArgs {
    /// L1 Vue-template source fidelity, or L2/L3 canonical Full dump bytes
    #[arg(long, value_enum, value_name = "LEVEL", requires = "roundtrip")]
    pub level: Option<Level>,

    /// Read FILE and require byte identity after the selected level's roundtrip
    #[arg(long, value_name = "FILE", requires = "level")]
    pub roundtrip: Option<PathBuf>,

    /// Capture L1, L2 and L3 pages from one native template ladder run
    #[arg(long, requires_all = ["json", "source"])]
    pub all_levels: bool,

    /// Write the stage feed as JSON
    #[arg(long, requires = "all_levels")]
    pub json: bool,

    /// Raw Vue template to capture (.vue SFCs await the native container)
    #[arg(value_name = "FILE", requires = "all_levels")]
    pub source: Option<PathBuf>,
}

pub fn run(args: DumpArgs) {
    if let Some(path) = args.source.as_deref() {
        all_levels::run(path);
        return;
    }
    let (Some(path), Some(level)) = (args.roundtrip.as_deref(), args.level) else {
        eprintln!("dump: choose --level and --roundtrip together");
        std::process::exit(2);
    };
    let input = match std::fs::read_to_string(path) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("dump: cannot read {}: {error}", path.display());
            std::process::exit(1);
        }
    };
    let printed = match roundtrip::reprint(level, &input) {
        Ok(printed) => printed,
        Err(error) => {
            eprintln!("dump: {}: {error}", path.display());
            std::process::exit(1);
        }
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
    println!(
        "dump: {} roundtrip OK: {} ({} bytes)",
        level.id(),
        path.display(),
        input.len(),
    );
}
