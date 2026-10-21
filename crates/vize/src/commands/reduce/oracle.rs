//! The composable interestingness oracles (P3-14): diagnostics, remarks,
//! folio content, budget breaches, crashes, and the llvm-reduce escape
//! hatch — an external script. A candidate is interesting when **every**
//! configured check holds (conjunction), so oracles compose by listing.
//!
//! The L2-based checks share one run per candidate: the SFC's inline HTML
//! template is parsed (L1), lowered, and taken through the L2 transform
//! pipeline under `Pair(BudgetObserver, RemarkCollector)` — the same channel
//! the TS-32 corpus and the Spolvero feed read, so "the remark the feed
//! shows" and "the remark the reducer preserves" cannot disagree.

use std::process::{Command, Stdio};

use vize_atelier_dom::{DomCompilerOptions, compile_template_legacy_with_options};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_curator::repro::Page as ReproPage;
use vize_l0::dump::{Dump, Mode as DumpMode};
use vize_l0::pass::{BudgetObserver, Pair, RemarkCollector, RemarkKind};
use vize_l0::stage::pipeline_wire_id;
use vize_l0::{Allocator, String, cstr};

use crate::commands::davinci_ice::{self, IceFailure};

/// One interestingness check.
#[derive(Debug, Clone)]
pub(crate) enum Check {
    /// The repro, with the candidate as its artifact, replays to exactly
    /// the recorded failure (stage, pass and reason byte-equal).
    Reproduces {
        repro: ReproPage,
        failure: IceFailure,
    },
    /// Compiling the candidate panics (any internal compiler error).
    Crash,
    /// Some remark has this origin (`stage.pass`), kind, and name.
    Remark {
        origin: String,
        kind: RemarkKind,
        name: String,
    },
    /// The post-transform L2 folio contains this text.
    FolioContains(String),
    /// Some L1→L2 diagnostic's message contains this text.
    Diagnostic(String),
    /// The L2 transform pipeline walked the tree more than this many times.
    WalksOver(u32),
    /// The canonical default legacy and Davinci DOM lanes emit different code.
    DomDivergence,
    /// `sh -c "<command> <file>"` exits 0 with the candidate written to
    /// `file` — the sovereign interestingness script.
    Script {
        command: String,
        file: std::path::PathBuf,
    },
}

/// One L2 run's observable products.
struct L2Run {
    folio: String,
    remarks: Vec<vize_l0::pass::observer::RecordedRemark>,
    diagnostics: Vec<String>,
    walks: u32,
}

fn l2_run(source: &str) -> Option<L2Run> {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).ok()?;
    let template = descriptor.template.as_ref()?;
    let html = template.lang.as_deref().is_none_or(|lang| lang == "html");
    if template.src.is_some() || !html {
        return None;
    }
    let allocator = Allocator::default();
    let (tree, errors) = vize_l1::markup::parse(&allocator, &template.content);
    let mut lowered =
        vize_l1_to_l2::lower_with_caps(&allocator, &tree, &errors, vize_l1_to_l2::LegacyCaps::VUE3);
    let mut observers = Pair(BudgetObserver::new(), RemarkCollector::new());
    let _facts = vize_l1_to_l2::pass::run_transform(&mut lowered, &mut observers);
    let Pair(budget, remarks) = observers;
    Some(L2Run {
        folio: vize_l2::dump::Page::of(&lowered.root.ops).print_to_string(DumpMode::Full),
        remarks: remarks.finish(),
        diagnostics: lowered
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.clone())
            .collect(),
        walks: budget.walks,
    })
}

fn crashes(source: &str) -> bool {
    davinci_ice::silence_panics();
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        davinci_ice::compile_source(source, false, false);
    }))
    .is_err()
}

fn outputs_differ(old: &str, new: &str) -> bool {
    old != new
}

/// The same default-lane comparison used by the canonical DOM corpus sweep.
/// Inputs outside that sweep's ordinary comparable boundary are not
/// interesting, so a reducer cannot silently turn a divergence into a skip.
fn dom_diverges(source: &str) -> bool {
    let Ok(descriptor) = parse_sfc(source, SfcParseOptions::default()) else {
        return false;
    };
    let Some(template) = descriptor.template.as_ref() else {
        return false;
    };

    let old_allocator = Allocator::new();
    let (_, errors, old) = compile_template_legacy_with_options(
        &old_allocator,
        &template.content,
        DomCompilerOptions::default(),
    );
    if errors.iter().any(|error| !error.is_recoverable()) {
        return false;
    }
    let mut old_output = old.preamble.clone();
    old_output.push('\n');
    old_output.push_str(&old.code);

    let new_allocator = Allocator::new();
    let Ok(new_output) = vize_l1_to_l2::emit_dom_source_patch_facts_observed_with_options(
        &new_allocator,
        &template.content,
        vize_l1_to_l2::LegacyCaps::VUE3,
        &vize_l1_to_l2::DomEmitOptions::DEFAULT,
    )
    else {
        return false;
    };
    let new_output = new_output.emit.assembled();
    outputs_differ(old_output.as_str(), new_output.as_str())
}

fn script_holds(command: &str, file: &std::path::Path, candidate: &str) -> bool {
    if std::fs::write(file, candidate.as_bytes()).is_err() {
        return false;
    }
    Command::new("sh")
        .arg("-c")
        .arg(cstr!("{command} \"$1\"").as_str())
        .arg("vize-reduce")
        .arg(file)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

impl Check {
    fn needs_l2(&self) -> bool {
        matches!(
            self,
            Self::Remark { .. } | Self::FolioContains(_) | Self::Diagnostic(_) | Self::WalksOver(_)
        )
    }
}

/// Whether `candidate` satisfies every check.
pub(crate) fn interesting(checks: &[Check], candidate: &str) -> bool {
    let s2 = checks
        .iter()
        .any(Check::needs_l2)
        .then(|| l2_run(candidate))
        .flatten();
    for check in checks {
        let holds = match check {
            Check::Reproduces { repro, failure } => {
                let mut replayed = repro.clone();
                replayed.artifact = String::from(candidate);
                replayed.normalize();
                matches!(davinci_ice::replay(&replayed), Ok(Some(found)) if found == *failure)
            }
            Check::Crash => crashes(candidate),
            Check::Remark { origin, kind, name } => s2.as_ref().is_some_and(|s2| {
                s2.remarks.iter().any(|remark| {
                    remark.kind == *kind
                        && remark.name == *name
                        && origin
                            .split_once('.')
                            .map(|(stage, pass)| (pipeline_wire_id(stage), pass))
                            == Some((remark.stage.as_str(), remark.pass.as_str()))
                })
            }),
            Check::FolioContains(text) => s2
                .as_ref()
                .is_some_and(|s2| s2.folio.contains(text.as_str())),
            Check::Diagnostic(text) => s2.as_ref().is_some_and(|s2| {
                s2.diagnostics
                    .iter()
                    .any(|message| message.contains(text.as_str()))
            }),
            Check::WalksOver(limit) => s2.as_ref().is_some_and(|s2| s2.walks > *limit),
            Check::DomDivergence => dom_diverges(candidate),
            Check::Script { command, file } => script_holds(command, file, candidate),
        };
        if !holds {
            return false;
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::outputs_differ;

    #[test]
    fn dom_divergence_requires_byte_different_outputs() {
        assert!(!outputs_differ("same", "same"));
        assert!(outputs_differ("legacy", "davinci"));
    }
}
