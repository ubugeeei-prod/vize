//! Small Rust CLI for contract validation, canonicalization, and fingerprints.
//!
//! Application contracts: `vize-marquette schema|validate|canonical|fingerprint [contract.json]`.
//! Test-run evidence: `vize-marquette test-run schema|validate|canonical|fingerprint [evidence.json]`.
//! `schema` prints the embedded JSON Schema and takes no path.

use std::{
    env, fs,
    io::{self, Write},
    process::ExitCode,
};

use vize_marquette::{
    APPLICATION_CONTRACT_JSON_SCHEMA, ApplicationContract, ContractDiagnostic, DiagnosticSeverity,
    TEST_RUN_EVIDENCE_JSON_SCHEMA, TestRunEvidence, canonical_json, canonical_test_run_json,
    contract_fingerprint, test_run_fingerprint,
};

const USAGE: &str =
    "usage: vize-marquette [test-run] schema|validate|canonical|fingerprint [<path.json>]";

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
    let mut command = arguments.next().ok_or(USAGE)?;
    let test_run = command == "test-run";
    if test_run {
        command = arguments.next().ok_or(USAGE)?;
    }

    if command == "schema" {
        if arguments.next().is_some() {
            return Err("unexpected additional arguments".into());
        }
        print!(
            "{}",
            if test_run {
                TEST_RUN_EVIDENCE_JSON_SCHEMA
            } else {
                APPLICATION_CONTRACT_JSON_SCHEMA
            }
        );
        return Ok(ExitCode::SUCCESS);
    }

    let path = arguments.next().ok_or("missing document path")?;
    if arguments.next().is_some() {
        return Err("unexpected additional arguments".into());
    }
    let bytes = fs::read(path)?;

    if test_run {
        let evidence: TestRunEvidence = serde_json::from_slice(&bytes)?;
        match command.as_str() {
            "validate" => print_diagnostics(&evidence.validate()),
            "canonical" => {
                io::stdout().write_all(&canonical_test_run_json(&evidence)?)?;
                Ok(ExitCode::SUCCESS)
            }
            "fingerprint" => {
                println!("{}", test_run_fingerprint(&evidence)?);
                Ok(ExitCode::SUCCESS)
            }
            _ => Err(USAGE.into()),
        }
    } else {
        let contract: ApplicationContract = serde_json::from_slice(&bytes)?;
        match command.as_str() {
            "validate" => print_diagnostics(&contract.validate()),
            "canonical" => {
                io::stdout().write_all(&canonical_json(&contract)?)?;
                Ok(ExitCode::SUCCESS)
            }
            "fingerprint" => {
                println!("{}", contract_fingerprint(&contract)?);
                Ok(ExitCode::SUCCESS)
            }
            _ => Err(USAGE.into()),
        }
    }
}

/// Prints diagnostics as JSON and maps any error to a failing exit code.
fn print_diagnostics(
    diagnostics: &[ContractDiagnostic],
) -> Result<ExitCode, Box<dyn std::error::Error>> {
    println!("{}", serde_json::to_string_pretty(diagnostics)?);
    Ok(
        if diagnostics
            .iter()
            .any(|value| value.severity == DiagnosticSeverity::Error)
        {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        },
    )
}
