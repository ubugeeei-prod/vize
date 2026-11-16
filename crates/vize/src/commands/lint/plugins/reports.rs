//! Rule severities and the mapping of plugin reports to lint diagnostics.

use std::sync::{Mutex, OnceLock};

use serde_json::Value;

use super::field;
use crate::config::LintRuleSeverity;
use crate::config::plugin_plan::{PlannedCapability, PlannedPlugin};
use vize_l0::{FxHashMap, String, cstr};
use vize_patina::plugin_batch::{batch::diagnostics, document::PluginDocument};
use vize_patina::{Fix, LintDiagnostic, Severity, TextEdit};

/// The enabled rules of one job.
pub(super) enum Rules {
    /// One v2 rule: its public ID, severity, messages and fixability.
    V2 {
        id: &'static str,
        severity: Severity,
        messages: FxHashMap<String, String>,
        /// `meta.docs`, shown as help.
        docs: Option<String>,
        /// `meta.fixable === "safe"`: `--fix` may apply its fixes.
        safe_fixes: bool,
    },
    /// A v1 plugin's rules by public ID; `None` is configured off.
    V1 {
        plugin: String,
        severities: FxHashMap<String, Option<Severity>>,
    },
}

impl Rules {
    pub(super) fn v2(
        plugin: &PlannedPlugin,
        capability: &PlannedCapability,
        configured: &FxHashMap<String, LintRuleSeverity>,
    ) -> Result<Option<Self>, String> {
        let data = &capability.data;
        let invalid = || cstr!("[plugin/plan-invalid] {}: malformed lint.rule", plugin.name);
        let default = field(data, "defaultLevel").as_str().ok_or_else(invalid)?;
        let Some(severity) = severity(configured.get(capability.key.as_str()), default) else {
            return Ok(None);
        };
        let messages = field(data, "messages")
            .as_object()
            .ok_or_else(invalid)?
            .iter()
            .map(|(id, text)| Some((String::from(id.as_str()), String::from(text.as_str()?))))
            .collect::<Option<FxHashMap<_, _>>>()
            .ok_or_else(invalid)?;
        let docs = field(data, "docs")
            .as_str()
            .filter(|docs| !docs.is_empty())
            .map(String::from);
        Ok(Some(Self::V2 {
            id: intern(&capability.key),
            severity,
            messages,
            docs,
            safe_fixes: field(data, "fixable").as_str() == Some("safe"),
        }))
    }

    pub(super) fn v1(
        plugin: &PlannedPlugin,
        capability: &PlannedCapability,
        configured: &FxHashMap<String, LintRuleSeverity>,
    ) -> Result<Option<Self>, String> {
        let rules = field(&capability.data, "rules").as_array().ok_or_else(|| {
            cstr!(
                "[plugin/plan-invalid] {}: malformed lint.plugin-v1",
                plugin.name
            )
        })?;
        let mut severities = FxHashMap::default();
        for rule in rules {
            let id = cstr!("{}/{}", plugin.name, rule.as_str().unwrap_or_default());
            // v1 plugins report warnings unless the config says otherwise.
            severities.insert(id.clone(), severity(configured.get(id.as_str()), "warn"));
        }
        if severities.values().all(Option::is_none) {
            return Ok(None);
        }
        Ok(Some(Self::V1 {
            plugin: plugin.name.clone(),
            severities,
        }))
    }

    /// Map one document's reports to authored-span diagnostics.
    pub(super) fn map(
        &self,
        document: &PluginDocument,
        reports: &Value,
    ) -> Result<Vec<LintDiagnostic>, String> {
        match self {
            Self::V1 { plugin, severities } => {
                let reports = reports.as_str().ok_or_else(|| {
                    cstr!("[plugin/report-invalid] {plugin}: v1 reports must be JSON text")
                })?;
                let found = diagnostics(document, plugin, reports)
                    .map_err(|error| cstr!("[plugin/report-invalid] {plugin}: {error}"))?;
                Ok(found
                    .into_iter()
                    .filter_map(|found| {
                        let severity = (*severities.get(found.rule_id.as_str())?)?;
                        let fix = found.fix.as_ref().map(|text| {
                            Fix::new(
                                cstr!("Apply the {} fix", found.rule_id),
                                TextEdit::replace(found.start, found.end, text.as_str()),
                            )
                        });
                        let diagnostic = diagnostic(
                            intern(&found.rule_id),
                            severity,
                            &found.message,
                            found.start,
                            found.end,
                        );
                        Some(match fix {
                            Some(fix) => diagnostic.with_fix(fix),
                            None => diagnostic,
                        })
                    })
                    .collect())
            }
            Self::V2 {
                id,
                severity,
                messages,
                docs,
                ..
            } => {
                let invalid = |detail: &str| cstr!("[plugin/report-invalid] {id}: {detail}");
                let reports = reports
                    .as_array()
                    .ok_or_else(|| invalid("reports must be an array"))?;
                let mut mapped = Vec::with_capacity(reports.len());
                for report in reports {
                    let node = field(report, "node")
                        .as_u64()
                        .and_then(|node| {
                            document
                                .nodes
                                .iter()
                                .find(|candidate| u64::from(candidate.id) == node)
                        })
                        .ok_or_else(|| invalid("a report names a node the document lacks"))?;
                    let template = field(report, "messageId")
                        .as_str()
                        .and_then(|message| messages.get(message))
                        .ok_or_else(|| invalid("a report names an undeclared message"))?;
                    let message = render(template, field(report, "data"))
                        .map_err(|detail| invalid(&detail))?;
                    let diagnostic = diagnostic(id, *severity, &message, node.start, node.end);
                    let diagnostic = match docs {
                        Some(docs) => diagnostic.with_help(docs.as_str()),
                        None => diagnostic,
                    };
                    mapped.push(match field(report, "fix").as_str() {
                        Some(text) => diagnostic.with_fix(Fix::new(
                            cstr!("Apply the {id} fix"),
                            TextEdit::replace(node.start, node.end, text),
                        )),
                        None => diagnostic,
                    });
                }
                mapped.sort_by_key(|diagnostic| (diagnostic.start, diagnostic.end));
                Ok(mapped)
            }
        }
    }
}

/// Public IDs of the enabled rules, interned.
impl Rules {
    /// The rule ID when `--fix` may apply this rule's fixes.
    pub(super) fn safe_fix_id(&self) -> Option<&'static str> {
        match self {
            Self::V2 {
                id,
                safe_fixes: true,
                ..
            } => Some(id),
            _ => None,
        }
    }

    pub(super) fn ids(&self) -> Vec<&'static str> {
        match self {
            Self::V2 { id, .. } => vec![id],
            Self::V1 { severities, .. } => severities
                .iter()
                .filter(|(_, severity)| severity.is_some())
                .map(|(id, _)| intern(id))
                .collect(),
        }
    }
}

fn severity(configured: Option<&LintRuleSeverity>, default: &str) -> Option<Severity> {
    match configured {
        Some(LintRuleSeverity::Off) => None,
        Some(LintRuleSeverity::Warn) => Some(Severity::Warning),
        Some(LintRuleSeverity::Error) => Some(Severity::Error),
        None => match default {
            "warn" => Some(Severity::Warning),
            "error" => Some(Severity::Error),
            _ => None,
        },
    }
}

fn diagnostic(
    id: &'static str,
    severity: Severity,
    message: &str,
    start: u32,
    end: u32,
) -> LintDiagnostic {
    match severity {
        Severity::Error => LintDiagnostic::error(id, message, start, end),
        Severity::Warning => LintDiagnostic::warn(id, message, start, end),
    }
}

/// `{name}` placeholders take the report's bounded data; a missing value fails closed.
fn render(template: &str, data: &Value) -> Result<String, std::string::String> {
    let mut output = String::default();
    let mut rest = template;
    while let Some((head, tail)) = rest.split_once('{') {
        output.push_str(head);
        let Some((name, after)) = tail.split_once('}') else {
            output.push('{');
            rest = tail;
            continue;
        };
        let value = match data.get(name) {
            Some(Value::String(text)) => text.clone(),
            Some(Value::Number(number)) => cstr!("{number}").into_string(),
            Some(Value::Bool(flag)) => cstr!("{flag}").into_string(),
            _ => return Err(cstr!("message data lacks `{name}`").into_string()),
        };
        output.push_str(&value);
        rest = after;
    }
    output.push_str(rest);
    Ok(output)
}

/// Lint diagnostics carry `'static` rule IDs. Plugin rule IDs come from the
/// plan, which is fixed for the process, so each distinct ID is leaked once.
fn intern(id: &str) -> &'static str {
    static IDS: OnceLock<Mutex<FxHashMap<String, &'static str>>> = OnceLock::new();
    let Ok(mut ids) = IDS.get_or_init(Default::default).lock() else {
        return "plugin";
    };
    if let Some(found) = ids.get(id) {
        return found;
    }
    let leaked: &'static str = Box::leak(Box::<str>::from(id));
    ids.insert(String::from(id), leaked);
    leaked
}
