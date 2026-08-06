//! Generation of undefined-reference checks and instance-global declarations.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;

use vize_croquis::{BindingMetadata, Croquis, analyzer::extract_identifiers_oxc};

use crate::virtual_ts::types::{VirtualTsOptions, VizeMapping};

use super::context::ScopeGenerationOptions;

/// Handle undefined references from template.
pub(super) fn generate_undefined_refs(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    template_offset: u32,
    options_api: bool,
    script_content: Option<&str>,
) {
    if summary.undefined_refs.is_empty() {
        return;
    }
    // The instance form leans on the `__default__` alias the script rewrite
    // declares; an Options API shape without one (no plain default export)
    // keeps the free-name emission rather than minting an unresolved alias.
    let options_api = options_api && ts.contains("__default__");
    // Names the plain script declares itself resolve from an enclosing scope of
    // the template closure even when the analyzer never tracked them as
    // bindings (a `namespace` is the measured case). Those are not unknown
    // template names, so they keep the free-name emission: a property access on
    // the instance would invent a `TS2339` `vue-tsc` does not report.
    let script_scope_names = if options_api {
        script_content.map_or_else(FxHashSet::default, script_top_level_binding_names)
    } else {
        FxHashSet::default()
    };

    // Collect type export names to exclude from undefined refs
    let type_export_names: FxHashSet<&str> = summary
        .type_exports
        .iter()
        .map(|te| te.name.as_str())
        .collect();

    let mut seen_names: FxHashSet<&str> = FxHashSet::default();
    let mut emitted_header = false;
    let mut emitted_instance = false;
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
        let on_instance = options_api && !script_scope_names.contains(undef.name.as_str());

        if !emitted_header {
            ts.push_str("\n  // Undefined references from template:\n");
            emitted_header = true;
        }
        if on_instance && !emitted_instance {
            // A value typed as the public instance: the missing key reports
            // `TS2339` naming the instance type (the code and shape `vue-tsc`
            // emits for Options API templates), while the declared binding lets
            // the interpolation expression itself resolve, so the bare `TS2304`
            // disappears instead of doubling up (#3888).
            ts.push_str(
                "  const __vize_template_instance = undefined as unknown as (typeof __default__ extends abstract new (...args: any) => infer __I ? __I : {});\n  void __vize_template_instance;\n",
            );
            emitted_instance = true;
        }

        let gen_start = ts.len();
        // Use void expression to reference the name without creating an unused variable
        let expr_code = if on_instance {
            cstr!(
                "  var {}: any = undefined; void ({});\n  void (__vize_template_instance.{});\n",
                undef.name,
                undef.name,
                undef.name
            )
        } else {
            cstr!("  void ({});\n", undef.name)
        };
        let name_offset = if on_instance {
            expr_code.rfind(undef.name.as_str()).unwrap_or(0)
        } else {
            expr_code.find(undef.name.as_str()).unwrap_or(0)
        };
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

/// Names bound at the plain script's own top level, including the TypeScript-only
/// declaration forms the analyzer does not model (`namespace`, `enum`,
/// `interface`, `type`). The generated module keeps the script verbatim in a
/// scope that encloses the template closure, so every one of these stays
/// resolvable from a template expression.
fn script_top_level_binding_names(script: &str) -> FxHashSet<String> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts().with_module(true)).parse();
    if parsed.panicked {
        return FxHashSet::default();
    }
    let semantic = SemanticBuilder::new().build(&parsed.program).semantic;
    let scoping = semantic.scoping();
    let mut names = FxHashSet::default();
    for (name, _) in scoping.get_bindings(scoping.root_scope_id()) {
        names.insert(String::from(name.as_str()));
    }
    names
}

pub(super) fn generate_instance_global_refs(
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
            emitted_header: false,
        }
    }

    fn emit(&mut self, name: &str, src_start: usize, src_end: usize) {
        if !is_template_instance_global_name(name)
            || self.bindings.contains(name)
            || self.synthetic_setup_bindings.contains(name)
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
