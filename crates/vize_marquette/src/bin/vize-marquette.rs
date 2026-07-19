//! Small Rust CLI for contract validation, canonicalization, and fingerprints.

use std::{
    env, fs,
    io::{self, Write},
    process::ExitCode,
};

use vize_marquette::{ApplicationContract, canonical_json, contract_fingerprint};

/// Runs the marquette CLI and renders errors without panicking.
fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("vize-marquette: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Parses one command and returns its process-level result.
fn run() -> Result<ExitCode, Box<dyn std::error::Error>> {
    let mut arguments = env::args().skip(1);
    let command = arguments
        .next()
        .ok_or("usage: vize-marquette validate|canonical|fingerprint <contract.json>")?;
    let path = arguments.next().ok_or("missing contract path")?;
    if arguments.next().is_some() {
        return Err("unexpected additional arguments".into());
    }
    let contract: ApplicationContract = serde_json::from_slice(&fs::read(path)?)?;

    match command.as_str() {
        "validate" => {
            let diagnostics = contract.validate();
            println!("{}", serde_json::to_string_pretty(&diagnostics)?);
            Ok(
                if diagnostics
                    .iter()
                    .any(|value| value.severity == vize_marquette::DiagnosticSeverity::Error)
                {
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                },
            )
        }
        "canonical" => {
            io::stdout().write_all(&canonical_json(&contract)?)?;
            Ok(ExitCode::SUCCESS)
        }
        "fingerprint" => {
            println!("{}", contract_fingerprint(&contract)?);
            Ok(ExitCode::SUCCESS)
        }
        _ => {
            eprintln!("unknown command {command}");
            Err("unknown command".into())
        }
    }
}
