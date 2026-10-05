//! Complete actual native observations or explicit API/entry/driver refusals.

use serde::Serialize;
use vize_l0::String;
use vize_patina::{
    HelpLevel, LintPreset, Linter, Locale, native::template::NativeTemplateLintRefusal,
};

use super::current_api::{Application, Case, Observation, configured};

mod refusal;
use refusal::QueryRefusal;

#[derive(Serialize)]
#[serde(tag = "state", rename_all = "lowercase")]
enum Outcome {
    Handled { observation: String },
    Unsupported { reason: Reason },
}

#[derive(Serialize)]
struct Reason {
    api: std::string::String,
    entry: Option<String>,
    kind: &'static str,
    detail: std::string::String,
}

fn reason(
    api: &str,
    entry: Option<&str>,
    kind: &'static str,
    detail: std::string::String,
) -> Outcome {
    Outcome::Unsupported {
        reason: Reason {
            api: api.to_owned(),
            entry: entry.map(Into::into),
            kind,
            detail,
        },
    }
}

fn query(
    linter: &Linter,
    case: &Case,
    source: &str,
) -> Result<vize_patina::LintResult, QueryRefusal> {
    match case.entry.as_str() {
        "template" => linter
            .lint_native_template(source, &case.filename)
            .map_err(QueryRefusal::Template),
        "sfc" => linter
            .lint_native_sfc(source, &case.filename)
            .map_err(QueryRefusal::Sfc),
        _ => unreachable!("unprovided entry refused before original query"),
    }
}

fn observe(linter: &Linter, case: &Case) -> Result<Observation, QueryRefusal> {
    let initial = query(linter, case, &case.source)?;
    assert_eq!(initial.diagnostics.len(), case.diagnostics, "{}", case.id);
    let mut applications = Vec::new();
    for (diagnostic_index, diagnostic) in initial.diagnostics.iter().enumerate() {
        if let Some(fix) = &diagnostic.fix {
            // Every genuine offered edit independently uses the original bytes.
            let source = fix.apply(&case.source);
            let result = query(linter, case, &source)?;
            applications.push(Application {
                diagnostic_index,
                source,
                result,
            });
        }
    }
    assert_eq!(applications.len(), case.fixes, "{}", case.id);
    let unchanged_requery = if applications.is_empty() {
        Some(query(linter, case, &case.source)?)
    } else {
        None
    };
    Ok(Observation {
        initial,
        applications,
        unchanged_requery,
    })
}

fn current(api: &str, input: &str) -> Result<Outcome, serde_json::Error> {
    let case: Case = serde_json::from_str(input)?;
    assert_eq!(case.history.len(), 40, "full historical commit identity");
    if !matches!(case.entry.as_str(), "template" | "sfc") {
        return Ok(reason(
            api,
            Some(&case.entry),
            "EntryUnavailable",
            format!("native original {} entry is not provided", case.entry),
        ));
    }
    let linter = configured(&case);
    Ok(match observe(&linter, &case) {
        Ok(observation) => Outcome::Handled {
            observation: format!("{case:#?}\n{observation:#?}\n").into(),
        },
        Err(refusal) => reason(api, Some(&case.entry), refusal.kind(), refusal.detail()),
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
    let outcome = match api {
        "--native-current-api" => current(api, input)?,
        "--native-report" => {
            // Decode the authentic report schema without invoking a legacy run.
            let _: super::report::Case = serde_json::from_str(input)?;
            reason(
                api,
                None,
                "ApiUnavailable",
                "native whole report API is not provided".to_owned(),
            )
        }
        "--native-static-class" => {
            let case: super::static_class::Case = serde_json::from_str(input)?;
            if case.entry != "template" {
                reason(
                    api,
                    Some(&case.entry),
                    "EntryUnavailable",
                    format!("native original {} entry is not provided", case.entry),
                )
            } else {
                let linter = Linter::with_preset(LintPreset::Incremental)
                    .with_enabled_rules(Some(vec!["vapor/prefer-static-class".into()]))
                    .with_locale(Locale::En)
                    .with_help_level(HelpLevel::Full);
                match linter.lint_native_template(&case.source, "StaticClass.vue") {
                    Err(refusal) => reason(
                        api,
                        Some(&case.entry),
                        kind(&refusal),
                        format!("{refusal:?}"),
                    ),
                    // A future root callback alone cannot authenticate this
                    // separate whole repair observation and its edit/requery API.
                    Ok(_) => reason(
                        api,
                        Some(&case.entry),
                        "ApiUnavailable",
                        "native whole static-class repair API is not provided".to_owned(),
                    ),
                }
            }
        }
        _ => panic!("unexpected native observer API: {api}"),
    };
    Ok(format!("{}\n", serde_json::to_string(&outcome)?).into())
}
