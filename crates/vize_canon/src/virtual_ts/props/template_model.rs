//! The one normalized prop model every template-scope prop surface is built from.
//!
//! Vue resolves a component's declarations into exactly one prop contract: the
//! authored types with its own `default:`/`withDefaults` substitution applied.
//! The template's `props` const, the bare per-prop bindings and `$props` are all
//! views of that single contract, so they are all derived from
//! [`TemplatePropsModel`] rather than from separate scans.
//!
//! Before #4145 `$props` was the odd one out: it was emitted as a generic
//! instance global, `__VizeInstanceGlobal<'$props'>`, which reads
//! `ComponentPublicInstance['$props']` with that helper's *default* type
//! arguments. `P` defaults to `{}` there, so `$props` collapsed to `{}` for
//! every component and each declared prop read reported `TS2339`.

use vize_carton::{FxHashSet, String, append, cstr};
use vize_croquis::macros::MacroKind;
use vize_croquis::{Croquis, ScopeData, ScopeKind};

use super::super::helpers::to_safe_identifier;
use super::keyed_template_names::collect_keyed_template_prop_names;
use super::mappings::PropBindingMappings;
use super::setup_scoped::props_type_ref;
use super::template_bindings::{
    emit_macro_template_prop_bindings, should_skip_template_prop_binding,
};
use super::with_defaults::{collect_with_defaults_names_from_source, template_props_type_ref};
use super::{strip_generic_params, strip_outer_angle_brackets, type_reference_lookup_key};
use crate::virtual_ts::generator::setup_scope::define_props_type_requires_setup_scope;

mod opaque_defaults;
mod type_shape;

use opaque_defaults::{generate_opaque_defaults_capture, opaque_defaults_props_type_ref};
use type_shape::{has_top_level_type_operator, is_plain_inline_type_literal};

/// The component's resolved prop contract as the template sees it.
pub(crate) struct TemplatePropsModel {
    /// The authored contract: `Props`, `Props<T>`, or the setup-scoped
    /// `__VizeSetupProps` when the type argument only resolves inside `__setup`.
    authored_type_ref: String,
    /// The authored contract with Vue's default substitution applied. This is
    /// what a template read of a prop resolves against.
    resolved_type_ref: String,
    /// Props whose declaration carries a value Vue substitutes when the prop is
    /// absent, so the resolved contract may not leave them `undefined`.
    defaulted_prop_names: FxHashSet<String>,
    /// Whether the component declares any prop at all. A component that
    /// declares none has `$props: {}` in Vue too, so there is nothing to model.
    declares_props: bool,
}

impl TemplatePropsModel {
    pub(crate) fn new(summary: &Croquis) -> Self {
        let props = summary.macros.props();
        let models = summary.macros.models();
        let define_props_type_args = summary
            .macros
            .define_props()
            .and_then(|m| m.type_args.as_ref());
        let declares_props =
            !props.is_empty() || !models.is_empty() || define_props_type_args.is_some();

        let setup_scoped = define_props_type_requires_setup_scope(summary);
        let generic_param = sfc_generic_param(summary);
        let authored_type_ref =
            props_type_ref(generic_param, setup_scoped.then_some("__VizeSetupProps"));

        let (mut defaulted_prop_names, undefined_default_names) =
            collect_with_defaults_default_names(summary);
        // Imported runtime props and type-based defaults retain their resolved
        // names separately from authored values, so a raw `undefined` expression
        // cannot acquire the imported resolver's default-presence facts.
        for prop in props {
            if prop
                .default_value
                .as_deref()
                .is_some_and(|value| value.trim() != "undefined")
            {
                defaulted_prop_names.insert(prop.name.as_str().into());
            }
        }
        for model in models {
            if model.default_value.is_some() {
                defaulted_prop_names.insert(model.name.as_str().into());
            }
        }

        // `__DefineProps` is the setup macro's own return type, but it applies
        // `{ [K in __VizeBooleanKey<T>]-?: boolean }`, and that key filter can
        // only be decided when `keyof T` is concrete. A generic SFC leaves it
        // deferred, so TypeScript must also consider the branch where a
        // string-typed prop *is* a boolean key and resolves it to `T[K] &
        // boolean` — which made authored `as string` casts fail with `TS2352`
        // across nuxt-ui and reka-ui (#4242). It stays exactly where it already
        // was: a type-only declaration with no defaults, whose keys are known.
        // Everywhere else the authored alias is used unwrapped, which also keeps
        // the generated helper name out of diagnostic text.
        let base_type_ref = if define_props_type_args.is_some() && defaulted_prop_names.is_empty() {
            cstr!("__DefineProps<{authored_type_ref}>")
        } else {
            authored_type_ref.clone()
        };
        // Boolean casting needs concrete keys, the same condition `__DefineProps`
        // relies on above.
        let boolean_casting = (define_props_type_args.is_some() && generic_param.is_none())
            .then_some(&undefined_default_names);
        let resolved_type_ref = opaque_defaults_props_type_ref(summary).unwrap_or_else(|| {
            template_props_type_ref(
                base_type_ref.as_str(),
                &defaulted_prop_names,
                boolean_casting,
            )
        });

        Self {
            authored_type_ref,
            resolved_type_ref,
            defaulted_prop_names,
            declares_props,
        }
    }

    /// The template's `$props`. Vue's public instance props are the same
    /// resolved contract, readonly at the top level only — `vue-tsc` reports
    /// `TS2540` for `$props.x = …` while accepting a nested `$props.x.y = …`,
    /// which is what a shallow `Readonly` gives.
    ///
    /// `None` for a component that declares no props: `$props` then keeps the
    /// generic instance-global form, whose `{}` is what `vue-tsc` reports too.
    pub(crate) fn instance_props_type(&self) -> Option<String> {
        self.declares_props
            .then(|| cstr!("Readonly<{}>", self.resolved_type_ref))
    }

    /// Emit the setup-scope capture an opaque `withDefaults` argument is typed
    /// from. Must precede the template scope that reads the resolved contract.
    pub(crate) fn emit_defaults_capture(ts: &mut String, summary: &Croquis) {
        let model = Self::new(summary);
        generate_opaque_defaults_capture(ts, summary, model.authored_type_ref.as_str());
    }
}

/// The SFC's `<script setup generic="…">` parameter list, which the generated
/// `Props` alias is instantiated with.
fn sfc_generic_param(summary: &Croquis) -> Option<&str> {
    summary
        .scopes
        .iter()
        .find(|scope| matches!(scope.kind, ScopeKind::ScriptSetup))
        .and_then(|scope| match scope.data() {
            ScopeData::ScriptSetup(data) => data.generic.as_ref().map(|generic| generic.as_str()),
            _ => None,
        })
}

/// Props a `withDefaults` call defaults (with a value), and those it defaults
/// to literally `undefined`.
fn collect_with_defaults_default_names(
    summary: &Croquis,
) -> (FxHashSet<String>, FxHashSet<String>) {
    let mut undefined = FxHashSet::default();
    let mut names = summary
        .macros
        .resolved_prop_defaults()
        .map(String::from)
        .collect();
    for call in summary.macros.all_calls() {
        if call.kind != MacroKind::WithDefaults {
            continue;
        }
        let Some(runtime_args) = &call.runtime_args else {
            continue;
        };
        collect_with_defaults_names_from_source(runtime_args.as_str(), &mut names, &mut undefined);
    }
    (names, undefined)
}

/// Names and authored anchor for the mapper-owned props object in template scope.
pub(crate) struct TemplatePropsScope<'a> {
    props_shadow_anchor: Option<&'a std::ops::Range<usize>>,
    props_object_binding: &'a str,
}

impl<'a> TemplatePropsScope<'a> {
    pub(crate) fn new(
        props_shadow_anchor: Option<&'a std::ops::Range<usize>>,
        props_object_binding: &'a str,
    ) -> Self {
        Self {
            props_shadow_anchor,
            props_object_binding,
        }
    }
}

/// Returns the template names declared only as keyed props locals: the
/// analyzer never bound them, yet each one is now a template local whose
/// `satisfies keyof` key reports a name the props do not declare.
pub(crate) fn generate_props_variables(
    ts: &mut String,
    binding_mappings: &mut PropBindingMappings<'_>,
    summary: &Croquis,
    check_props: bool,
    scope: TemplatePropsScope<'_>,
) -> FxHashSet<String> {
    let TemplatePropsScope {
        props_shadow_anchor,
        props_object_binding,
    } = scope;
    let mut keyed_names = FxHashSet::default();
    let model = TemplatePropsModel::new(summary);
    if !model.declares_props {
        return keyed_names;
    }

    let props = summary.macros.props();
    let has_props = !props.is_empty();
    let models = summary.macros.models();
    let define_props_type_args = summary
        .macros
        .define_props()
        .and_then(|m| m.type_args.as_ref());

    let template_props_type_ref = model.resolved_type_ref.as_str();
    let defaulted_prop_names = &model.defaulted_prop_names;

    ts.push_str("  // Props are available in template as variables\n");
    ts.push_str("  // Access via `propName` or `props.propName`\n");
    let props_start = ts.len() + "  const ".len();
    append!(
        *ts,
        "  const {props_object_binding}: {template_props_type_ref} = {{}} as {template_props_type_ref};\n"
    );
    if props_object_binding == "props"
        && let Some(anchor) = props_shadow_anchor
    {
        binding_mappings.link_setup_shadow(
            anchor.clone(),
            props_start..props_start + props_object_binding.len(),
        );
    }
    append!(
        *ts,
        "  void {props_object_binding}; // Mark as used to avoid TS6133\n"
    );

    let mut emitted_names = FxHashSet::default();
    if let Some(type_args) = define_props_type_args {
        let type_name = strip_outer_angle_brackets(type_args.trim());

        let type_properties = summary
            .types
            .extract_properties(type_reference_lookup_key(type_name));
        for prop in &type_properties {
            if should_skip_template_prop_binding(summary, prop.name.as_str()) {
                continue;
            }
            binding_mappings.emit(
                ts,
                props_object_binding,
                template_props_type_ref,
                prop.name.as_str(),
                defaulted_prop_names.contains(&prop.name),
            );
            emitted_names.insert(prop.name.as_str().into());
        }
        if has_props {
            emit_macro_template_prop_bindings(
                ts,
                binding_mappings,
                props_object_binding,
                template_props_type_ref,
                props,
                defaulted_prop_names,
                &mut emitted_names,
            );
        }

        if should_emit_keyed_template_prop_bindings(summary, type_name, &emitted_names) {
            let names = collect_keyed_template_prop_names(summary, &emitted_names);
            if check_props && !names.is_empty() {
                // Keep literal public-instance members (including user plugin
                // augmentations), but never let an index signature hide typos.
                append!(
                    *ts,
                    "  const __vize_keyed_template_context = {props_object_binding} as typeof {props_object_binding} & {{ [K in keyof __Ctx as string extends K ? never : number extends K ? never : symbol extends K ? never : K]: __Ctx[K] }};\n  void __vize_keyed_template_context;\n"
                );
            }
            for name in names {
                if check_props {
                    emit_keyed_template_prop_binding(
                        ts,
                        binding_mappings,
                        template_props_type_ref,
                        name.as_str(),
                        defaulted_prop_names.contains(&name),
                    );
                } else {
                    emit_unchecked_template_prop_binding(ts, props_object_binding, name.as_str());
                }
                keyed_names.insert(name);
            }
        }
    } else if has_props {
        // Runtime-declared props: generate individual variables
        emit_macro_template_prop_bindings(
            ts,
            binding_mappings,
            props_object_binding,
            template_props_type_ref,
            props,
            defaulted_prop_names,
            &mut emitted_names,
        );
    }
    for model in models {
        if emitted_names.contains(model.name.as_str())
            || should_skip_template_prop_binding(summary, model.name.as_str())
        {
            continue;
        }
        binding_mappings.emit(
            ts,
            props_object_binding,
            template_props_type_ref,
            model.name.as_str(),
            model.default_value.is_some(),
        );
        emitted_names.insert(model.name.as_str().into());
    }
    ts.push('\n');
    keyed_names
}

fn emit_keyed_template_prop_binding(
    ts: &mut String,
    binding_mappings: &mut PropBindingMappings<'_>,
    props_type_ref: &str,
    prop_name: &str,
    has_default: bool,
) {
    let binding_name = to_safe_identifier(prop_name);
    let start = ts.len() + "  const ".len();
    if has_default {
        append!(
            *ts,
            "  const {binding_name} = __vize_keyed_template_context[(\"{prop_name}\" satisfies keyof typeof __vize_keyed_template_context)] as Exclude<{props_type_ref}[\"{prop_name}\"], undefined>;\n"
        );
    } else {
        append!(
            *ts,
            "  const {binding_name} = __vize_keyed_template_context[(\"{prop_name}\" satisfies keyof typeof __vize_keyed_template_context)];\n"
        );
    }
    append!(*ts, "  void {binding_name};\n");
    let property_start = start + binding_name.len() + " = __vize_keyed_template_context[(\"".len();
    binding_mappings.link(
        start..start + binding_name.len(),
        property_start..property_start + prop_name.len(),
    );
}

fn emit_unchecked_template_prop_binding(
    ts: &mut String,
    props_object_binding: &str,
    prop_name: &str,
) {
    let binding_name = to_safe_identifier(prop_name);
    append!(
        *ts,
        "  const {binding_name} = ({props_object_binding} as Record<string, unknown>)[\"{prop_name}\"];\n"
    );
    append!(*ts, "  void {binding_name};\n");
}

fn should_emit_keyed_template_prop_bindings(
    summary: &Croquis,
    type_name: &str,
    emitted_names: &FxHashSet<String>,
) -> bool {
    if has_top_level_type_operator(type_name) {
        return true;
    }
    if is_plain_inline_type_literal(type_name) {
        return false;
    }
    let base_name = strip_generic_params(type_name).trim();
    if summary.types.definitions().has_interface_extends(base_name) {
        return true;
    }
    if let Some(body) = summary.types.definitions().resolve(base_name) {
        return has_top_level_type_operator(body.as_str())
            || !is_plain_inline_type_literal(body.as_str());
    }
    emitted_names.is_empty() && !summary.types.definitions().is_defined(base_name)
}
