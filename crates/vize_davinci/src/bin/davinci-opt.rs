//! `davinci-opt` - the Davinci stage-dump driver.
//!
//! Currently a round-trip verifier for folio files:
//!
//! ```text
//! davinci-opt --roundtrip <file> [--stage croquis]
//! ```
//!
//! reads `<file>`, parses it with the selected stage's folio, prints it
//! back in `Full` mode, and byte-compares against the input. Exit code 0
//! means the file is canonical (round-trip identity); 1 means a parse
//! failure or a byte difference; 2 means a usage error.
//!
//! Only the `croquis` stage exists today; running the pipeline itself
//! lands in phase 2. The binary is host-side and may use `std`; the
//! `vize_davinci` library stays `no_std + alloc`.

use std::process::ExitCode;

use vize_carton::{String, cstr};
use vize_davinci::folio::{Folio, FolioMode, croquis::CroquisFolio};

const USAGE: &str = "usage: davinci-opt --roundtrip <file> [--stage croquis]";

struct Args {
    roundtrip: std::path::PathBuf,
    stage: String,
}

fn parse_args() -> Result<Args, String> {
    let mut roundtrip = None;
    let mut stage = None;
    let mut argv = std::env::args().skip(1);
    while let Some(arg) = argv.next() {
        match arg.as_str() {
            "--roundtrip" => {
                let value = argv
                    .next()
                    .ok_or_else(|| cstr!("--roundtrip needs a file argument"))?;
                roundtrip = Some(std::path::PathBuf::from(value));
            }
            "--stage" => {
                let value = argv
                    .next()
                    .ok_or_else(|| cstr!("--stage needs a stage name"))?;
                stage = Some(String::from(value.as_str()));
            }
            "--help" | "-h" => return Err(String::default()),
            other => return Err(cstr!("unknown argument: {other}")),
        }
    }
    Ok(Args {
        roundtrip: roundtrip.ok_or_else(|| cstr!("--roundtrip <file> is required"))?,
        stage: stage.unwrap_or_else(|| String::from("croquis")),
    })
}

/// First line (1-based) at which the two texts differ, for the mismatch
/// report.
fn first_divergent_line(input: &str, printed: &str) -> usize {
    let mut line = 1;
    let mut a = input.split('\n');
    let mut b = printed.split('\n');
    loop {
        match (a.next(), b.next()) {
            (Some(x), Some(y)) if x == y => line += 1,
            _ => return line,
        }
    }
}

fn run_roundtrip(args: &Args) -> ExitCode {
    let path = args.roundtrip.display();
    let input = match std::fs::read_to_string(&args.roundtrip) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("davinci-opt: cannot read {path}: {error}");
            return ExitCode::from(1);
        }
    };
    let printed = match args.stage.as_str() {
        "croquis" => match CroquisFolio::parse(&input) {
            Ok(folio) => folio.print_to_string(FolioMode::Full),
            Err(error) => {
                eprintln!("davinci-opt: {path}: {error}");
                return ExitCode::from(1);
            }
        },
        other => {
            eprintln!("davinci-opt: unknown stage: {other} (available: croquis)");
            return ExitCode::from(2);
        }
    };
    if printed.as_str() == input.as_str() {
        println!("roundtrip OK: {path} ({} bytes)", input.len());
        ExitCode::SUCCESS
    } else {
        let line = first_divergent_line(input.as_str(), printed.as_str());
        eprintln!(
            "davinci-opt: {path}: round-trip mismatch starting at line {line} \
             (input {} bytes, printed {} bytes); the input is not canonical - \
             canonical text is what print(parse(input)) emits",
            input.len(),
            printed.len(),
        );
        ExitCode::from(1)
    }
}

fn main() -> ExitCode {
    match parse_args() {
        Ok(args) => run_roundtrip(&args),
        Err(message) => {
            if message.is_empty() {
                println!("{USAGE}");
                ExitCode::SUCCESS
            } else {
                eprintln!("davinci-opt: {message}");
                eprintln!("{USAGE}");
                ExitCode::from(2)
            }
        }
    }
}
