//! Component prop value type-check generation.
//!
//! For each prop passed to a child component usage — dynamic bindings and
//! static attribute values alike — emits a typed assertion plus a single call
//! into the child's generic functional prop-checker so TypeScript can
//! validate the values and infer generics across the component boundary.

use super::super::helpers::to_safe_identifier_fragment;
use super::super::scope::{
    append_chained_vif_guard_close, append_chained_vif_guard_open, append_ignored_vif_guard_open,
};
use super::super::types::{VizeMapping, VizeSubSpan};
use super::prop_sources::{
    PropSpan, append_prop_value, generated_prop_span, generated_prop_span_preserving_comments,
    generated_prop_value, prop_name_source_range, prop_value_source_range,
};
use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use vize_carton::FxHashMap;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_carton::profile;
use vize_croquis::{
    ScopeChain,
    croquis::{ComponentUsage, PassedProp},
};

pub(super) fn is_checkable_prop(prop: &PassedProp) -> bool {
    !prop.name_is_dynamic && prop.name.as_str() != "key" && prop.name.as_str() != "ref"
}

#[derive(Clone, Copy)]
pub(crate) struct ComponentPropSource<'a> {
    pub(crate) template: Option<&'a str>,
    pub(crate) offset: u32,
    pub(crate) scopes: &'a ScopeChain,
    pub(crate) model_modifiers: Option<&'a super::model_modifiers::ModelModifierBindings>,
    pub(crate) diagnostic_ownership:
        Option<&'a crate::virtual_ts::diagnostic_ownership::DiagnosticOwnershipCollector>,
    inference_only: bool,
    inert: bool,
    expected_packet: bool,
    protect_callback_primaries: bool,
}

impl<'a> ComponentPropSource<'a> {
    pub(crate) const fn new(
        template: Option<&'a str>,
        offset: u32,
        scopes: &'a ScopeChain,
    ) -> Self {
        Self {
            template,
            offset,
            scopes,
            model_modifiers: None,
            diagnostic_ownership: None,
            inference_only: false,
            inert: false,
            expected_packet: false,
            protect_callback_primaries: false,
        }
    }

    pub(crate) const fn with_model_modifiers(
        mut self,
        bindings: &'a super::model_modifiers::ModelModifierBindings,
    ) -> Self {
        self.model_modifiers = Some(bindings);
        self
    }

    pub(crate) const fn with_diagnostic_ownership<'b>(
        self,
        ownership: Option<
            &'b crate::virtual_ts::diagnostic_ownership::DiagnosticOwnershipCollector,
        >,
    ) -> ComponentPropSource<'b>
    where
        'a: 'b,
    {
        ComponentPropSource {
            template: self.template,
            offset: self.offset,
            scopes: self.scopes,
            model_modifiers: self.model_modifiers,
            diagnostic_ownership: ownership,
            inference_only: self.inference_only,
            inert: self.inert,
            expected_packet: self.expected_packet,
            protect_callback_primaries: self.protect_callback_primaries,
        }
    }

    pub(crate) const fn with_expected_packet(mut self, expected: bool) -> Self {
        self.expected_packet = expected;
        self
    }

    pub(crate) const fn with_callback_primary_protection(mut self, protect: bool) -> Self {
        self.protect_callback_primaries = protect;
        self
    }

    fn protect_callback_primary(self, generated: std::ops::Range<usize>) {
        if self.protect_callback_primaries
            && let Some(collector) = self.diagnostic_ownership
        {
            profile!("canon.virtual_ts.root_callback.protect_primary", {
                collector.borrow_mut().protect_callback_primary(generated);
            });
        }
    }

    pub(crate) fn value_source_range(self, prop: &PassedProp) -> Option<std::ops::Range<usize>> {
        prop_value_source_range(self, prop)
    }

    pub(crate) const fn inference_only(mut self) -> Self {
        self.inference_only = true;
        self
    }

    /// An inference copy whose mappings are discarded. The native checker drops
    /// its diagnostics as unmapped, so the content mapper ignores them too.
    pub(crate) const fn inert(mut self) -> Self {
        self.inference_only = true;
        self.inert = true;
        self
    }

    pub(super) const fn is_inference_only(self) -> bool {
        self.inference_only
    }

    pub(crate) fn record_prop_projection(
        self,
        usage: &ComponentUsage,
        prop: &PassedProp,
        name: std::ops::Range<usize>,
        value: std::ops::Range<usize>,
    ) {
        use crate::virtual_ts::diagnostic_ownership::{InputOwner, InputRole};
        let Some(collector) = self.diagnostic_ownership else {
            return;
        };
        let ordinary_owner = InputOwner::for_prop(usage, prop);
        let Some(owner) = ordinary_owner.or_else(|| {
            (self.expected_packet || self.inert)
                .then(|| InputOwner::for_expected_projection(usage, prop))
                .flatten()
        }) else {
            return;
        };
        let name_source = prop_name_source_range(self, prop);
        let value_source = prop_value_source_range(self, prop).or_else(|| name_source.clone());
        // Do not retain a RefCell borrow while another emitter runs.
        let mut ledger = collector.borrow_mut();
        let mut record = |owner, generated, source| {
            if self.inert {
                ledger.record_inert(owner, generated, source);
            } else if ordinary_owner.is_some() {
                ledger.record(owner, generated, source, self.inference_only);
            } else {
                // The root relocation proof classifies the callback's creation.
                // Its contextual body still owns diagnostics; this record only
                // prevents an inference mapping from widening authored EXPECT.
                ledger.record_expect_geometry(owner, generated, source, self.inference_only);
            }
        };
        if let Some(source) = name_source {
            record(owner.with_role(InputRole::Name), name, source);
        }
        if let Some(source) = value_source {
            record(owner, value, source);
        }
    }

    /// A merged `class`/`style` entry repeats each dynamic value that its own
    /// per-prop assignment already checks; that assignment owns diagnostics.
    pub(super) fn record_merged_prop_echo(
        self,
        usage: &ComponentUsage,
        prop: &PassedProp,
        value: std::ops::Range<usize>,
    ) {
        let (Some(collector), Some(owner), Some(source)) = (
            self.diagnostic_ownership,
            crate::virtual_ts::diagnostic_ownership::InputOwner::for_prop(usage, prop),
            prop_value_source_range(self, prop),
        ) else {
            return;
        };
        collector.borrow_mut().record(owner, value, source, true);
    }
}

pub(crate) struct ComponentPropCheckContext<'a, 'b> {
    pub(crate) inference_prelude:
        Option<&'a std::cell::RefCell<super::super::scope::RootOwnerBuffer>>,
    pub(crate) ts: &'b mut String,
    pub(crate) mappings: &'b mut Vec<VizeMapping>,
    pub(crate) template_binding_access: &'a TemplateBindingAccess,
    pub(crate) source_context: ComponentPropSource<'a>,
    pub(crate) indent: &'b str,
    pub(crate) strict_v_model: bool,
    pub(crate) typed_fallthrough_values: bool,
}

impl<'a, 'b> ComponentPropCheckContext<'a, 'b> {
    pub(crate) fn new(
        ts: &'b mut String,
        mappings: &'b mut Vec<VizeMapping>,
        template_binding_access: &'a TemplateBindingAccess,
        source_context: ComponentPropSource<'a>,
        indent: &'b str,
    ) -> Self {
        Self {
            inference_prelude: None,
            ts,
            mappings,
            template_binding_access,
            source_context,
            indent,
            strict_v_model: false,
            typed_fallthrough_values: false,
        }
    }

    /// Also check what a `v-model` writes back (`strictVModel`).
    pub(crate) fn strict_v_model(mut self, enabled: bool) -> Self {
        self.strict_v_model = enabled;
        self
    }

    /// Also validate values contributed by inherited fallthrough spread keys.
    pub(crate) fn typed_fallthrough_values(mut self, enabled: bool) -> Self {
        self.typed_fallthrough_values = enabled;
        self
    }
}

pub(super) fn collect_generated_class_bindings<'a>(
    usage: &'a ComponentUsage,
    template_binding_access: &TemplateBindingAccess,
) -> Vec<(&'a PassedProp, String)> {
    collect_generated_named_bindings(usage, template_binding_access, "class")
}

pub(super) fn collect_generated_named_bindings<'a>(
    usage: &'a ComponentUsage,
    template_binding_access: &TemplateBindingAccess,
    name: &str,
) -> Vec<(&'a PassedProp, String)> {
    usage
        .props
        .iter()
        .filter(|prop| is_checkable_prop(prop) && prop.name.as_str() == name)
        .filter_map(|prop| {
            generated_prop_value(prop, template_binding_access).map(|value| (prop, value))
        })
        .collect()
}

pub(super) fn merged_class_binding_value(
    bindings: &[(&PassedProp, String)],
    mut ranges: Option<&mut Vec<std::ops::Range<usize>>>,
) -> Option<String> {
    match bindings {
        [] => None,
        [(_, value)] => {
            if let Some(ranges) = ranges.as_mut() {
                ranges.push(0..value.len());
            }
            Some(value.clone())
        }
        _ => {
            let mut value = String::default();
            value.push('[');
            for (index, (_, binding_value)) in bindings.iter().enumerate() {
                if index > 0 {
                    value.push_str(", ");
                }
                let generated = append_prop_value(&mut value, binding_value.as_str());
                if let Some(ranges) = ranges.as_mut() {
                    ranges.push(generated);
                }
            }
            value.push(']');
            Some(value)
        }
    }
}

/// Generate component prop value checks at the given indentation level.
pub(crate) fn generate_component_prop_checks(
    ctx: &mut ComponentPropCheckContext<'_, '_>,
    usage: &ComponentUsage,
    idx: usize,
    component_ref: &str,
) {
    let ts = &mut *ctx.ts;
    let mappings = &mut *ctx.mappings;
    let template_binding_access = ctx.template_binding_access;
    let source_context = ctx.source_context;
    let indent = ctx.indent;
    let strict_v_model = ctx.strict_v_model;
    let component_type_name = to_safe_identifier_fragment(usage.name.as_str());
    let has_inline_callback = usage
        .props
        .iter()
        .any(crate::virtual_ts::scope::is_inline_callback_prop);
    let grouped_guard_expr = usage.vif_guard.as_deref().filter(|_| has_inline_callback);
    let grouped_guard = grouped_guard_expr.is_some();
    if let Some(guard) = grouped_guard_expr {
        append_ignored_vif_guard_open(ts, indent, guard, "Inference-only guard");
    }
    let callback_indent = if grouped_guard {
        cstr!("{indent}  ")
    } else {
        String::from(indent)
    };
    let resolve = |text: &mut String, source_context| {
        super::callback_prop_resolution::generate_callback_props_resolution(
            text,
            usage,
            idx,
            component_ref,
            template_binding_access,
            source_context,
            callback_indent.as_str(),
        )
    };
    let resolved_props = if let Some(prelude) = ctx.inference_prelude {
        let mut prelude = prelude.borrow_mut();
        let super::super::scope::RootOwnerBuffer {
            text,
            diagnostic_ownership,
            ..
        } = &mut *prelude;
        resolve(
            text,
            source_context.with_diagnostic_ownership(
                source_context
                    .diagnostic_ownership
                    .map(|_| &*diagnostic_ownership),
            ),
        )
    } else {
        resolve(ts, source_context)
    };
    let mut name_occurrences: FxHashMap<String, u32> = FxHashMap::default();
    for prop in &usage.props {
        if !is_checkable_prop(prop) && !crate::virtual_ts::scope::is_inline_ref_callback_prop(prop)
        {
            continue;
        }
        // Static attribute values are checked exactly like dynamic bindings:
        // a static `msg="text"` must still satisfy the child's prop type
        // (vue-tsc reports TS2322 here; skipping them was a false negative).
        if prop.value.is_some() {
            let prop_src_start = (source_context.offset + prop.start) as usize;
            let prop_src_end = (source_context.offset + prop.end) as usize;
            let value_src_range = prop_value_source_range(source_context, prop);
            let generated_value = profile!(
                "canon.virtual_ts.prop_check.value",
                if crate::virtual_ts::scope::is_inline_ref_callback_prop(prop) {
                    generated_prop_span_preserving_comments(prop, template_binding_access)
                } else {
                    generated_prop_span(prop, template_binding_access)
                }
                .unwrap_or_else(|| PropSpan::plain(String::default()))
            );
            append!(
                *ts,
                "{indent}// @vize-map: prop -> {prop_src_start}:{prop_src_end}\n",
            );

            let safe_prop_name = to_safe_identifier_fragment(prop.name.as_str());
            let expr_indent = if usage.vif_guard.is_some() {
                cstr!("{indent}  ")
            } else {
                indent.into()
            };

            if !grouped_guard && let Some(ref guard) = usage.vif_guard {
                append_chained_vif_guard_open(ts, indent, guard, "Inference-only guard");
            }

            // A repeated attribute name (static class next to :class) still
            // checks every authored value, but each check constant needs a
            // unique name or the virtual TS redeclares it (TS2451).
            let occurrence = name_occurrences
                .entry(String::from(safe_prop_name.as_str()))
                .and_modify(|count| *count += 1)
                .or_insert(1);
            let check_name = if *occurrence == 1 {
                cstr!("__vize_prop_check_{idx}_{safe_prop_name}")
            } else {
                cstr!("__vize_prop_check_{idx}_{safe_prop_name}_{occurrence}")
            };
            let gen_stmt_start = ts.len();
            append!(*ts, "{expr_indent}const ");
            let check_name_start = ts.len();
            profile!("canon.virtual_ts.prop_check.name_emission", {
                ts.push_str(check_name.as_str());
            });
            let check_name_end = ts.len();
            let inline_callback = crate::virtual_ts::scope::is_inline_callback_prop(prop);
            if inline_callback && let Some(resolution) = resolved_props.as_ref() {
                let camel_prop_name = super::super::helpers::to_camel_case(prop.name.as_str());
                let resolved_props = resolution.resolved_props.as_str();
                let selected_props = resolution.selected_props.as_str();
                append!(
                    *ts,
                    ": __VizeResolvedProp<typeof {resolved_props}, typeof {selected_props}, '{camel_prop_name}', __{component_type_name}_{idx}_prop_{safe_prop_name}> = ",
                );
            } else {
                append!(
                    *ts,
                    ": __{component_type_name}_{idx}_prop_{safe_prop_name} = ",
                );
            }
            let value_gen_range = profile!("canon.virtual_ts.prop_check.value_emission", {
                append_prop_value(ts, generated_value.text.as_str())
            });
            ts.push_str(";\n");
            let statement_end = ts.len();
            let gen_stmt_end = if generated_value.omits_tail() {
                generated_value.mapped_limit(value_gen_range.start)
            } else {
                statement_end
            };
            let value_span = if generated_value.omits_tail() {
                value_gen_range.start..gen_stmt_end
            } else {
                value_gen_range
            };
            append!(*ts, "{expr_indent}void {check_name};\n");

            // The synthetic identifier receives the child prop-type error
            // (TS2322-class), which vue-tsc anchors at the attribute name;
            // the initializer keeps the exact authored expression so errors
            // inside the value land on the authored bytes.
            let diagnostic_value_span = value_span.clone();
            let name_src_range = prop_name_source_range(source_context, prop);
            let mut sub_spans = Vec::new();
            if let Some(src_range) = name_src_range.or_else(|| value_src_range.clone()) {
                sub_spans.push(VizeSubSpan {
                    gen_range: check_name_start..check_name_end,
                    src_range,
                });
            }
            if let Some(spans) =
                super::model_modifiers::modifier_sub_spans(source_context, prop, value_span.clone())
            {
                sub_spans.extend(spans);
            } else if let Some(src_range) = value_src_range.clone() {
                sub_spans.push(VizeSubSpan {
                    gen_range: value_span,
                    src_range,
                });
            }
            mappings.push(VizeMapping {
                gen_range: gen_stmt_start..gen_stmt_end,
                src_range: prop_src_start..prop_src_end,
                sub_spans,
            });
            if inline_callback {
                source_context.protect_callback_primary(gen_stmt_start..statement_end);
            }
            source_context.record_prop_projection(
                usage,
                prop,
                check_name_start..check_name_end,
                diagnostic_value_span,
            );
            if strict_v_model
                && super::model_update::is_model_prop(usage, prop)
                && let Some(target_source) = value_src_range
            {
                super::model_update::append_model_update_check(
                    ts,
                    mappings,
                    (component_type_name.as_str(), idx),
                    prop,
                    generated_value.text.as_str(),
                    target_source,
                    expr_indent.as_str(),
                );
            }

            if !grouped_guard && usage.vif_guard.is_some() {
                append_chained_vif_guard_close(ts, indent);
            }
        }
    }
    if grouped_guard {
        append!(*ts, "{indent}}}\n");
    }

    super::generic_props_call::generate_generic_props_call(
        ts,
        mappings,
        usage,
        idx,
        template_binding_access,
        source_context,
        indent,
        ctx.typed_fallthrough_values,
    );
}
