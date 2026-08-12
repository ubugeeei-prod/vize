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

use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;

use vize_croquis::{BindingMetadata, Croquis, analyzer::extract_identifiers_oxc};

use crate::virtual_ts::types::{VirtualTsOptions, VizeMapping};

use super::super::context::ScopeGenerationOptions;
use super::{is_declared_template_context_name, is_template_instance_global_name};

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
        scope_options.virtual_ts_options,
        scope_options.setup_spread_bindings,
    );
    for undef in &summary.undefined_refs {
        let src_start = (template_offset + undef.offset) as usize;
        let src_end = src_start + undef.name.len();
        emitter.emit(undef.name.as_str(), src_start, src_end);
    }

    for expr in &summary.template_expressions {
        for ident in extract_identifiers_oxc(expr.content.as_str()) {
            let name = ident.as_str();
            let Some(relative_offset) = expr.content.find(name) else {
                continue;
            };
            let src_start = (template_offset + expr.start) as usize + relative_offset;
            let src_end = src_start + name.len();
            emitter.emit(name, src_start, src_end);
        }
    }
}

struct InstanceGlobalRefsEmitter<'a> {
    ts: &'a mut String,
    mappings: &'a mut Vec<VizeMapping>,
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
    emitted_header: bool,
}

impl<'a> InstanceGlobalRefsEmitter<'a> {
    fn new(
        ts: &'a mut String,
        mappings: &'a mut Vec<VizeMapping>,
        summary: &'a Croquis,
        options: &'a VirtualTsOptions,
        synthetic_setup_bindings: &'a [String],
    ) -> Self {
        Self {
            ts,
            mappings,
            options,
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
            emitted_header: false,
        }
    }

    fn emit(&mut self, name: &str, src_start: usize, src_end: usize) {
        if !is_template_instance_global_name(name)
            || self.bindings.contains(name)
            || self.synthetic_setup_bindings.contains(name)
            || self.type_export_names.contains(name)
            || is_declared_template_context_name(name, self.options)
        {
            return;
        }

        let strict = self.options.strict_instance_globals;
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
            if !strict {
                self.ts.push_str(
                    "  type __VizeInstanceGlobal<K extends string> = K extends keyof __Ctx ? __Ctx[K] : any;\n",
                );
            }
            self.emitted_header = true;
        }

        let gen_start = self.ts.len();
        let stmt = match (strict, already_declared) {
            (false, _) => {
                cstr!("  const {name}: __VizeInstanceGlobal<'{name}'> = undefined as any;\n")
            }
            (true, false) => cstr!("  const {name} = __ctx.{name};\n"),
            (true, true) => cstr!("  void (__ctx.{name});\n"),
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
