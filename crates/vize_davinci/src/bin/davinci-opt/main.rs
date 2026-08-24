//! `davinci-opt` - the Davinci stage-dump driver.
//!
//! Two modes, exactly one of which must be selected:
//!
//! ```text
//! davinci-opt --roundtrip <file> [--stage croquis]
//! davinci-opt --pipeline "<syntax>" [--stage <stage>]
//!             [--folio-dir <dir> [--folio-after-change]]
//!             [--timing-json <path>] < folio
//! ```
//!
//! `--roundtrip` reads `<file>`, parses it with the selected stage's folio,
//! prints it back in `Full` mode, and byte-compares against the input. Exit
//! code 0 means the file is canonical (round-trip identity); 1 means a parse
//! failure or a byte difference; 2 means a usage error.
//!
//! `--pipeline` reads a folio from **stdin**, runs the parsed pipeline plan
//! over it, and prints the resulting folio in `Full` mode to **stdout** (the
//! TS-17 snapshot surface); a `walks=/passes=` summary goes to stderr. The
//! syntax and its exact rejection messages live in
//! `vize_davinci::pass::pipeline`; a malformed string is a usage error (exit
//! 2). Until P2-9 lands a pass catalogue there is no name -> implementation
//! binding, so every named pass runs as an optional, fusable no-op -
//! binding names to descriptions is the caller's step by design
//! (`pass/pipeline.rs`), and this caller has nothing to bind to yet. Exit
//! codes: 0 = ran, 1 = stdin unreadable or folio parse failure, 2 = usage.
//!
//! Three pipeline-mode extras (P2-13): `--folio-dir <dir>` writes the
//! artifact's canonical folio after every pass into `<dir>` (page names
//! `{seq:03}-{stage}.{pass}.folio`), `--folio-after-change` hash-gates that
//! to the passes whose artifact actually changed (with today's no-op bodies:
//! none), and `--timing-json <path>` writes the run's profile export - the
//! per-walk spans the timing observer records - against the P0-11 schema
//! (`davinci-road/plan/profile-export.schema.json`). Since P2-18 the folio
//! directory also carries its Spolvero feed (`spolvero.json`): the same
//! pages as one schema-versioned JSON document
//! (`davinci-road/plan/spolvero-feed.schema.json`), written even when the
//! hash gate emitted zero pages.
//!
//! Stages: `croquis` (default, the P0-10 page) and `budget-observer` (the
//! first `#[derive(Folio)]` page, P2-4). The binary is host-side and may use
//! `std`; the `vize_davinci` library stays `no_std + alloc`.

mod args;
mod export;

use std::process::ExitCode;

use args::{Args, Mode, parse_args};

use vize_carton::String;
use vize_davinci::folio::dump::FolioDump;
use vize_davinci::folio::{Folio, FolioMode, croquis::CroquisFolio};
use vize_davinci::pass::{
    BudgetObserver, Fusability, Pair, PassDesc, PassKind, Pipeline, Preserved, TimingObserver,
    parse_pipelines, pipeline::PipelineSpec, print_pipelines, run_pipeline,
};

const USAGE: &str = "usage: davinci-opt --roundtrip <file> [--stage croquis]\n       davinci-opt --pipeline \"<syntax>\" [--stage <stage>] [--folio-dir <dir> [--folio-after-change]] [--timing-json <path>] < folio";

/// The stage list every stage-related message reports, alphabetical.
const STAGES: &str = "budget-observer, croquis";

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

/// Why a stage could not reprint the input.
enum StageError {
    Parse(vize_davinci::folio::FolioError),
    UnknownStage,
}

/// Parse `input` with `stage`'s folio and print it back canonically.
fn stage_reprint(stage: &str, input: &str) -> Result<String, StageError> {
    fn reprint<T: Folio>(input: &str) -> Result<String, StageError> {
        T::parse(input)
            .map(|folio| folio.print_to_string(FolioMode::Full))
            .map_err(StageError::Parse)
    }
    match stage {
        "croquis" => reprint::<CroquisFolio>(input),
        "budget-observer" => reprint::<BudgetObserver>(input),
        _ => Err(StageError::UnknownStage),
    }
}

fn run_roundtrip(file: &std::path::Path, stage: &str) -> ExitCode {
    let path = file.display();
    let input = match std::fs::read_to_string(file) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("davinci-opt: cannot read {path}: {error}");
            return ExitCode::from(1);
        }
    };
    let printed = match stage_reprint(stage, &input) {
        Ok(printed) => printed,
        Err(StageError::Parse(error)) => {
            eprintln!("davinci-opt: {path}: {error}");
            return ExitCode::from(1);
        }
        Err(StageError::UnknownStage) => {
            eprintln!("davinci-opt: unknown stage: {stage} (available: {STAGES})");
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

/// Leak a parsed name into the `&'static str` the const-data pass manager
/// requires. Fine in a short-lived driver binary; never do this in the
/// library.
fn leak(text: &str) -> &'static str {
    Box::leak(text.to_owned().into_boxed_str())
}

/// Bind parsed segments to runnable plans: every named pass is an optional,
/// fusable no-op until P2-9's catalogue exists (see the module docs).
fn build_plans(segments: &[PipelineSpec<'_>]) -> Vec<Pipeline> {
    segments
        .iter()
        .map(|segment| {
            let passes: Vec<PassDesc> = segment
                .passes
                .iter()
                .map(|name| {
                    PassDesc::new(
                        leak(name),
                        PassKind::Optional,
                        Fusability::Fusable,
                        Preserved::ALL,
                    )
                })
                .collect();
            Pipeline::new(leak(segment.stage), passes.leak())
        })
        .collect()
}

fn run_pipeline_mode(syntax: &str, args: &Args) -> ExitCode {
    let stage = args.stage.as_str();
    let segments = match parse_pipelines(syntax) {
        Ok(segments) => segments,
        Err(error) => {
            eprintln!("davinci-opt: --pipeline: {error}");
            return ExitCode::from(2);
        }
    };
    let mut bytes = Vec::new();
    if let Err(error) = std::io::Read::read_to_end(&mut std::io::stdin(), &mut bytes) {
        eprintln!("davinci-opt: cannot read stdin: {error}");
        return ExitCode::from(1);
    }
    let Ok(input) = core::str::from_utf8(&bytes) else {
        eprintln!("davinci-opt: stdin is not valid UTF-8");
        return ExitCode::from(1);
    };
    // Parse the artifact, run the plans over it (no-op bodies, so the
    // artifact is unchanged - what --pipeline exercises today is the parse,
    // the grouping and the driver), and print the result canonically.
    let printed = match stage_reprint(stage, input) {
        Ok(printed) => printed,
        Err(StageError::Parse(error)) => {
            eprintln!("davinci-opt: <stdin>: {error}");
            return ExitCode::from(1);
        }
        Err(StageError::UnknownStage) => {
            eprintln!("davinci-opt: unknown stage: {stage} (available: {STAGES})");
            return ExitCode::from(2);
        }
    };
    // The timing observer records one span per walk into the global profiler
    // (P2-3); the profiler is enabled only when the export was asked for, so
    // without --timing-json the observer costs one atomic load per walk.
    if args.timing_json.is_some() {
        let profiler = vize_carton::profiler::global_profiler();
        profiler.clear();
        profiler.enable();
    }
    let mut dump = args
        .folio_dir
        .as_ref()
        .map(|_| FolioDump::new(args.folio_after_change));
    if let Some(dump) = dump.as_mut() {
        dump.seed(printed.as_str());
    }
    let mut observers = Pair(TimingObserver::new(), BudgetObserver::new());
    for plan in build_plans(&segments) {
        run_pipeline(&plan, &mut observers, |event| {
            if let Some(dump) = dump.as_mut() {
                dump.after_pass(event, printed.as_str());
            }
            Ok(())
        })
        .expect("a no-op pass body cannot fail");
    }
    eprintln!(
        "davinci-opt: pipeline {}: walks={} passes={}",
        print_pipelines(&segments),
        observers.1.walks,
        observers.1.passes,
    );
    if let (Some(dir), Some(dump)) = (args.folio_dir.as_deref(), dump.as_ref()) {
        if let Err(message) = export::write_dump(dir, dump) {
            eprintln!("davinci-opt: {message}");
            return ExitCode::from(1);
        }
        eprintln!(
            "davinci-opt: folio-dir {}: {} page(s)",
            dir.display(),
            dump.pages.len(),
        );
    }
    if let Some(path) = args.timing_json.as_deref() {
        let result = export::write_timing(path);
        vize_carton::profiler::global_profiler().disable();
        if let Err(message) = result {
            eprintln!("davinci-opt: {message}");
            return ExitCode::from(1);
        }
    }
    print!("{printed}");
    ExitCode::SUCCESS
}

fn main() -> ExitCode {
    match parse_args() {
        Ok(args) => match &args.mode {
            Mode::Roundtrip(path) => run_roundtrip(path, args.stage.as_str()),
            Mode::Pipeline(syntax) => run_pipeline_mode(syntax.as_str(), &args),
        },
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
