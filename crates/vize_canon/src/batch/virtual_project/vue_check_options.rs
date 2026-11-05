//! The `vueCompilerOptions` that shape template checks, resolved once for
//! `vize check` and for the Content Mapper (#11688).
//!
//! Both hosts call [`resolve`], so an option one of them honors cannot be
//! missing from the other: the mapper read only part of this set, and its
//! hosts (tsc, tsrs and the editor's backend A) silently missed the errors
//! `vize check` reported. `target` is resolved separately because it needs the
//! project's paths.

use serde_json::{Map, Value};

use crate::virtual_ts::{ResolveStyleClassNames, VirtualTsCheckOptions};

/// Every key [`resolve`] reads. `vue_check_options_tests` fails when a key is
/// read here without being listed, or read by one host outside [`resolve`].
#[cfg(test)]
pub(super) const CHECK_OPTION_KEYS: [&str; 20] = [
    "strictTemplates",
    "checkUnknownProps",
    "checkUnknownComponents",
    "checkUnknownEvents",
    "checkUnknownDirectives",
    "strictVModel",
    "strictCssModules",
    "strictComponentAttrs",
    "vapor",
    "jsxSlots",
    "inferComponentDollarEl",
    "inferTemplateDollarEl",
    "inferComponentDollarRefs",
    "inferTemplateDollarRefs",
    "inferTemplateDollarSlots",
    "inferTemplateDollarAttrs",
    "fallthroughAttributes",
    "checkRequiredFallthroughAttributes",
    "resolveStyleClassNames",
    "resolveStyleImports",
];

/// Apply a project's flattened `vueCompilerOptions` to `checks`. `None` means
/// the options could not be read; unknown props then stay checked.
#[expect(clippy::disallowed_types, reason = "serde_json keys are std String")]
pub(super) fn resolve(
    checks: &mut VirtualTsCheckOptions,
    options: Option<&Map<std::string::String, Value>>,
) {
    let value = |name: &str| options.and_then(|options| options.get(name));
    let flag = |name: &str| value(name).and_then(Value::as_bool);
    let set = |name: &str| flag(name).unwrap_or(false);
    // `checkUnknownProps` and these four default to `strictTemplates`.
    let strict = |name: &str| {
        flag(name)
            .or_else(|| flag("strictTemplates"))
            .unwrap_or(false)
    };

    checks.check_unknown_props = options.is_none() || strict("checkUnknownProps");
    checks.check_unknown_fallthrough_props = flag("checkUnknownProps") == Some(true);
    checks.check_unknown_components = strict("checkUnknownComponents");
    checks.check_unknown_events = strict("checkUnknownEvents");
    checks.check_unknown_directives = strict("checkUnknownDirectives");
    checks.strict_v_model = strict("strictVModel");
    checks.strict_css_modules = set("strictCssModules");
    checks.strict_component_attrs = set("strictComponentAttrs");
    checks.vapor = set("vapor");
    checks.jsx_slots = set("jsxSlots");
    checks.infer_component_dollar_el = set("inferComponentDollarEl");
    checks.infer_template_dollar_el = set("inferTemplateDollarEl");
    checks.infer_component_dollar_refs = set("inferComponentDollarRefs");
    checks.infer_template_dollar_refs = set("inferTemplateDollarRefs");
    checks.infer_template_dollar_slots = set("inferTemplateDollarSlots");
    checks.infer_template_dollar_attrs = set("inferTemplateDollarAttrs");
    checks.fallthrough_attributes_specified = value("fallthroughAttributes").is_some();
    checks.fallthrough_attributes = set("fallthroughAttributes");
    checks.check_required_fallthrough_attributes = set("checkRequiredFallthroughAttributes");
    checks.resolve_style_class_names = match flag("resolveStyleClassNames") {
        Some(true) => ResolveStyleClassNames::All,
        Some(false) => ResolveStyleClassNames::None,
        None => ResolveStyleClassNames::Scoped,
    };
    checks.resolve_style_imports = set("resolveStyleImports");
}
