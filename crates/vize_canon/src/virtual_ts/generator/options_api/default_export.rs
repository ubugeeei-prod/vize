//! Default-export span classification for Options API virtual TypeScript.

use oxc_allocator::Allocator;
use oxc_ast::ast::{ExportDefaultDeclarationKind, Program, Statement};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{FxHashSet, String};

use super::{
    component_options_from_program, computed::writable_computed_names, has_unresolved_extends,
};

/// Owned results shared by default-export rewriting and Options API bindings.
#[derive(Default)]
pub(in crate::virtual_ts::generator) struct OptionsApiScriptFacts {
    pub default_export: DefaultExportTargets,
    pub writable_computed: FxHashSet<String>,
    pub has_unresolved_extends: bool,
}

/// Derive the Options API facts from the existing default-export parse.
pub(in crate::virtual_ts::generator) fn analyze_options_api_script(
    script: &str,
    classify_default_export: bool,
    options_api: bool,
) -> OptionsApiScriptFacts {
    let mut facts = OptionsApiScriptFacts::default();
    // Preserve the existing default-export-only no-export fast path.
    if !options_api && !script.contains("export default") {
        return facts;
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, script, SourceType::ts()).parse();
    if parsed.panicked {
        return facts;
    }
    if classify_default_export {
        facts.default_export = default_export_targets(script, &parsed.program);
    }
    if options_api && let Some(options) = component_options_from_program(&parsed.program) {
        facts.writable_computed = writable_computed_names(&parsed.program, options);
        facts.has_unresolved_extends = has_unresolved_extends(script, &parsed.program, options);
    }
    facts
}

/// Byte offsets locating the rewriteable shape of a `<script>` default export.
///
/// All fields are offsets into the parsed `script`. A single default export is
/// at most one of these (an SFC module has one default export), so at most one
/// field is `Some`.
#[derive(Default, Clone, Copy)]
pub(in crate::virtual_ts::generator) struct DefaultExportTargets {
    /// A plain object-literal default export (`export default { ... }`) — the
    /// Options API shape — as `(export_start, object_start, object_end)`. Used
    /// to wrap the object in `defineComponent` so `this` in computed/methods
    /// gets Vue's instance typing. Anything else (already-wrapped
    /// `defineComponent({...})`, identifiers, calls, `as`/`satisfies`) stays
    /// `None` so only the bare options object is wrapped.
    pub object: Option<(usize, usize, usize)>,
    /// A class-declaration default export (`export default class Foo {}`, the
    /// class-component shape — vue-class-component / vue-property-decorator) as
    /// `(export_start, class_start, class_end, name_start, name_end)`.
    /// `export_start..class_start` is the `export default ` keyword (stripped);
    /// `class_start..class_end` is the class declaration; `name_start..name_end`
    /// is the class identifier. Decorators written before `export default` sit
    /// ahead of `export_start`; decorators after it fall inside the class span —
    /// so stripping only the keyword run keeps `@Component()` on a real class
    /// declaration either way (the line-based fallback would move it onto a
    /// `const`, which TypeScript rejects with TS1206). Anonymous default classes
    /// stay `None` (no name to alias by) and fall through to the generic
    /// `expr` rewrite below.
    pub class: Option<(usize, usize, usize, usize, usize)>,
    /// Any other default-export shape, rewritten to a bare
    /// `const __default__ = <expr>` at module scope, as
    /// `(export_start, expr_start, expr_end)`. Covers
    /// `export default defineComponent({...})`, identifiers, parenthesized /
    /// `as` / `satisfies` expressions, anonymous classes/functions, and
    /// `export default{` with no space — including multi-line / awkwardly
    /// formatted variants. `export_start..expr_start` is the `export default`
    /// keyword run that is dropped; `expr_start..expr_end` is the exported
    /// expression copied verbatim. This is the span-based replacement for the
    /// former line-scanning fallback, so it is only populated when neither
    /// `object` nor `class` applies.
    pub expr: Option<(usize, usize, usize)>,
}

/// The rewrite tests use the same parse and classification as the generator.
#[cfg(test)]
pub(in crate::virtual_ts::generator) fn find_default_export_targets(
    script: &str,
) -> DefaultExportTargets {
    analyze_options_api_script(script, true, false).default_export
}

fn default_export_targets(script: &str, program: &Program<'_>) -> DefaultExportTargets {
    let mut targets = DefaultExportTargets::default();
    if !script.contains("export default") {
        return targets;
    }
    for statement in program.body.iter() {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            continue;
        };
        match &export.declaration {
            ExportDefaultDeclarationKind::ObjectExpression(object) => {
                let object_span = object.span();
                targets.object = Some((
                    export.span.start as usize,
                    object_span.start as usize,
                    object_span.end as usize,
                ));
            }
            ExportDefaultDeclarationKind::ClassDeclaration(class) if let Some(id) = &class.id => {
                targets.class = Some((
                    export.span.start as usize,
                    class.span.start as usize,
                    class.span.end as usize,
                    id.span.start as usize,
                    id.span.end as usize,
                ));
            }
            // Every other default-export shape (already-wrapped
            // `defineComponent(...)`, identifiers, `as`/`satisfies`,
            // anonymous classes/functions, ...) is rewritten verbatim to a
            // bare `const __default__ = <expr>` using the declaration span.
            // Slicing on these AST offsets is correct regardless of source
            // formatting (`export default{` with no space, multi-line calls),
            // which the previous line scanner mishandled.
            other => {
                let declaration_span = other.span();
                targets.expr = Some((
                    export.span.start as usize,
                    declaration_span.start as usize,
                    declaration_span.end as usize,
                ));
            }
        }
        // A module has a single default export; stop at the first one.
        break;
    }
    targets
}

#[cfg(test)]
mod tests {
    use super::analyze_options_api_script;
    use vize_carton::{FxHashSet, String};

    #[test]
    fn shared_options_facts_preserve_export_and_repeated_mixin_precedence() {
        let script = r#"
export const shared = { computed: { ratio: { get() { return 1 }, set(value) {} } } };
const readonly = { computed: { ratio() { return 2 } } };
export default {
    extends: ImportedBase,
    mixins: [shared, readonly, shared],
    computed: { own: { get() { return 3 }, set(value) {} }, local() { return 4 } }
}
"#;
        let facts = analyze_options_api_script(script, true, true);
        let (export_start, start, end) = facts.default_export.object.unwrap();
        assert_eq!(&script[export_start..start], "export default ");
        assert_eq!(&script[start..end], script[start..].trim_end());
        assert!(facts.default_export.class.is_none());
        assert!(facts.default_export.expr.is_none());
        assert!(facts.has_unresolved_extends);
        assert_eq!(
            facts.writable_computed,
            FxHashSet::from_iter([String::from("own"), String::from("ratio")])
        );
    }

    #[test]
    fn rewrite_and_options_binding_gates_remain_independent() {
        let script = "export default { extends: ImportedBase, computed: { ratio: { set(v) {} } } }";
        let rewrite_only = analyze_options_api_script(script, true, false);
        assert!(rewrite_only.default_export.object.is_some());
        assert!(rewrite_only.writable_computed.is_empty());
        assert!(!rewrite_only.has_unresolved_extends);
        let options_only = analyze_options_api_script(script, false, true);
        assert!(options_only.default_export.object.is_none());
        assert!(options_only.default_export.class.is_none());
        assert!(options_only.default_export.expr.is_none());
        assert_eq!(
            options_only.writable_computed,
            FxHashSet::from_iter([String::from("ratio")])
        );
        assert!(options_only.has_unresolved_extends);
    }

    #[test]
    fn authored_export_spacing_keeps_unconditional_computed_analysis() {
        for keyword in ["export\n default", "export /* gap */ default"] {
            let script = format!(
                "{keyword} {{ extends: ImportedBase, computed: {{ ratio: {{ set(v) {{}} }} }} }}"
            );
            let facts = analyze_options_api_script(&script, true, true);
            assert_eq!(
                facts.writable_computed,
                FxHashSet::from_iter([String::from("ratio")])
            );
            // These two existing helpers keep their original literal guards.
            assert!(facts.default_export.object.is_none());
            assert!(!facts.has_unresolved_extends);
        }
    }
}
