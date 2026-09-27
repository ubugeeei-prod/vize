//! `vize dump --level l1|l2|l3 --roundtrip FILE` validates existing input.
//!
//! L1 checks lossless Vue-template source, including recoverable holes.
//! L2/L3 check canonical Full dump bytes, not semantic IR validity.
//! No compiler pipeline or source-output replacement is involved.

mod roundtrip;

use std::path::PathBuf;

use clap::{Args, ValueEnum};

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
pub struct DumpArgs {
    /// L1 Vue-template source fidelity, or L2/L3 canonical Full dump bytes
    #[arg(long, value_enum, value_name = "LEVEL")]
    pub level: Level,

    /// Read FILE and require byte identity after the selected level's roundtrip
    #[arg(long, value_name = "FILE")]
    pub roundtrip: PathBuf,
}

pub fn run(args: DumpArgs) {
    let path = args.roundtrip.as_path();
    let input = match std::fs::read_to_string(path) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("dump: cannot read {}: {error}", path.display());
            std::process::exit(1);
        }
    };
    let printed = match roundtrip::reprint(args.level, &input) {
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
            args.level.id(),
            input.len(),
            printed.len(),
        );
        std::process::exit(1);
    }
    println!(
        "dump: {} roundtrip OK: {} ({} bytes)",
        args.level.id(),
        path.display(),
        input.len(),
    );
}
