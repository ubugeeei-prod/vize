//! `vize reduce` - shrink a failing SFC or crash repro to a minimal one that
//! still fails the same way (Davinci P3-14, `folio-reduce`).
//!
//! llvm-reduce's model: a dumb deterministic driver ([`driver`]), an
//! IR-aware deletion vocabulary ([`vocabulary`]: S1 subtrees, always
//! re-printable), and a sovereign interestingness predicate ([`oracle`]),
//! composed by listing checks.
//!
//! - A `*.repro.folio` input (the ICE policy's reproducer, charter #30) is
//!   reduced on its `source` artifact while it keeps replaying to **exactly**
//!   the recorded failure; the output is a repro folio `vize repro` accepts.
//! - A `.vue` input needs at least one check flag.
//!
//! The reduced artifact goes to `--out` (or stdout); one summary line goes
//! to stderr. Exit codes: 0 = reduced (or already minimal), 1 = the input
//! does not satisfy the oracle, 2 = usage, unreadable, or unreplayable input.

mod driver;
mod oracle;
mod vocabulary;

#[cfg(test)]
mod tests;

use std::path::{Path, PathBuf};

use clap::Args;
use vize_davinci::folio::repro::ReproFolio;
use vize_davinci::folio::{Folio, FolioMode};
use vize_davinci::pass::RemarkKind;
use vize_s0::{String, cstr};

use super::davinci_ice::{self, ARTIFACT_STAGE_SOURCE, IceFailure};
use driver::{ReduceError, reduce};
use oracle::Check;

#[allow(clippy::disallowed_types)]
#[derive(Args, Default)]
pub struct ReduceArgs {
    /// A `.vue` file or a `repro.folio` written by a failed build
    pub input: PathBuf,

    /// Write the reduced artifact here instead of stdout
    #[arg(long, value_name = "PATH")]
    pub out: Option<PathBuf>,

    /// Keep inputs whose compile panics (an internal compiler error)
    #[arg(long)]
    pub crash: bool,

    /// Keep inputs emitting this remark: "<stage.pass> <kind> <name>"
    #[arg(long, value_name = "REMARK")]
    pub remark: Vec<std::string::String>,

    /// Keep inputs whose post-transform S2 folio contains this text
    #[arg(long, value_name = "TEXT")]
    pub folio_contains: Vec<std::string::String>,

    /// Keep inputs with an S1->S2 diagnostic whose message contains this text
    #[arg(long, value_name = "TEXT")]
    pub diagnostic: Vec<std::string::String>,

    /// Keep inputs whose S2 transform pipeline walks more than N times
    #[arg(long, value_name = "N")]
    pub walks_over: Option<u32>,

    /// Keep inputs for which `sh -c "<CMD> <file>"` exits 0
    #[arg(long, value_name = "CMD")]
    pub script: Option<std::string::String>,

    /// Stop after this many oracle evaluations
    #[arg(long, value_name = "N", default_value = "10000")]
    pub max_runs: usize,
}

fn fail(code: i32, message: &str) -> ! {
    eprintln!("reduce: {message}");
    std::process::exit(code);
}

fn parse_remark(spec: &str) -> Result<Check, String> {
    let parts: Vec<&str> = spec.split_whitespace().collect();
    let [origin, kind, name] = parts.as_slice() else {
        return Err(cstr!(
            "--remark expects \"<stage.pass> <kind> <name>\", got `{spec}`"
        ));
    };
    let kind = RemarkKind::from_name(kind)
        .ok_or_else(|| cstr!("--remark kind `{kind}` is not applied, missed, or analysis"))?;
    if !origin.contains('.') {
        return Err(cstr!("--remark origin `{origin}` is not `stage.pass`"));
    }
    Ok(Check::Remark {
        origin: String::from(*origin),
        kind,
        name: String::from(*name),
    })
}

/// The flag-selected checks, in flag order.
fn flag_checks(args: &ReduceArgs, scratch: &Path) -> Result<Vec<Check>, String> {
    let mut checks = Vec::new();
    if args.crash {
        checks.push(Check::Crash);
    }
    for spec in &args.remark {
        checks.push(parse_remark(spec)?);
    }
    for text in &args.folio_contains {
        checks.push(Check::FolioContains(String::from(text.as_str())));
    }
    for text in &args.diagnostic {
        checks.push(Check::Diagnostic(String::from(text.as_str())));
    }
    if let Some(limit) = args.walks_over {
        checks.push(Check::WalksOver(limit));
    }
    if let Some(command) = &args.script {
        checks.push(Check::Script {
            command: String::from(command.as_str()),
            file: scratch.to_path_buf(),
        });
    }
    Ok(checks)
}

/// Where a script check writes candidates: beside nothing the user owns,
/// keeping the input's file name so the script sees a `.vue`.
fn scratch_file(input: &Path) -> PathBuf {
    let name = input.file_name().map_or_else(
        || std::ffi::OsString::from("candidate.vue"),
        std::ffi::OsStr::to_os_string,
    );
    let dir = std::env::temp_dir().join(cstr!("vize-reduce-{}", std::process::id()).as_str());
    let _ = std::fs::create_dir_all(&dir);
    dir.join(name)
}

pub fn run(args: ReduceArgs) {
    let path = args.input.as_path();
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| fail(2, &cstr!("cannot read {}: {error}", path.display())));
    let scratch = scratch_file(path);
    let mut checks = flag_checks(&args, &scratch).unwrap_or_else(|message| fail(2, &message));
    let repro = path.to_string_lossy().ends_with(".folio");
    let (artifact, repro_folio) = if repro {
        let folio = ReproFolio::parse(&text)
            .unwrap_or_else(|error| fail(2, &cstr!("{}: {error}", path.display())));
        if folio.artifact_stage.as_str() != ARTIFACT_STAGE_SOURCE {
            fail(
                2,
                &cstr!(
                    "cannot reduce artifact stage `{}`; only `{ARTIFACT_STAGE_SOURCE}` reduces today",
                    folio.artifact_stage
                ),
            );
        }
        let failure = IceFailure {
            stage: folio.failed_stage.clone(),
            pass: folio.failed_pass.clone(),
            reason: folio.reason.clone(),
        };
        checks.insert(
            0,
            Check::Reproduces {
                repro: folio.clone(),
                failure,
            },
        );
        (folio.artifact.clone(), Some(folio))
    } else {
        if checks.is_empty() {
            fail(
                2,
                "a .vue input needs at least one check (--crash, --remark, --folio-contains, \
                 --diagnostic, --walks-over, --script)",
            );
        }
        (String::from(text.as_str()), None)
    };
    davinci_ice::silence_panics();
    let outcome = reduce(artifact.as_str(), args.max_runs, &mut |candidate| {
        oracle::interesting(&checks, candidate)
    });
    let _ = std::fs::remove_dir_all(scratch.parent().unwrap_or(&scratch));
    let reduction = match outcome {
        Ok(reduction) => reduction,
        Err(ReduceError::NotInteresting) => fail(
            1,
            "the input does not satisfy the oracle; nothing to preserve",
        ),
    };
    let output = match repro_folio {
        Some(mut folio) => {
            folio.artifact = reduction.text.clone();
            folio.normalize();
            folio.print_to_string(FolioMode::Full)
        }
        None => reduction.text.clone(),
    };
    match args.out.as_deref() {
        Some(out) => std::fs::write(out, output.as_bytes())
            .unwrap_or_else(|error| fail(2, &cstr!("cannot write {}: {error}", out.display()))),
        None => print!("{output}"),
    }
    let before = artifact.len();
    let after = reduction.text.len();
    eprintln!(
        "reduce: {before} -> {after} bytes ({}.{}% of the input), {} oracle run(s){}",
        after * 100 / before.max(1),
        (after * 1000 / before.max(1)) % 10,
        reduction.runs,
        if reduction.exhausted {
            ", run budget exhausted"
        } else {
            ", 1-minimal"
        }
    );
}
