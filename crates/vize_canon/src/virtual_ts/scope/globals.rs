//! Generation of undefined-reference checks and instance-global declarations.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_carton::FxHashSet;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;

use vize_croquis::{Croquis, ScopeKind};

use crate::virtual_ts::types::{VirtualTsOptions, VizeMapping};

use super::context::ScopeGenerationOptions;

mod instance;
pub(super) use instance::generate_instance_global_refs;

/// Handle undefined references from template.
///
/// The public-instance form is the Vue 3 Options API opt-in: it requires the
/// script rewrite to have actually declared the `__default__` alias the
/// instance conditional reads, and stays off for legacy Vue 2, whose default
/// export is a plain options object — not a constructor — so the conditional
/// would degrade to `{}` and turn every filter or runtime-resolved name into
/// a `TS2339` the pinned Vue 2 parity surfaces do not contain.
///
/// A `<script setup>` block alongside the plain script keeps the form off too:
/// there the plain script's default export is a partial options object
/// (`inheritAttrs`, `name`, ...) whose instance type describes none of the
/// template's real bindings, which live in the setup scope.
pub(super) fn generate_undefined_refs(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    template_offset: u32,
    options: &ScopeGenerationOptions<'_, '_>,
) {
    if summary.undefined_refs.is_empty() {
        return;
    }
    let resolve_on_instance = options.options_api
        && options.has_default_alias
        && !options.legacy_vue2
        && !has_script_setup(summary);
    let script_content = options.script_content;
    // Names the plain script declares itself resolve from an enclosing scope of
    // the template closure even when the analyzer never tracked them as
    // bindings (a `namespace` is the measured case). Those are not unknown
    // template names, so they keep the free-name emission: a property access on
    // the instance would invent a `TS2339` `vue-tsc` does not report.
    //
    // `None` turns the form off for the whole template: either this shape never
    // resolves one, or the script did not parse, in which case which names it
    // declares is unknown and the free-name emission is the only safe one.
    let script_scope_names = if resolve_on_instance {
        match script_content {
            Some(script) => script_top_level_binding_names(script),
            None => Some(FxHashSet::default()),
        }
    } else {
        None
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
        // A name a configured `globalTypes` entry or a CSS module already
        // declares in this scope keeps the free-name emission too: the `var` the
        // instance form hoists would redeclare it (`TS2451`).
        let on_instance = script_scope_names.as_ref().is_some_and(|names| {
            !names.contains(undef.name.as_str())
                && !is_declared_template_context_name(
                    undef.name.as_str(),
                    options.virtual_ts_options,
                )
        });

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

/// Whether the SFC authored a `<script setup>` block: its bindings are the
/// template's real surface, and the plain script's default export next to it
/// carries only the options the setup form cannot express.
fn has_script_setup(summary: &Croquis) -> bool {
    summary
        .scopes
        .iter()
        .any(|scope| matches!(scope.kind, ScopeKind::ScriptSetup))
}

/// Names bound at the plain script's own top level, including the TypeScript-only
/// declaration forms the analyzer does not model (`namespace`, `enum`,
/// `interface`, `type`). The generated module keeps the script verbatim in a
/// scope that encloses the template closure, so every one of these stays
/// resolvable from a template expression.
///
/// `<script lang="ts">` may hold JSX, which only the TSX dialect parses, so the
/// plain source type is tried first and TSX second. `None` means neither parsed
/// the script cleanly: its top-level names are then unknown, and no template
/// name may be classified against the public instance on a partial program.
fn script_top_level_binding_names(script: &str) -> Option<FxHashSet<String>> {
    for source_type in [SourceType::ts(), SourceType::tsx()] {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, script, source_type.with_module(true)).parse();
        if parsed.panicked || !parsed.diagnostics.is_empty() {
            continue;
        }
        let semantic = SemanticBuilder::new().build(&parsed.program).semantic;
        let scoping = semantic.scoping();
        let mut names = FxHashSet::default();
        for (name, _) in scoping.get_bindings(scoping.root_scope_id()) {
            names.insert(String::from(name.as_str()));
        }
        return Some(names);
    }
    None
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
