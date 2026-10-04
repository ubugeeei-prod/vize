//! Complete actual results/metadata and independent original-input edit queries.

use serde::Serialize;
use vize_l0::String;
use vize_patina::{LintResult, Linter, native::template::NativeTemplateLintRefusal};

use super::case::{Case, configured};

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "each actual offered fix remains in the complete capture"
)]
struct Application {
    diagnostic_index: usize,
    source: String,
    result: LintResult,
}

#[derive(Debug)]
#[expect(
    dead_code,
    reason = "all results remain unsorted and unfiltered in the complete capture"
)]
struct Observation {
    initial: LintResult,
    applications: Vec<Application>,
    unchanged_requery: Option<LintResult>,
}

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "lowercase")]
enum NativeOutcome {
    Handled { observation: String },
    Refused { context: String, refusal: Refusal },
}

#[derive(Serialize)]
struct Refusal {
    kind: &'static str,
    detail: std::string::String,
    unprovided_rule: Option<String>,
}

fn query(
    native: bool,
    linter: &Linter,
    case: &Case,
    source: &str,
) -> Result<LintResult, NativeTemplateLintRefusal> {
    if native {
        linter.lint_native_template(source, &case.filename)
    } else {
        Ok(linter.lint_template(source, &case.filename))
    }
}

fn observe(
    native: bool,
    linter: &Linter,
    case: &Case,
) -> Result<Observation, NativeTemplateLintRefusal> {
    let initial = query(native, linter, case, &case.source)?;
    // The old warning-count assertion is witness metadata, never a full oracle.
    // Even unexpected diagnostics/fixes must remain available for review.
    let mut applications = Vec::new();
    for (diagnostic_index, diagnostic) in initial.diagnostics.iter().enumerate() {
        if let Some(fix) = &diagnostic.fix {
            let source = fix.apply(&case.source);
            let result = query(native, linter, case, &source)?;
            applications.push(Application {
                diagnostic_index,
                source,
                result,
            });
        }
    }
    let unchanged_requery = if applications.is_empty() {
        Some(query(native, linter, case, &case.source)?)
    } else {
        None
    };
    Ok(Observation {
        initial,
        applications,
        unchanged_requery,
    })
}

fn kind(refusal: &NativeTemplateLintRefusal) -> &'static str {
    match refusal {
        NativeTemplateLintRefusal::Source(_) => "Source",
        NativeTemplateLintRefusal::Parse(_) => "Parse",
        NativeTemplateLintRefusal::UnsupportedVueVersion { .. } => "UnsupportedVueVersion",
        NativeTemplateLintRefusal::UnsupportedVaporMode { .. } => "UnsupportedVaporMode",
        NativeTemplateLintRefusal::UnprovidedRule { .. } => "UnprovidedRule",
        NativeTemplateLintRefusal::Recovered { .. } => "Recovered",
        NativeTemplateLintRefusal::UnsupportedComponent => "UnsupportedComponent",
        NativeTemplateLintRefusal::SourceMismatch => "SourceMismatch",
        NativeTemplateLintRefusal::Header(_) => "Header",
        NativeTemplateLintRefusal::UnsupportedContext { .. } => "UnsupportedContext",
        NativeTemplateLintRefusal::UnsupportedAttribute { .. } => "UnsupportedAttribute",
        NativeTemplateLintRefusal::UnsupportedText { .. } => "UnsupportedText",
        NativeTemplateLintRefusal::Comment { .. } => "Comment",
        NativeTemplateLintRefusal::Interpolation { .. } => "Interpolation",
        NativeTemplateLintRefusal::UnexpectedChild { .. } => "UnexpectedChild",
    }
}

pub(super) fn capture(api: &str, input: &str) -> Result<String, serde_json::Error> {
    let case: Case = serde_json::from_str(input)?;
    let (linter, identity) = configured(&case);
    let context = format!("{case:#?}\n{identity:#?}\n");
    let native = api == "--native";
    match observe(native, &linter, &case) {
        Ok(observation) => {
            let output = format!("{context}{observation:#?}\n");
            if !native {
                return Ok(output.into());
            }
            let outcome = NativeOutcome::Handled {
                observation: output.into(),
            };
            Ok(format!("{}\n", serde_json::to_string(&outcome)?).into())
        }
        Err(refusal) => {
            let unprovided_rule = match &refusal {
                NativeTemplateLintRefusal::UnprovidedRule { rule } => Some(rule.clone()),
                _ => None,
            };
            let outcome = NativeOutcome::Refused {
                context: context.into(),
                refusal: Refusal {
                    kind: kind(&refusal),
                    detail: format!("{refusal:?}"),
                    unprovided_rule,
                },
            };
            Ok(format!("{}\n", serde_json::to_string(&outcome)?).into())
        }
    }
}
