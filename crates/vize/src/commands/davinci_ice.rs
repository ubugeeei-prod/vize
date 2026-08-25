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

use std::cell::RefCell;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::Once;

use vize_davinci::folio::repro::{ReproFolio, failure_text};
use vize_davinci::folio::{Folio, FolioMode};
use vize_davinci::legacy_plan;
use vize_davinci::pass::{
    BudgetObserver, Fusability, Pair, PassDesc, PassKind, Pipeline, Preserved, TimingObserver,
    parse_pipelines, pipeline::PipelineSpec, run_pipeline,
};
use vize_s0::{FxHashMap, String, cstr};

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

/// Validate a `--davinci-inject-panic <file-stem>:<pass>` spec against the
/// plan the build will run, so an injection that could never fire is an
/// argument error rather than a silently green run.
pub(crate) fn parse_inject_spec(spec: &str, plan: &Pipeline) -> Result<(String, String), String> {
    let split = spec.split_once(':');
    let Some((stem, pass)) = split.filter(|(stem, pass)| !stem.is_empty() && !pass.is_empty())
    else {
        return Err(cstr!("expected `<file-stem>:<pass>`, got `{spec}`"));
    };
    if !plan.passes.iter().any(|desc| desc.name == pass) {
        return Err(cstr!(
            "pass `{pass}` is not in the compile plan {}",
            plan_string(plan)
        ));
    }
    Ok((String::from(stem), String::from(pass)))
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
#[allow(clippy::disallowed_types)]
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

/// Drive `pipeline` through the pass manager with no-op bodies, panicking at
/// `inject_pass`, and catch the unwind. `Err` carries the exactly-attributed
/// failure; `Ok` means no pass by that name ran (a stale repro replayed
/// against a renamed pass).
pub(crate) fn run_injected(pipeline: &str, inject_pass: &str) -> Result<(), IceFailure> {
    silence_panics();
    let segments =
        parse_pipelines(pipeline).expect("repro pipeline strings are validated before this call");
    let plans = build_plans(&segments);
    let last: RefCell<Option<(String, String)>> = RefCell::new(None);
    let outcome = catch_unwind(AssertUnwindSafe(|| {
        for plan in &plans {
            // The timing observer keeps a --profile-json build's export
            // honest about davinci-driven walks; with profiling off it costs
            // one atomic load per walk.
            let mut observers = Pair(TimingObserver::new(), BudgetObserver::new());
            run_pipeline(plan, &mut observers, |event| {
                *last.borrow_mut() = Some((
                    String::from(event.pipeline.stage),
                    String::from(event.desc().name),
                ));
                if event.desc().name == inject_pass {
                    panic!("injected davinci panic in pass `{inject_pass}`");
                }
                Ok(())
            })
            .expect("injected panics unwind; no-op bodies cannot fail");
        }
    }));
    match outcome {
        Ok(()) => Ok(()),
        Err(payload) => {
            let (stage, pass) = last
                .into_inner()
                .expect("a panic inside run_pipeline recorded the pass it entered");
            Err(IceFailure {
                stage,
                pass,
                reason: panic_reason(payload),
            })
        }
    }
}

/// Assemble the repro for a compile-path failure whose last-good stage is
/// the authored source.
pub(crate) fn source_repro(
    plan_str: &str,
    mode: &'static str,
    inject: Option<&str>,
    failure: &IceFailure,
    source: String,
) -> ReproFolio {
    let mut config: FxHashMap<String, String> = FxHashMap::default();
    config.insert(String::from(CONFIG_MODE), String::from(mode));
    if let Some(pass) = inject {
        config.insert(String::from(CONFIG_INJECT), String::from(pass));
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

/// Replay a parsed repro: `Ok(Some(_))` reproduced a failure, `Ok(None)`
/// completed without one, `Err` means this repro cannot be replayed at all.
pub(crate) fn replay(folio: &ReproFolio) -> Result<Option<IceFailure>, String> {
    if let Some(pass) = folio.config.get(CONFIG_INJECT) {
        return Ok(run_injected(folio.pipeline.as_str(), pass.as_str()).err());
    }
    if folio.artifact_stage.as_str() != ARTIFACT_STAGE_SOURCE {
        return Err(cstr!(
            "cannot replay artifact stage `{}`; only `{ARTIFACT_STAGE_SOURCE}` replays today",
            folio.artifact_stage
        ));
    }
    let mode = folio.config.get(CONFIG_MODE).map_or("dom", |m| m.as_str());
    let Some((ssr, vapor)) = mode_flags(mode) else {
        return Err(cstr!("unknown mode `{mode}` in [repro.config]"));
    };
    silence_panics();
    let segments = parse_pipelines(folio.pipeline.as_str())
        .expect("repro pipeline strings are validated at folio parse time");
    let stage = segments.first().map_or("", |segment| segment.stage);
    match catch_unwind(AssertUnwindSafe(|| {
        compile_source(folio.artifact.as_str(), ssr, vapor);
    })) {
        Ok(()) => Ok(None),
        // The same attribution rule the record side used for a real-compile
        // panic: the plan's stage, no pass.
        Err(payload) => Ok(Some(IceFailure {
            stage: String::from(stage),
            pass: String::default(),
            reason: panic_reason(payload),
        })),
    }
}

/// Compile an embedded source with the recorded mode's defaults. Diagnostics
/// are irrelevant to a replay - only a panic matters - so results and errors
/// are discarded alike.
fn compile_source(source: &str, ssr: bool, vapor: bool) {
    use vize_atelier_core::{CodegenOptions, options::CustomElementMatcher};
    use vize_atelier_sfc::{
        ScriptCompileOptions, SfcCompileOptions, SfcParseOptions, StyleCompileOptions,
        TemplateCompileOptions,
        compile_sfc_with_custom_elements_template_syntax_and_codegen_options, parse_sfc,
    };

    let parse_options = || SfcParseOptions {
        filename: "repro.vue".into(),
        ..Default::default()
    };
    let Ok(descriptor) = parse_sfc(source, parse_options()) else {
        return;
    };
    let has_scoped = descriptor.styles.iter().any(|style| style.scoped);
    let options = SfcCompileOptions {
        parse: parse_options(),
        script: ScriptCompileOptions::default(),
        template: TemplateCompileOptions {
            id: Some("repro.vue".into()),
            scoped: has_scoped,
            ssr,
            ..Default::default()
        },
        style: StyleCompileOptions {
            id: "repro.vue".into(),
            scoped: has_scoped,
            ..Default::default()
        },
        vapor,
        scope_id: None,
    };
    let _ = compile_sfc_with_custom_elements_template_syntax_and_codegen_options(
        &descriptor,
        options,
        vize_atelier_core::TemplateSyntaxMode::Standard,
        CustomElementMatcher::from_patterns(Vec::new()),
        CodegenOptions::default(),
    );
}
