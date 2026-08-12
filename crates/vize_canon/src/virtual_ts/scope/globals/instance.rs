//! Template references to `$`-prefixed component instance globals.
//!
//! Two forms are emitted. The permissive one declares each name once with a
//! conditional type that widens to `any` when the instance type does not carry
//! it, which keeps a project with no authoritative ambient declaration surface
//! free of invented diagnostics. The strict one, enabled when the project does
//! publish such a surface (currently Nuxt's generated `.nuxt` types), reads the
//! name off the template context instead, so a global nothing declares reports
//! the `TS2339` the Vue toolchain reports, once per authored occurrence.
//!
//! Names the caller already declared, including the ones mined out of a
//! generated `ComponentCustomProperties`, never reach either form: they are
//! filtered by `is_declared_template_context_name`.
//!
//! `$props` is neither form: it is the component's *own* prop contract, and
//! reading it off `ComponentPublicInstance` resolves that helper's default `P`,
//! which is `{}` — so every declared prop read through `$props` reported
//! `TS2339 … does not exist on type '{}'` (#4145). It is declared from the one
//! template prop model instead.

use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;

use vize_croquis::{BindingMetadata, Croquis, analyzer::extract_identifier_refs_oxc};

use crate::virtual_ts::helpers::is_vue2_instance_member;
use crate::virtual_ts::props::TemplatePropsModel;
use crate::virtual_ts::types::{VirtualTsOptions, VizeMapping};

use super::super::context::ScopeGenerationOptions;
use super::{
    has_script_setup, is_declared_template_context_name, is_template_instance_global_name,
};

#[cfg(test)]
mod tests;

pub(in crate::virtual_ts::scope) fn generate_instance_global_refs(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    template_offset: u32,
    scope_options: &ScopeGenerationOptions<'_, '_>,
) {
    if summary.undefined_refs.is_empty() && summary.template_expressions.is_empty() {
        return;
    }

    let mut emitter = InstanceGlobalRefsEmitter::new(
        ts,
        mappings,
        summary,
        scope_options,
        scope_options.setup_spread_bindings,
    );
    for undef in &summary.undefined_refs {
        let src_start = (template_offset + undef.offset) as usize;
        let src_end = src_start + undef.name.len();
        emitter.emit(undef.name.as_str(), src_start, src_end);
    }

    for expr in &summary.template_expressions {
        // Only a `$`-prefixed name survives `emit`, so an expression without a
        // `$` never needs a parse. A real template holds thousands of
        // interpolations and directive expressions and almost none of them read
        // an instance global, so the byte scan keeps the parse off that path.
        if !expr.content.as_str().contains('$') {
            continue;
        }
        // The identifier's own span, not `content.find(name)`: the first
        // occurrence of `$props` in `$propsList && $props` is the `$propsList`
        // prefix, and mapping to it would point diagnostics at the wrong
        // template range.
        for ident in extract_identifier_refs_oxc(expr.content.as_str()) {
            let name = ident.name.as_str();
            let src_start = (template_offset + expr.start + ident.offset) as usize;
            let src_end = src_start + name.len();
            emitter.emit(name, src_start, src_end);
        }
    }
}

/// The type the template's `$props` resolves against.
///
/// A `<script setup>` component reads it off the one template prop model, so
/// `$props`, the local `props` const and the bare per-prop bindings cannot drift
/// apart.
///
/// A plain Options API component declares no macro props, so its `$props` comes
/// off the authored default export's own instance type — the same value
/// `generate_undefined_refs` resolves unknown template names against, and under
/// the same conditions: a `<script setup>` block alongside the plain script owns
/// the template's bindings, and legacy Vue 2's default export is an options
/// object rather than a constructor.
///
/// `None` keeps the generic instance-global form, whose `{}` for a component
/// that declares no props is what `vue-tsc` reports too.
///
/// Resolved lazily: building the prop model walks the macro and type tables, and
/// the overwhelming majority of templates never name `$props` at all.
fn instance_props_type(
    summary: &Croquis,
    scope_options: &ScopeGenerationOptions<'_, '_>,
) -> Option<String> {
    if let Some(props_type) = TemplatePropsModel::new(summary).instance_props_type() {
        return Some(props_type);
    }
    let resolve_on_instance = scope_options.options_api
        && scope_options.has_default_alias
        && !scope_options.legacy_vue2
        && !has_script_setup(summary);
    resolve_on_instance.then(|| {
        String::from(
            "(typeof __default__ extends abstract new (...args: any) => infer __I ? __I extends { $props: infer __P } ? __P : {} : {})",
        )
    })
}

struct InstanceGlobalRefsEmitter<'a> {
    ts: &'a mut String,
    mappings: &'a mut Vec<VizeMapping>,
    summary: &'a Croquis,
    scope_options: &'a ScopeGenerationOptions<'a, 'a>,
    options: &'a VirtualTsOptions,
    bindings: &'a BindingMetadata,
    synthetic_setup_bindings: FxHashSet<&'a str>,
    type_export_names: FxHashSet<&'a str>,
    seen_names: FxHashSet<String>,
    /// Authored `(name, source offset)` pairs already emitted. Strict resolution
    /// reports one diagnostic per authored occurrence the way `vue-tsc` does, so
    /// it deduplicates by position; the two collection passes above can reach the
    /// same occurrence twice.
    seen_occurrences: FxHashSet<(String, usize)>,
    /// Memoized `instance_props_type`; the outer `Option` is "not resolved yet".
    instance_props_type: Option<Option<String>>,
    emitted_header: bool,
    emitted_instance_global_alias: bool,
}

impl<'a> InstanceGlobalRefsEmitter<'a> {
    fn new(
        ts: &'a mut String,
        mappings: &'a mut Vec<VizeMapping>,
        summary: &'a Croquis,
        scope_options: &'a ScopeGenerationOptions<'a, 'a>,
        synthetic_setup_bindings: &'a [String],
    ) -> Self {
        Self {
            ts,
            mappings,
            summary,
            scope_options,
            options: scope_options.virtual_ts_options,
            bindings: &summary.bindings,
            synthetic_setup_bindings: synthetic_setup_bindings
                .iter()
                .map(|name| name.as_str())
                .collect(),
            type_export_names: summary
                .type_exports
                .iter()
                .map(|te| te.name.as_str())
                .collect(),
            seen_names: FxHashSet::default(),
            seen_occurrences: FxHashSet::default(),
            instance_props_type: None,
            emitted_header: false,
            emitted_instance_global_alias: false,
        }
    }

    fn resolve_instance_props_type(&mut self) -> Option<String> {
        self.instance_props_type
            .get_or_insert_with(|| instance_props_type(self.summary, self.scope_options))
            .clone()
    }

    fn emit(&mut self, name: &str, src_start: usize, src_end: usize) {
        if !is_template_instance_global_name(name)
            || self.bindings.contains(name)
            || self.synthetic_setup_bindings.contains(name)
            || self.type_export_names.contains(name)
            || is_declared_template_context_name(name, self.options)
            || (self.scope_options.legacy_vue2 && is_vue2_instance_member(name))
        {
            return;
        }

        // `$props` resolves against the component's own prop contract, and is
        // declared once whatever the strictness: strict resolution exists to
        // report an *undeclared* global, and `$props` is always declared.
        let declared_type = if name == "$props" {
            self.resolve_instance_props_type()
        } else {
            None
        };

        let strict = self.options.strict_instance_globals && declared_type.is_none();
        // Strict resolution keeps every authored occurrence so an undeclared
        // name reports once per use, the way `vue-tsc` does. The permissive form
        // declares the name once and every later use resolves to that binding.
        let already_declared = if strict {
            if !self.seen_occurrences.insert((name.into(), src_start)) {
                return;
            }
            !self.seen_names.insert(name.into())
        } else {
            if !self.seen_names.insert(name.into()) {
                return;
            }
            false
        };

        if !self.emitted_header {
            self.ts
                .push_str("\n  // Instance globals from ComponentPublicInstance:\n");
            self.emitted_header = true;
        }
        // The `__VizeInstanceGlobal` alias is only emitted for names that
        // actually read the public instance: an alias declared and never used
        // is a `TS6196` under `noUnusedLocals`.
        if !strict && declared_type.is_none() && !self.emitted_instance_global_alias {
            self.ts.push_str(
                "  type __VizeInstanceGlobal<K extends string> = K extends keyof __Ctx ? __Ctx[K] : any;\n",
            );
            self.emitted_instance_global_alias = true;
        }

        let gen_start = self.ts.len();
        let stmt = match (declared_type.as_deref(), strict, already_declared) {
            (Some(props_type), _, _) => {
                cstr!("  const {name}: {props_type} = undefined as any;\n")
            }
            (None, false, _) => {
                cstr!("  const {name}: __VizeInstanceGlobal<'{name}'> = undefined as any;\n")
            }
            (None, true, false) => cstr!("  const {name} = __ctx.{name};\n"),
            (None, true, true) => cstr!("  void (__ctx.{name});\n"),
        };
        // The strict forms read the name off `__ctx`, and that access is where
        // TypeScript reports an undeclared global, so the mapping anchors to the
        // trailing occurrence rather than the leading binding name.
        let name_offset = if strict {
            stmt.rfind(name)
        } else {
            stmt.find(name)
        };
        let gen_name_start = gen_start + name_offset.unwrap_or(0);
        let gen_name_end = gen_name_start + name.len();

        self.ts.push_str(&stmt);
        self.mappings.push(VizeMapping {
            gen_range: gen_name_start..gen_name_end,
            src_range: src_start..src_end,
            sub_spans: Vec::new(),
        });
        append!(
            *self.ts,
            "  // @vize-map: {gen_name_start}:{gen_name_end} -> {src_start}:{src_end}\n",
        );
    }
}
