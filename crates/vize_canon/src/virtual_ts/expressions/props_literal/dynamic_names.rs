//! `v-bind:[name]` entries of the checked props literal (#13269).
//!
//! A dynamic argument is no static contract key (#2797), but at runtime it
//! is still a prop. Without it, `<Child :[propName]="value" />` reached the
//! whole-props check as `{}` and reported a required prop the binding
//! supplies. Each such binding is spread as `...{ [name]: value }`, so
//! TypeScript decides as vue-tsc does: a literal-typed name supplies and
//! checks its prop, while a `string` name supplies nothing required. As a
//! spread it raises no `TS1117`/`TS2783` against a static key of the same
//! name.
//!
//! Vue camelizes a prop key at runtime, so `:[name]` with `name: 'some-message'`
//! sets `someMessage`. A literal key therefore passes through a type-level
//! camelize, [`CAMELIZED_KEY_ALIAS`], when the camelized name is a prop of the
//! child; any other key stays the attribute Vue passes through as authored. The
//! checked call declares the alias after itself only when the usage has such a
//! binding, and a `string` name stays `string`.
//!
//! Like a `v-bind="bag"` spread, the entry owns its name and value
//! expressions: both are mapped to their authored bytes, and
//! `scope::component_prop_expressions` drops their bare `void (…)` copies.
//! Only the mapped props call (the one that owns the ARIA value contract)
//! carries the entries; inference copies keep their static-key contract.

use super::append_spread_value;
use crate::virtual_ts::expressions::component_props::ComponentPropSource;
use crate::virtual_ts::expressions::prop_sources::prop_value_source_range;
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use crate::virtual_ts::types::VizeMapping;
use vize_carton::append;
use vize_croquis::croquis::{ComponentUsage, PassedProp};

/// The block-local camelize alias of the props call `$CamelizedPropKey_{idx}`.
const CAMELIZED_KEY_ALIAS: &str = "$CamelizedPropKey_";

/// Whether `prop` is an authored `v-bind:[name]="value"` (or `:[name]`).
pub(crate) fn is_dynamic_name_binding(template: Option<&str>, prop: &PassedProp) -> bool {
    prop.name_is_dynamic
        && prop.is_dynamic
        && prop.value.is_some()
        && template
            .and_then(|source| source.get(prop.start as usize..prop.end as usize))
            .is_some_and(|raw| raw.starts_with(":[") || raw.starts_with("v-bind:["))
}

/// Append one spread entry per `v-bind:[name]="value"` of `usage`.
pub(super) fn append_dynamic_name_entries(
    ts: &mut crate::virtual_ts::TextBuf,
    mappings: &mut Vec<VizeMapping>,
    usage: &ComponentUsage,
    template_binding_access: &TemplateBindingAccess,
    source_context: ComponentPropSource<'_>,
    expr_indent: &str,
) {
    let Some(idx) = source_context.aria_value_contract_index else {
        return;
    };
    let property_key = crate::virtual_ts::generated_names::lib("PropertyKey");
    let component = crate::virtual_ts::helpers::to_safe_identifier_fragment(usage.name.as_str());
    for prop in &usage.props {
        if !is_dynamic_name_binding(source_context.template, prop) {
            continue;
        }
        let Some(value) = prop.value.as_deref() else {
            continue;
        };
        append!(
            *ts,
            "{expr_indent}  ...{{ [(undefined as unknown as <S extends {property_key}>(name: S) => S extends string ? {CAMELIZED_KEY_ALIAS}{idx}<S> extends keyof __{component}_Props_{idx} ? {CAMELIZED_KEY_ALIAS}{idx}<S> : S : S)("
        );
        append_mapped(
            ts,
            mappings,
            prop.name.as_str(),
            name_source_range(source_context, prop),
            (template_binding_access, source_context, usage),
        );
        ts.push_str(")]: ");
        append_mapped(
            ts,
            mappings,
            value,
            prop_value_source_range(source_context, prop),
            (template_binding_access, source_context, usage),
        );
        ts.push_str(" },\n");
    }
}

/// Declare the camelize alias the dynamic keys of props call `idx` read, when
/// `usage` has one. Vue's `camelize` uppercases the character after each `-`.
pub(crate) fn append_camelized_key_alias(
    ts: &mut crate::virtual_ts::TextBuf,
    usage: &ComponentUsage,
    template: Option<&str>,
    idx: usize,
    indent: &str,
) {
    if !usage
        .props
        .iter()
        .any(|prop| is_dynamic_name_binding(template, prop))
    {
        return;
    }
    let capitalize = crate::virtual_ts::generated_names::lib("Capitalize");
    append!(
        *ts,
        "{indent}type {CAMELIZED_KEY_ALIAS}{idx}<S extends string> = S extends `${{infer H}}-${{infer C}}${{infer T}}` ? `${{H}}${{{capitalize}<C>}}${{{CAMELIZED_KEY_ALIAS}{idx}<T>}}` : S;\n"
    );
}

fn append_mapped(
    ts: &mut crate::virtual_ts::TextBuf,
    mappings: &mut Vec<VizeMapping>,
    expression: &str,
    source: Option<std::ops::Range<usize>>,
    (template_binding_access, source_context, usage): (
        &TemplateBindingAccess,
        ComponentPropSource<'_>,
        &ComponentUsage,
    ),
) {
    let (gen_range, sub_spans) = append_spread_value(
        ts,
        expression.trim(),
        source.clone(),
        template_binding_access,
        source_context.scopes,
        usage.scope_id,
    );
    if let Some(src_range) = source {
        mappings.push(VizeMapping {
            gen_range,
            src_range,
            sub_spans,
        });
    }
}

/// The authored bytes of the `[name]` argument expression.
fn name_source_range(
    source_context: ComponentPropSource<'_>,
    prop: &PassedProp,
) -> Option<std::ops::Range<usize>> {
    let raw = source_context
        .template?
        .get(prop.start as usize..prop.end as usize)?;
    let open = raw.find('[')? + 1;
    let mut depth = 1_usize;
    let close = raw.get(open..)?.char_indices().find_map(|(index, ch)| {
        match ch {
            '[' => depth += 1,
            ']' => depth -= 1,
            _ => {}
        }
        (depth == 0).then_some(open + index)
    })?;
    let argument = raw.get(open..close)?;
    let name = prop.name.as_str().trim();
    let relative = argument.find(name)?;
    let start = source_context.offset as usize + prop.start as usize + open + relative;
    Some(start..start + name.len())
}
