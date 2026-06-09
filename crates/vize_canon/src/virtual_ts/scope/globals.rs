//! Generation of undefined-reference checks and instance-global declarations.

use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;

use vize_croquis::{BindingMetadata, Croquis, analyzer::extract_identifiers_oxc};

use crate::virtual_ts::types::{VirtualTsOptions, VizeMapping};

/// Handle undefined references from template.
pub(super) fn generate_undefined_refs(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    template_offset: u32,
) {
    if summary.undefined_refs.is_empty() {
        return;
    }

    // Collect type export names to exclude from undefined refs
    let type_export_names: FxHashSet<&str> = summary
        .type_exports
        .iter()
        .map(|te| te.name.as_str())
        .collect();

    let mut seen_names: FxHashSet<&str> = FxHashSet::default();
    let mut emitted_header = false;
    for undef in &summary.undefined_refs {
        if !seen_names.insert(undef.name.as_str()) {
            continue;
        }
        if is_template_instance_global_name(undef.name.as_str()) {
            continue;
        }
        // Skip names that match type exports (these are type-level, not value-level)
        if type_export_names.contains(undef.name.as_str()) {
            continue;
        }

        let src_start = (template_offset + undef.offset) as usize;
        let src_end = src_start + undef.name.len();

        if !emitted_header {
            ts.push_str("\n  // Undefined references from template:\n");
            emitted_header = true;
        }

        let gen_start = ts.len();
        // Use void expression to reference the name without creating an unused variable
        let expr_code = cstr!("  void ({});\n", undef.name);
        let name_offset = expr_code.find(undef.name.as_str()).unwrap_or(0);
        let gen_name_start = gen_start + name_offset;
        let gen_name_end = gen_name_start + undef.name.len();

        ts.push_str(&expr_code);
        mappings.push(VizeMapping {
            gen_range: gen_name_start..gen_name_end,
            src_range: src_start..src_end,
            sub_spans: Vec::new(),
        });
        append!(
            *ts,
            "  // @vize-map: {gen_name_start}:{gen_name_end} -> {src_start}:{src_end}\n",
        );
    }
}

pub(super) fn generate_instance_global_refs(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    template_offset: u32,
    options: &VirtualTsOptions,
) {
    if summary.undefined_refs.is_empty() && summary.template_expressions.is_empty() {
        return;
    }

    let mut emitter = InstanceGlobalRefsEmitter::new(ts, mappings, summary, options);
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
    type_export_names: FxHashSet<&'a str>,
    seen_names: FxHashSet<String>,
    emitted_header: bool,
}

impl<'a> InstanceGlobalRefsEmitter<'a> {
    fn new(
        ts: &'a mut String,
        mappings: &'a mut Vec<VizeMapping>,
        summary: &'a Croquis,
        options: &'a VirtualTsOptions,
    ) -> Self {
        Self {
            ts,
            mappings,
            options,
            bindings: &summary.bindings,
            type_export_names: summary
                .type_exports
                .iter()
                .map(|te| te.name.as_str())
                .collect(),
            seen_names: FxHashSet::default(),
            emitted_header: false,
        }
    }

    fn emit(&mut self, name: &str, src_start: usize, src_end: usize) {
        if !is_template_instance_global_name(name)
            || self.bindings.contains(name)
            || self.type_export_names.contains(name)
            || is_declared_template_context_name(name, self.options)
            || !self.seen_names.insert(name.into())
        {
            return;
        }

        if !self.emitted_header {
            self.ts
                .push_str("\n  // Instance globals from ComponentPublicInstance:\n");
            self.ts.push_str(
                "  type __VizeInstanceGlobal<K extends string> = K extends keyof __Ctx ? __Ctx[K] : any;\n",
            );
            self.emitted_header = true;
        }

        let gen_start = self.ts.len();
        let stmt = cstr!("  const {name}: __VizeInstanceGlobal<'{name}'> = undefined as any;\n");
        let gen_name_start = gen_start + stmt.find(name).unwrap_or(0);
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

fn is_template_instance_global_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix('$') else {
        return false;
    };
    !rest.is_empty()
        && rest
            .chars()
            .all(|c| c == '_' || c == '$' || c.is_ascii_alphanumeric())
}

fn is_declared_template_context_name(name: &str, options: &VirtualTsOptions) -> bool {
    matches!(name, "$attrs" | "$slots" | "$refs" | "$emit" | "$event")
        || options
            .template_globals
            .iter()
            .any(|global| global.name.as_str() == name)
        || options
            .css_modules
            .iter()
            .any(|module_name| module_name.as_str() == name)
}
