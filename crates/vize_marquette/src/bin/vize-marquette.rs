//! Small Rust CLI for contract validation, canonicalization, and fingerprints.
//!
//! Application contracts: `vize-marquette schema|validate|canonical|fingerprint [contract.json]`.
//! Test-run evidence: `vize-marquette test-run schema|validate|canonical|fingerprint [evidence.json]`.
//! Native build profiles: `vize-marquette native-build schema|validate [profile.json]`.
//! `schema` prints the embedded JSON Schema and takes no path.

use std::{
    env, fs,
    io::{self, Write},
    process::ExitCode,
};

use vize_marquette::{
    APPLICATION_CONTRACT_JSON_SCHEMA, ApplicationContract, ContractDiagnostic, DiagnosticSeverity,
    NATIVE_BUILD_PROFILE_JSON_SCHEMA, NativeBuildProfile, TEST_RUN_EVIDENCE_JSON_SCHEMA,
    TestRunEvidence, canonical_json, canonical_test_run_json, contract_fingerprint,
    test_run_fingerprint, validate_native_build_profile,
};

const USAGE: &str = "usage: vize-marquette [test-run] \
                     schema|validate|canonical|fingerprint [<path.json>]\n       \
                     vize-marquette native-build schema|validate [<profile.json>]";

#[derive(Clone, Copy, PartialEq, Eq)]
enum DocumentKind {
    Application,
    TestRun,
    NativeBuild,
}

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
    let kind = match command.as_str() {
        "test-run" => DocumentKind::TestRun,
        "native-build" => DocumentKind::NativeBuild,
        _ => DocumentKind::Application,
    };
    if kind != DocumentKind::Application {
        command = arguments.next().ok_or(USAGE)?;
    }

    if command == "schema" {
        if arguments.next().is_some() {
            return Err("unexpected additional arguments".into());
        }
        print!("{}", schema(kind));
        return Ok(ExitCode::SUCCESS);
    }

    let path = arguments.next().ok_or("missing document path")?;
    if arguments.next().is_some() {
        return Err("unexpected additional arguments".into());
    }
    let bytes = fs::read(path)?;

    match kind {
        DocumentKind::Application => run_application(&command, &bytes),
        DocumentKind::TestRun => run_test_run(&command, &bytes),
        DocumentKind::NativeBuild => run_native_build(&command, &bytes),
    }
}

fn schema(kind: DocumentKind) -> &'static str {
    match kind {
        DocumentKind::Application => APPLICATION_CONTRACT_JSON_SCHEMA,
        DocumentKind::TestRun => TEST_RUN_EVIDENCE_JSON_SCHEMA,
        DocumentKind::NativeBuild => NATIVE_BUILD_PROFILE_JSON_SCHEMA,
    }
}

fn run_application(command: &str, bytes: &[u8]) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let contract: ApplicationContract = serde_json::from_slice(bytes)?;
    match command {
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

fn run_test_run(command: &str, bytes: &[u8]) -> Result<ExitCode, Box<dyn std::error::Error>> {
    let evidence: TestRunEvidence = serde_json::from_slice(bytes)?;
    match command {
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
}

fn run_native_build(command: &str, bytes: &[u8]) -> Result<ExitCode, Box<dyn std::error::Error>> {
    if command != "validate" {
        return Err(USAGE.into());
    }
    let profile: NativeBuildProfile = serde_json::from_slice(bytes)?;
    let diagnostics = validate_native_build_profile(&profile);
    println!("{}", serde_json::to_string_pretty(&diagnostics)?);
    Ok(if diagnostics.is_empty() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
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
