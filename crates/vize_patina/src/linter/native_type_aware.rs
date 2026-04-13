use super::{
    corsa_session::{CorsaTypeAwareSession, TypeProbe},
    LintResult, Linter,
};
use crate::diagnostic::LintDiagnostic;
use corsa::utils::{
    is_any_like_type_texts, is_promise_like_type_texts, is_unknown_like_type_texts,
};
use vize_carton::{profile, String, ToCompactString};

mod driver;
mod markers;
mod parsing;
mod rule_queries;
mod template_queries;

#[cfg(test)]
mod tests;

const RULE_REQUIRE_TYPED_PROPS: &str = "type/require-typed-props";
const RULE_REQUIRE_TYPED_EMITS: &str = "type/require-typed-emits";
const RULE_NO_FLOATING_PROMISES: &str = "type/no-floating-promises";
const RULE_NO_UNSAFE_TEMPLATE_BINDING: &str = "type/no-unsafe-template-binding";

pub(crate) fn has_active_type_aware_rules(linter: &Linter) -> bool {
    [
        RULE_REQUIRE_TYPED_PROPS,
        RULE_REQUIRE_TYPED_EMITS,
        RULE_NO_FLOATING_PROMISES,
        RULE_NO_UNSAFE_TEMPLATE_BINDING,
    ]
    .into_iter()
    .any(|rule_name| linter.registry.has_rule(rule_name) && linter.is_rule_enabled(rule_name))
}

pub(crate) fn lint_sfc_with_corsa(linter: &Linter, source: &str, filename: &str) -> LintResult {
    let mut result = match profile!(
        "patina.type_aware.parse_sfc",
        super::script_rules::parse_sfc_for_lint(source, filename)
    ) {
        Ok(descriptor) => profile!(
            "patina.type_aware.driver",
            driver::lint_with_descriptor(linter, source, filename, descriptor)
        ),
        Err(_) => {
            if let Some((content, byte_offset)) = super::engine::extract_template_fast(source) {
                let mut fallback = profile!(
                    "patina.type_aware.fallback_template_lint",
                    linter.lint_template(&content, filename)
                );
                if byte_offset > 0 {
                    for diag in &mut fallback.diagnostics {
                        diag.start += byte_offset;
                        diag.end += byte_offset;
                        for label in &mut diag.labels {
                            label.start += byte_offset;
                            label.end += byte_offset;
                        }
                    }
                }
                fallback
            } else {
                LintResult {
                    filename: filename.to_compact_string(),
                    diagnostics: Vec::new(),
                    error_count: 0,
                    warning_count: 0,
                }
            }
        }
    };

    result.filename = filename.to_compact_string();
    result
}

pub(super) fn push_warning(result: &mut LintResult, diagnostic: LintDiagnostic) {
    result.warning_count += 1;
    result.diagnostics.push(diagnostic);
}

pub(super) fn with_corsa_session<T>(
    linter: &Linter,
    filename: &str,
    f: impl FnOnce(&mut CorsaTypeAwareSession) -> Result<T, String>,
) -> Result<T, String> {
    let mut guard = profile!("patina.type_aware.corsa.lock", linter.native_corsa.lock())
        .map_err(|_| "Failed to lock corsa type session".to_compact_string())?;

    let needs_new_session = guard
        .as_ref()
        .is_none_or(|session| !session.matches_source_file(filename));

    if needs_new_session {
        if let Some(session) = guard.as_mut() {
            session.close();
        }
        *guard = Some(profile!(
            "patina.type_aware.corsa.new_session",
            CorsaTypeAwareSession::new(filename)
        )?);
    }

    let session = guard
        .as_mut()
        .ok_or_else(|| "Failed to initialize corsa type session".to_compact_string())?;

    let result = profile!("patina.type_aware.corsa.session_work", f(session));
    if result.is_err() {
        session.close();
        *guard = None;
    }
    result
}

pub(super) fn should_warn_for_prop_access(probe: Option<&TypeProbe>) -> bool {
    let Some(probe) = probe else {
        return true;
    };
    if probe.type_texts.is_empty() {
        return true;
    }
    is_any_like_type_texts(&probe.type_texts) || is_unknown_like_type_texts(&probe.type_texts)
}

pub(super) fn should_warn_for_emit_validator(probe: Option<&TypeProbe>) -> bool {
    let Some(probe) = probe else {
        return true;
    };
    if probe.call_signatures.is_empty() {
        return true;
    }
    probe.call_signatures.iter().any(|signature| {
        signature.iter().any(|parameter_type| {
            parameter_type.is_empty()
                || is_any_like_type_texts(parameter_type)
                || is_unknown_like_type_texts(parameter_type)
        })
    })
}

pub(super) fn has_promise_like_return(probe: &TypeProbe) -> bool {
    let no_properties: &[vize_carton::CompactString] = &[];
    probe.return_types.iter().any(|return_type| {
        !return_type.is_empty() && is_promise_like_type_texts(return_type, no_properties)
    })
}

pub(super) fn has_unsafe_template_type(probe: Option<&TypeProbe>) -> bool {
    let Some(probe) = probe else {
        return false;
    };
    !probe.type_texts.is_empty()
        && (is_any_like_type_texts(&probe.type_texts)
            || is_unknown_like_type_texts(&probe.type_texts))
}
