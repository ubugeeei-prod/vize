//! The Davinci ICE machinery (charter #30, P2-13), shared by the build path
//! and `vize repro`.
//!
//! The policy this implements: an internal compiler error **fails the file
//! and writes a reproducer** - it never degrades to possibly-wrong output
//! (charter #26 forbids auto-fallback outright), and it never takes the rest
//! of the batch down with it. The reproducer is a `repro.folio`
//! ([`ReproFolio`]): pipeline string, replay config, the recorded failure,
//! and the last-good stage dump - which, until P2-12b routes the compile
//! path through the pass manager, is the authored source itself
//! (`artifact-stage=source`).
//!
//! # Attribution
//!
//! A panic injected through the pass-manager driver ([`run_injected`]) is
//! attributed exactly: the step records the stage and pass it is entering
//! before it can panic, so the caught failure names the pass. A panic caught
//! around the real compile is **not** attributable to a pass - the real
//! stages do not run through the driver yet - and is recorded with an empty
//! `failed-pass`, rendered as `?` by [`failure_text`]. Guessing a pass there
//! would be a plausible lie, which is worse than a stated unknown.
//!
//! # Unwind builds only
//!
//! The workspace release profile sets `panic = "abort"`, so in release
//! binaries a panic still aborts the process and none of this machinery
//! runs; catching is live in every unwind build (dev, test, CI), which is
//! where TS-23 pins it. Deciding whether the shipped profile should trade
//! its abort strategy for ICE recovery is a program decision this task
//! records and does not make. The default panic printer is suppressed (in
//! unwind builds only) so a guarded failure reports through the build's
//! error channel once, not twice.

use std::path::{Path, PathBuf};
use std::sync::Once;

use vize_davinci::folio::repro::{ReproFolio, failure_text};
use vize_davinci::folio::{Folio, FolioMode};
use vize_davinci::legacy_plan;
use vize_davinci::pass::{
    BudgetObserver, Fusability, Pair, PassDesc, PassFailure, PassKind, Pipeline, Preserved,
    TimingObserver, parse_pipelines, pipeline::PipelineSpec, run_pipeline,
};
use vize_s0::{FxHashMap, String, cstr};

mod budget;
mod replay;

pub(crate) use budget::plan_budget;
pub(crate) use replay::{compile_source, replay};

/// `artifact-stage` value for an embedded authored source.
pub(crate) const ARTIFACT_STAGE_SOURCE: &str = "source";
/// `[repro.config]` key naming the backend mode (`dom`/`ssr`/`vapor`).
pub(crate) const CONFIG_MODE: &str = "mode";
/// `[repro.config]` key carrying an injected-panic pass name (TS-23).
pub(crate) const CONFIG_INJECT: &str = "inject-panic";

/// A caught pipeline failure: where it landed and what the payload said.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct IceFailure {
    pub(crate) stage: String,
    /// Empty when the panic is not attributable to a pass.
    pub(crate) pass: String,
    pub(crate) reason: String,
}

impl IceFailure {
    /// The one-line rendering every reporting surface shares.
    pub(crate) fn text(&self) -> String {
        failure_text(
            self.stage.as_str(),
            self.pass.as_str(),
            self.reason.as_str(),
        )
    }
}

/// The legacy plan the compile path runs for the selected backend, and the
/// mode name recorded in `[repro.config]`.
pub(crate) fn compile_plan(ssr: bool, vapor: bool) -> (&'static Pipeline, &'static str) {
    if ssr {
        (&legacy_plan::SSR, "ssr")
    } else if vapor {
        (&legacy_plan::VAPOR, "vapor")
    } else {
        (&legacy_plan::DOM, "dom")
    }
}

/// `(ssr, vapor)` for a recorded mode name; `None` for an unknown mode.
pub(crate) fn mode_flags(mode: &str) -> Option<(bool, bool)> {
    match mode {
        "dom" => Some((false, false)),
        "ssr" => Some((true, false)),
        "vapor" => Some((false, true)),
        _ => None,
    }
}

/// A plan rendered in the P2-2 pipeline grammar's single canonical spelling.
pub(crate) fn plan_string(plan: &Pipeline) -> String {
    let mut out = String::from(plan.stage);
    out.push('(');
    for (index, pass) in plan.passes.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str(pass.name);
    }
    out.push(')');
    out
}

/// A TS-23 injected panic: the pass it fires in and, for a content-seeded
/// crash (P3-14), the element tag whose presence in the source it needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Injection {
    pub(crate) pass: String,
    pub(crate) when: Option<String>,
}

impl Injection {
    /// Whether the injection fires for `source`: always without a trigger,
    /// otherwise only while the source holds the trigger element.
    pub(crate) fn fires_on(&self, source: &str) -> bool {
        self.when
            .as_deref()
            .is_none_or(|tag| replay::has_element(source, tag))
    }
}

/// Validate a `--davinci-inject-panic <file-stem>:<pass>[:<tag>]` spec
/// against the plan the build will run, so an injection that could never
/// fire is an argument error rather than a silently green run.
pub(crate) fn parse_inject_spec(
    spec: &str,
    plan: &Pipeline,
) -> Result<(String, Injection), String> {
    let parts: Vec<&str> = spec.split(':').collect();
    let (stem, pass, when) = match parts.as_slice() {
        [stem, pass] => (*stem, *pass, None),
        [stem, pass, tag] if !tag.is_empty() => (*stem, *pass, Some(String::from(*tag))),
        _ => return Err(cstr!("expected `<file-stem>:<pass>[:<tag>]`, got `{spec}`")),
    };
    if stem.is_empty() || pass.is_empty() {
        return Err(cstr!("expected `<file-stem>:<pass>[:<tag>]`, got `{spec}`"));
    }
    if !plan.passes.iter().any(|desc| desc.name == pass) {
        return Err(cstr!(
            "pass `{pass}` is not in the compile plan {}",
            plan_string(plan)
        ));
    }
    Ok((
        String::from(stem),
        Injection {
            pass: String::from(pass),
            when,
        },
    ))
}

/// Suppress the default panic printer for guarded runs (unwind builds only -
/// under `panic = "abort"` the printer is the only report there is).
pub(crate) fn silence_panics() {
    static ONCE: Once = Once::new();
    if cfg!(panic = "unwind") {
        ONCE.call_once(|| std::panic::set_hook(Box::new(|_| {})));
    }
}

/// Extract a panic payload's text, newline-normalized so it fits a
/// line-atomic folio scalar. Record and replay both pass through here, so
/// exact failure equality survives the normalization.
#[expect(clippy::disallowed_types, reason = "dependency API uses std String")]
pub(crate) fn panic_reason(payload: Box<dyn core::any::Any + Send>) -> String {
    let text: &str = if let Some(text) = payload.downcast_ref::<&str>() {
        text
    } else if let Some(text) = payload.downcast_ref::<std::string::String>() {
        text.as_str()
    } else {
        "non-string panic payload"
    };
    String::from(text.replace('\n', " ").as_str())
}

/// Bind parsed segments to runnable plans; every named pass is an optional,
/// fusable no-op (the `davinci-opt` shape - no catalogue until P2-9). Names
/// are leaked into the `&'static str` the const-data manager requires; both
/// callers are one-shot CLI runs.
fn build_plans(segments: &[PipelineSpec<'_>]) -> Vec<Pipeline> {
    fn leak(text: &str) -> &'static str {
        Box::leak(text.to_owned().into_boxed_str())
    }
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

/// Drive `pipeline` through the pass manager with no-op bodies, failing at
/// `inject_pass`. `Err` carries the exactly-attributed failure; `Ok` means no
/// pass by that name ran (a stale repro replayed against a renamed pass) or
/// the pipeline string does not parse.
///
/// The injected pass fails through the pass manager's error channel rather
/// than by unwinding, so the CLI never aborts on its own fault injection. The
/// reason text keeps the historical wording so recorded repros still compare
/// equal on replay.
pub(crate) fn run_injected(pipeline: &str, inject_pass: &str) -> Result<(), IceFailure> {
    let Ok(segments) = parse_pipelines(pipeline) else {
        return Ok(());
    };
    for plan in &build_plans(&segments) {
        let mut last: Option<(String, String)> = None;
        // The timing observer keeps a --profile-json build's export honest
        // about davinci-driven walks; with profiling off it costs one atomic
        // load per walk.
        let mut observers = Pair(TimingObserver::new(), BudgetObserver::new());
        let outcome = run_pipeline(plan, &mut observers, |event| {
            last = Some((
                String::from(event.pipeline.stage),
                String::from(event.desc().name),
            ));
            if event.desc().name == inject_pass {
                return Err(PassFailure::new("injected davinci failure"));
            }
            Ok(())
        });
        if outcome.is_err()
            && let Some((stage, pass)) = last
        {
            return Err(IceFailure {
                stage,
                pass,
                reason: cstr!("injected davinci panic in pass `{inject_pass}`"),
            });
        }
    }
    Ok(())
}

/// Assemble the repro for a compile-path failure whose last-good stage is
/// the authored source.
pub(crate) fn source_repro(
    plan_str: &str,
    mode: &'static str,
    inject: Option<&Injection>,
    failure: &IceFailure,
    source: String,
) -> ReproFolio {
    let mut config: FxHashMap<String, String> = FxHashMap::default();
    config.insert(String::from(CONFIG_MODE), String::from(mode));
    if let Some(injection) = inject {
        config.insert(String::from(CONFIG_INJECT), injection.pass.clone());
        if let Some(tag) = &injection.when {
            config.insert(String::from(replay::CONFIG_INJECT_WHEN), tag.clone());
        }
    }
    let mut folio = ReproFolio {
        pipeline: String::from(plan_str),
        failed_stage: failure.stage.clone(),
        failed_pass: failure.pass.clone(),
        reason: failure.reason.clone(),
        artifact_stage: String::from(ARTIFACT_STAGE_SOURCE),
        config,
        artifact: source,
    };
    folio.normalize();
    folio
}

/// Write `folio` as `{stem}.repro.folio` under `dir`, creating `dir` first
/// (the compile phase runs before the output writer creates anything).
///
/// # Errors
///
/// Returns a formatted message naming the path that failed.
pub(crate) fn write_repro(dir: &Path, stem: &str, folio: &ReproFolio) -> Result<PathBuf, String> {
    std::fs::create_dir_all(dir)
        .map_err(|error| cstr!("cannot create {}: {error}", dir.display()))?;
    let path = dir.join(cstr!("{stem}.repro.folio").as_str());
    std::fs::write(&path, folio.print_to_string(FolioMode::Full).as_bytes())
        .map_err(|error| cstr!("cannot write {}: {error}", path.display()))?;
    Ok(path)
}
