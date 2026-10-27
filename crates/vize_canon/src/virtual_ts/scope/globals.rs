//! Generation of undefined-reference checks and instance-global declarations.

use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_carton::{FxHashSet, String, append, cstr};

use vize_croquis::{Croquis, ScopeKind};

use crate::virtual_ts::types::{VirtualTsOptions, VizeMapping};

use super::context::ScopeGenerationOptions;

mod authored_ranges;
mod instance;
mod loop_sources;
pub(super) use instance::generate_instance_global_refs;
mod member_root;
mod strict_candidate;
mod strict_expression;
mod template_scope;
use {
    member_root::{is_call_root_occurrence, is_member_root_occurrence},
    strict_candidate::is_strict_template_context_candidate,
    strict_expression::generate_strict_expression_refs,
    template_scope::is_visible_template_binding,
};

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
///
/// Every other Vue 3 template reads its unknown names off the template
/// context's public instance (`__VizeTemplateInstance`, see
/// [`PUBLIC_INSTANCE_PRELUDE`]): a `ComponentCustomProperties` member resolves
/// with its declared type, and a name nothing declares reports once per
/// authored occurrence with the instance wording. Legacy Vue 2 and the strict
/// instance-global form keep their own emission.
pub(super) fn generate_undefined_refs(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    template_binding_access: &TemplateBindingAccess,
    template_offset: u32,
    options: &ScopeGenerationOptions<'_, '_>,
) {
    let check_keyed = options.check_options.check_props
        && !options.legacy_vue2
        && !options.keyed_prop_bindings.is_empty();
    let undefined_refs = super::super::script_facts::undefined_refs(summary);
    if undefined_refs.is_empty()
        && ((!options.virtual_ts_options.strict_instance_globals && !check_keyed)
            || summary.template_expressions.is_empty())
    {
        return;
    }
    let resolve_on_instance = options.options_api
        && options.has_default_alias
        && !options.legacy_vue2
        && !has_script_setup(summary);
    // Every other Vue 3 template reads an unknown name off the public instance
    // its context is typed as (`__Ctx`, Vue's `ComponentPublicInstance`), the
    // way `vue-tsc` reads it off its template context. A member a global mixin
    // or a plugin declares through `ComponentCustomProperties` then resolves
    // with its declared type, while a name nothing declares still reports once
    // per authored occurrence. Legacy Vue 2 keeps the free-name form: its
    // context is a structural fallback that carries none of the instance
    // surface. The strict instance-global form (a project with generated
    // types, Nuxt) keeps resolving names on its own strict context.
    let resolve_on_public_instance = !resolve_on_instance
        && !options.legacy_vue2
        && !options.virtual_ts_options.strict_instance_globals;
    let undefined_refs = if resolve_on_public_instance {
        loop_sources::authored_references(summary, undefined_refs)
    } else {
        undefined_refs
    };
    let script_content = options.script_content;
    // Names the script declares itself resolve from an enclosing scope of the
    // template closure even when the analyzer never tracked them as bindings
    // (a `namespace` is the measured case). Those are not unknown template
    // names, so they keep the free-name emission: a property access on the
    // instance would invent a `TS2339` `vue-tsc` does not report.
    //
    // `None` turns both instance forms off for the whole template: either this
    // shape never resolves one, or the script did not parse, in which case
    // which names it declares is unknown and the free-name emission is the
    // only safe one.
    let script_scope_names = if resolve_on_instance || resolve_on_public_instance {
        match script_content {
            Some(script) => script_top_level_binding_names(script),
            None => Some(FxHashSet::default()),
        }
    } else {
        None
    };

    let type_export_names: FxHashSet<&str> = summary
        .type_exports
        .iter()
        .map(|te| te.name.as_str())
        .collect();

    let mut seen_names: FxHashSet<String> = FxHashSet::default();
    let mut seen_strict_occurrences: FxHashSet<(String, usize)> = FxHashSet::default();
    let mut emitted_header = false;
    let mut emitted_instance = false;
    let mut emitted_public_instance = false;
    let authored_ranges = authored_ranges::collect(mappings);
    let interpolations = authored_ranges::interpolations(summary, template_offset);
    for undef in &undefined_refs {
        let name = undef.name.as_str();
        if !is_strict_template_context_candidate(name) {
            continue;
        }
        if is_template_instance_global_name(undef.name.as_str()) {
            continue;
        }
        if template_binding_access.contains(name)
            || is_visible_template_binding(summary, name, undef.offset)
        {
            continue;
        }
        // Skip names that match type exports (these are type-level, not value-level)
        if type_export_names.contains(name) {
            continue;
        }

        let src_start = (template_offset + undef.offset) as usize;
        let src_end = src_start + undef.name.len();
        // A name a configured `globalTypes` entry or a CSS module already
        // declares in this scope keeps the free-name emission too: the `var` the
        // instance form hoists would redeclare it (`TS2451`).
        let script_declared = script_scope_names
            .as_ref()
            .is_some_and(|names| names.contains(name));
        let context_declared = is_declared_template_context_name(name, options.virtual_ts_options);
        // A keyed prop local is declared by the props projection (an opaque
        // props type hides the name from the analyzer); it resolves lexically.
        let unknown_to_scope = script_scope_names.is_some()
            && !script_declared
            && !context_declared
            && !options.keyed_prop_bindings.contains(name);
        let on_instance = resolve_on_instance && unknown_to_scope;
        let member_root = is_member_root_occurrence(summary, undef.offset, name);
        let call_root = is_call_root_occurrence(summary, undef.offset, name);
        let strict_ref_shape = member_root || call_root;
        let keyed = check_keyed && !context_declared && options.keyed_prop_bindings.contains(name);
        let on_strict_template_context = keyed
            || options.virtual_ts_options.strict_instance_globals
                && !on_instance
                && !script_declared
                && !context_declared
                && strict_ref_shape;
        let on_public_instance =
            resolve_on_public_instance && unknown_to_scope && !on_strict_template_context;

        // Interpolations are checked as authored expressions in their lexical
        // scope. A duplicate read would escape their authored TS directives.
        // Control-flow guards still need the fallback: some generated guard
        // copies exist only for narrowing and deliberately suppress diagnostics.
        if !on_instance
            && !on_public_instance
            && !on_strict_template_context
            && authored_ranges::contains(&interpolations, src_start)
            && authored_ranges::contains(&authored_ranges, src_start)
        {
            continue;
        }

        if on_strict_template_context {
            if !seen_strict_occurrences.insert((undef.name.clone(), src_start)) {
                continue;
            }
            let keyed = options.keyed_prop_bindings.contains(name);
            let already_declared = member_root || keyed || !seen_names.insert(String::from(name));
            emit_header(ts, &mut emitted_header);
            emit_strict_template_context_ref(
                ts,
                mappings,
                (
                    if check_keyed && keyed {
                        "__vize_keyed_template_context"
                    } else {
                        strict_receiver(keyed)
                    },
                    name,
                ),
                src_start,
                src_end,
                already_declared,
            );
            continue;
        }
        if on_public_instance {
            // One read per authored occurrence, the way `vue-tsc` reports an
            // undeclared name, and one hoisted declaration per name so the
            // authored expressions resolve it with the member's type (`any`
            // when the instance does not carry it, so a missing member reports
            // exactly once per use, from the read below).
            if !seen_strict_occurrences.insert((undef.name.clone(), src_start)) {
                continue;
            }
            emit_header(ts, &mut emitted_header);
            if !emitted_public_instance {
                ts.push_str(PUBLIC_INSTANCE_PRELUDE);
                emitted_public_instance = true;
            }
            if seen_names.insert(String::from(name)) {
                append!(
                    *ts,
                    "  var {name}!: __VizeTemplateInstanceMember<\"{name}\">; void ({name});\n"
                );
            }
            emit_strict_template_context_ref(
                ts,
                mappings,
                (PUBLIC_INSTANCE_RECEIVER, name),
                src_start,
                src_end,
                true,
            );
            continue;
        }
        if !seen_names.insert(String::from(name)) {
            continue;
        }

        emit_header(ts, &mut emitted_header);
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
        let gen_start = ts.len();
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

    if (options.virtual_ts_options.strict_instance_globals || check_keyed) && !resolve_on_instance {
        generate_strict_expression_refs(
            ts,
            mappings,
            summary,
            template_offset,
            options.virtual_ts_options,
            template_binding_access,
            options.keyed_prop_bindings,
            check_keyed,
            &type_export_names,
            &mut seen_names,
            &mut seen_strict_occurrences,
            &mut emitted_header,
        );
    }
}

/// The value a Vue 3 template without an Options API default export reads its
/// unknown names off. The mapped alias keeps the public instance's members and
/// gives the missing-property diagnostic a type name of its own
/// (`__VizeTemplateInstance`), which the projection rewrites to the component
/// instance wording the lexical form reports. `__Ctx` is the template context
/// type, so a `vue` without `ComponentPublicInstance` degrades to an index
/// signature and reports nothing, as the context itself does.
const PUBLIC_INSTANCE_PRELUDE: &str = "  type __VizeTemplateInstance = { [__K in keyof __Ctx]: __Ctx[__K] };\n  type __VizeTemplateInstanceMember<__K extends string> = __K extends keyof __VizeTemplateInstance ? __VizeTemplateInstance[__K] : any;\n  const __vize_public_instance = undefined as unknown as __VizeTemplateInstance;\n  void __vize_public_instance;\n";

const PUBLIC_INSTANCE_RECEIVER: &str = "__vize_public_instance";

pub(super) fn emit_header(ts: &mut String, emitted_header: &mut bool) {
    if !*emitted_header {
        ts.push_str("\n  // Undefined references from template:\n");
        *emitted_header = true;
    }
}

/// A keyed props local is declared from an opaque props type the analyzer
/// cannot list, so the typed props object, not the strict instance context,
/// decides whether the template name exists.
pub(super) fn strict_receiver(keyed_prop: bool) -> &'static str {
    if keyed_prop {
        "props"
    } else {
        "__vize_strict_template_context"
    }
}

pub(super) fn emit_strict_template_context_ref(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    (receiver, name): (&str, &str),
    src_start: usize,
    src_end: usize,
    already_declared: bool,
) {
    let expr_code = if already_declared {
        cstr!("  void ({receiver}.{name});\n")
    } else {
        cstr!("  var {name}: any = undefined; void ({name});\n  void ({receiver}.{name});\n")
    };
    let gen_start = ts.len();
    let name_offset = expr_code.rfind(name).unwrap_or(0);
    let gen_name_start = gen_start + name_offset;
    let gen_name_end = gen_name_start + name.len();
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

pub(super) fn is_template_instance_global_name(name: &str) -> bool {
    let Some(rest) = name.strip_prefix('$') else {
        return false;
    };
    // Vue 3 exposes the internal instance as `$` itself, in addition to
    // named public-instance members such as `$attrs` and `$refs`.
    rest.chars()
        .all(|c| c == '_' || c == '$' || c.is_ascii_alphanumeric())
}

pub(super) fn is_declared_template_context_name(name: &str, options: &VirtualTsOptions) -> bool {
    matches!(name, "$attrs" | "$slots" | "$refs" | "$emit" | "$event")
        || options
            .template_globals
            .iter()
            .any(|global| global.name.as_str() == name)
        || options
            .css_modules
            .iter()
            .any(|module_name| module_name.as_str() == name)
        || options.has_auto_import_binding_name(name)
        || options
            .external_template_bindings
            .iter()
            .any(|binding| binding.as_str() == name)
}
