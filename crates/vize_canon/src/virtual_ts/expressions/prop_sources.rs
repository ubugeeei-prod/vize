//! Authored source ranges and generated values for component prop checks.
//!
//! These helpers translate one passed prop into the pieces check emission
//! needs: the escaped or rewritten generated value, the authored
//! attribute-name range that anchors child prop-type errors (matching
//! vue-tsc), and the authored value range that anchors errors inside the
//! bound expression.

use super::component_props::ComponentPropSource;
use super::reserved_props::rewrite_reserved_template_prop;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_croquis::croquis::PassedProp;
use vize_croquis::drawer::strip_js_comments;

fn push_ts_string_literal(out: &mut String, value: &str) {
    out.push('"');
    for ch in value.chars() {
        match ch {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            _ => out.push(ch),
        }
    }
    out.push('"');
}

pub(super) fn generated_prop_value(
    prop: &PassedProp,
    template_prop_names: &FxHashSet<String>,
) -> Option<String> {
    if !prop.is_dynamic {
        let mut value = String::default();
        if let Some(static_value) = prop.value.as_ref() {
            push_ts_string_literal(&mut value, static_value.as_str());
        } else {
            value.push_str("true");
        }
        return Some(value);
    }

    let value = strip_js_comments(prop.value.as_ref()?.as_str());
    let trimmed_value = value.as_ref().trim();
    let rewritten_value = rewrite_reserved_template_prop(trimmed_value, template_prop_names);
    Some(rewritten_value.as_ref().map_or_else(
        || String::from(value.as_ref()),
        |s| String::from(s.as_str()),
    ))
}

pub(super) fn prop_value_source_range(
    source_context: ComponentPropSource<'_>,
    prop: &PassedProp,
) -> Option<std::ops::Range<usize>> {
    let source = source_context.template?;
    let value = prop.value.as_ref()?.as_str();
    let prop_start = prop.start as usize;
    let prop_end = prop.end as usize;
    let raw_prop = source.get(prop_start..prop_end)?;
    let relative_start = raw_prop.rfind(value)?;
    let source_start = source_context.offset as usize + prop_start + relative_start;
    Some(source_start..source_start + value.len())
}

/// Returns the authored range of the attribute name itself — `msg` inside
/// `:msg="expr"`, `v-bind:msg="expr"`, or `msg="text"`.
///
/// vue-tsc anchors prop-type diagnostics at the attribute name, so the
/// synthetic check identifier maps here for byte-identical positions.
pub(super) fn prop_name_source_range(
    source_context: ComponentPropSource<'_>,
    prop: &PassedProp,
) -> Option<std::ops::Range<usize>> {
    let source = source_context.template?;
    let prop_start = prop.start as usize;
    let raw_prop = source.get(prop_start..prop.end as usize)?;
    let name_region = raw_prop.split('=').next().unwrap_or(raw_prop);
    let name = prop.name.as_str();
    // The authored name sits right after the binding prefix; matching there
    // (instead of searching) keeps names like `bind` from anchoring inside
    // `v-bind:` and modifiers like `.sync` from stealing the match.
    let prefix_len = if let Some(rest) = name_region.strip_prefix("v-bind:") {
        rest.starts_with(name).then_some("v-bind:".len())?
    } else if let Some(rest) = name_region.strip_prefix(':') {
        rest.starts_with(name).then_some(1)?
    } else if name_region.starts_with(name) {
        0
    } else {
        name_region.find(name)?
    };
    let source_start = source_context.offset as usize + prop_start + prefix_len;
    Some(source_start..source_start + name.len())
}
