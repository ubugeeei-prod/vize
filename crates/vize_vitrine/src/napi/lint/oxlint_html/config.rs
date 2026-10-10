use std::path::Path;

use serde_json::Value;
use vize_l0::{FxHashMap, String, ToCompactString, cstr};
use vize_patina::{Linter, Severity};

use super::super::lint_options::{
    PatinaLintOptionsNapi, create_patina_linter, patina_help_level_from_option,
    patina_locale_from_option, patina_preset_from_option,
};
use super::super::rule_metadata::collect_patina_rule_metadata;
use super::{ProjectedRule, Projection, profile, root_json, rule_options, settings};
use profile::{Refusal, RefusalKind};

pub(super) struct Plan {
    pub linter: Linter,
    pub projection: Projection,
}

pub(super) fn decode(path: &Path, bytes: &[u8], expected: &[u8]) -> Result<(Value, Plan), Refusal> {
    let failure = |kind, details| Refusal {
        kind,
        path: path.to_path_buf(),
        details,
        original_bytes: Some(bytes.to_vec()),
    };
    if bytes != expected {
        return Err(failure(
            RefusalKind::ConfigIdentity,
            "root JSON differs from the original caller snapshot".into(),
        ));
    }
    let root = root_json::decode(bytes)
        .map_err(|error| failure(RefusalKind::RootJsonSyntax, error.to_compact_string()))?;
    if !root.is_object() {
        return Err(failure(
            RefusalKind::RootJsonSyntax,
            "JSON object required".into(),
        ));
    }
    let plan = project(&root).map_err(|error| failure(RefusalKind::ConfigProjection, error))?;
    Ok((root, plan))
}

fn project(root: &Value) -> Result<Plan, String> {
    let settings = settings::project(root)?;
    let deny_warnings = settings::deny_warnings(root)?;
    let metadata = collect_patina_rule_metadata()
        .into_iter()
        .map(|rule| (rule.name, rule))
        .collect::<FxHashMap<_, _>>();
    let mut options = PatinaLintOptionsNapi::default();
    let mut rules = Vec::new();
    if let Some(configured) = root.get("rules") {
        let Some(configured) = configured.as_object() else {
            return Err("/rules must be an object".into());
        };
        for (key, setting) in configured {
            let Some(name) = key.strip_prefix("vize/") else {
                continue; // The real host retains every foreign/core setting.
            };
            let Some(meta) = metadata.get(name) else {
                return Err(cstr!("/rules/{key}: unknown Vize rule"));
            };
            let (authored_severity, payload) = match setting.as_array() {
                Some(tuple) => {
                    let Some(severity) = tuple.first() else {
                        return Err(cstr!("/rules/{key}: severity tuple is empty"));
                    };
                    if tuple.len() > 2 {
                        return Err(cstr!(
                            "/rules/{key}: at most one option payload is supported"
                        ));
                    }
                    (severity, tuple.get(1))
                }
                None => (setting, None),
            };
            let severity =
                severity(authored_severity).map_err(|error| cstr!("/rules/{key}: {error}"))?;
            if severity.is_some() && name.starts_with("type/") {
                return Err(cstr!(
                    "/rules/{key}: native type-aware rules are unqualified"
                ));
            }
            let active = severity.is_some()
                && (meta.presets.is_empty()
                    || settings.preset == "incremental"
                    || meta.presets.contains(&settings.preset.as_str()));
            if active {
                rule_options::parse_rule_option(name, payload, &mut options)
                    .map_err(|error| cstr!("/rules/{key}: {error}"))?;
            } else {
                // Validate disabled/gated authoring without applying its options.
                let mut unused = PatinaLintOptionsNapi::default();
                rule_options::parse_rule_option(name, payload, &mut unused)
                    .map_err(|error| cstr!("/rules/{key}: {error}"))?;
            }
            rules.push(ProjectedRule {
                name: name.into(),
                severity,
                active,
                authored_options: payload.cloned().into_iter().collect(),
            });
        }
    }
    let enabled = rules
        .iter()
        .filter(|rule| rule.active)
        .map(|rule| rule.name.clone())
        .collect();
    let overrides = rules
        .iter()
        .filter(|rule| rule.active)
        .filter_map(|rule| rule.severity.map(|severity| (rule.name.clone(), severity)))
        .collect();
    let linter = create_patina_linter(patina_preset_from_option(Some(settings.preset.as_str())))
        .with_locale(patina_locale_from_option(Some(settings.locale.as_str())))
        .with_help_level(patina_help_level_from_option(Some(
            settings.help_level.as_str(),
        )))
        .with_type_aware_lint(false)
        .with_enabled_rules(Some(enabled));
    // Incremental enabled selection replaces the registry, so options come next.
    let linter =
        rule_options::apply_rule_options(linter, options).with_rule_severity_overrides(overrides);
    Ok(Plan {
        linter,
        projection: Projection {
            rules,
            settings,
            deny_warnings,
        },
    })
}

fn severity(value: &Value) -> Result<Option<Severity>, String> {
    match value {
        Value::String(value) => match value.as_str() {
            "off" => Ok(None),
            "warn" => Ok(Some(Severity::Warning)),
            "error" => Ok(Some(Severity::Error)),
            _ => Err("severity must be off, warn, error or 0/1/2".into()),
        },
        Value::Number(value) => match value.as_u64() {
            Some(0) => Ok(None),
            Some(1) => Ok(Some(Severity::Warning)),
            Some(2) => Ok(Some(Severity::Error)),
            _ => Err("severity must be off, warn, error or 0/1/2".into()),
        },
        _ => Err("severity must be off, warn, error or 0/1/2".into()),
    }
}
