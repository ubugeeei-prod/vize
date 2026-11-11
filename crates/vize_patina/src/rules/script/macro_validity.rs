//! The checks `script/valid-define-emits` and `script/valid-define-props`
//! share, following eslint-plugin-vue's `valid-define-emits` and
//! `valid-define-props`.
//!
//! Only top-level macros count (see [`super::setup_macros`]). A runtime
//! argument may not reference a binding of this `<script setup>`: each
//! identifier is resolved through scope, so a parameter of a function inside
//! the argument, an import and a type position (`typeof x`, `PropType<T>`) do
//! not count, while an `enum` or `namespace` does.

use oxc_ast::ast::{
    Argument, CallExpression, ExportDefaultDeclarationKind, Expression, IdentifierReference,
    ObjectPropertyKind, Program, Statement, TSType,
};
use oxc_ast_visit::Visit;
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::{GetSpan, Span};

use super::SfcScriptContext;
use super::setup_macros::setup_macros;

/// One finding, at the span upstream reports.
pub(super) enum Finding {
    /// More than one macro call; every call is reported.
    Multiple(Span),
    HasTypeAndArg(Span),
    ReferencingLocally(Span),
    /// A call with neither a type argument nor an argument.
    NotDefined(Span),
    /// The plain `<script>`'s `export default {}` declares the option too.
    DefinedInBoth(Span),
}

/// Check the `macro_name` macros of `program`; `option` is the Options API
/// counterpart (`emits` / `props`) looked up in the sibling block.
///
/// eslint-plugin-vue's `<script setup>` visitor treats a top-level identifier
/// call as the macro even when that name is locally bound, and it settles an
/// empty call or a duplicated option from the inline text of a `src` block.
/// A standalone module has no compiler macro, so a same-named binding there
/// is the module's own function.
pub(super) fn check(
    program: &Program<'_>,
    sfc: SfcScriptContext<'_>,
    macro_name: &str,
    option: &str,
) -> Vec<Finding> {
    if !sfc.is_script_setup && super::props_emits::compiler_macro_is_shadowed(program, macro_name) {
        return Vec::new();
    }
    let calls: Vec<&CallExpression<'_>> = setup_macros(program)
        .into_iter()
        .filter(|found| found.name == macro_name)
        .map(|found| found.call)
        .collect();
    let mut findings = Vec::new();
    let Some(first) = calls.first() else {
        return findings;
    };
    let mut semantic = None;
    let mut empty = None;
    for call in &calls {
        let has_type = call
            .type_arguments
            .as_ref()
            .is_some_and(|arguments| !arguments.params.is_empty());
        let Some(argument) = call.arguments.first() else {
            if !has_type {
                empty = Some(call.span);
            }
            continue;
        };
        if has_type {
            findings.push(Finding::HasTypeAndArg(call.span));
            continue;
        }
        let semantic = semantic.get_or_insert_with(|| {
            SemanticBuilder::new()
                .with_build_nodes(true)
                .build(program)
                .semantic
        });
        local_references(semantic, argument, &mut findings);
    }
    if calls.len() > 1 {
        findings.extend(calls.iter().map(|call| Finding::Multiple(call.span)));
        return findings;
    }
    let declared_in_options = sfc
        .sibling_program
        .is_some_and(|sibling| exports_option(sibling, option));
    match (empty, declared_in_options) {
        (Some(span), false) => findings.push(Finding::NotDefined(span)),
        (None, true) => findings.push(Finding::DefinedInBoth(first.span)),
        _ => {}
    }
    findings
}

fn local_references(semantic: &Semantic<'_>, argument: &Argument<'_>, findings: &mut Vec<Finding>) {
    let mut finder = LocalReferences {
        semantic,
        range: argument.span(),
        found: Vec::new(),
    };
    finder.visit_argument(argument);
    findings.extend(finder.found.into_iter().map(Finding::ReferencingLocally));
}

struct LocalReferences<'s, 'a> {
    semantic: &'s Semantic<'a>,
    range: Span,
    found: Vec<Span>,
}

impl<'a> Visit<'a> for LocalReferences<'_, 'a> {
    fn visit_ts_type(&mut self, _ty: &TSType<'a>) {}

    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        let scoping = self.semantic.scoping();
        let Some(symbol) = it
            .reference_id
            .get()
            .and_then(|reference| scoping.get_reference(reference).symbol_id())
        else {
            return;
        };
        let declared = scoping.symbol_span(symbol);
        let inside = self.range.start <= declared.start && declared.end <= self.range.end;
        if !inside && !scoping.symbol_flags(symbol).is_import() {
            self.found.push(it.span);
        }
    }
}

/// Upstream's `findProperty(exportDefault, option)` on the plain `<script>`.
fn exports_option(program: &Program<'_>, option: &str) -> bool {
    program.body.iter().any(|statement| {
        let Statement::ExportDefaultDeclaration(export) = statement else {
            return false;
        };
        let object = match &export.declaration {
            ExportDefaultDeclarationKind::ObjectExpression(object) => object,
            declaration => match declaration
                .as_expression()
                .map(super::setup_macros::skip_ts_as)
            {
                Some(Expression::ObjectExpression(object)) => object,
                _ => return false,
            },
        };
        object.properties.iter().any(|property| {
            matches!(property, ObjectPropertyKind::ObjectProperty(property)
                if property.key.static_name().as_deref() == Some(option))
        })
    })
}

#[cfg(test)]
mod tests {
    use crate::linter::Linter;

    fn reported(rule: &str, sfc: &str) -> Vec<String> {
        let mut result = Linter::new()
            .with_enabled_rules(Some(vec![rule.into()]))
            .lint_sfc(sfc, "Probe.vue");
        result
            .diagnostics
            .sort_by_key(|diagnostic| diagnostic.start);
        result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                let text = sfc
                    .get(diagnostic.start as usize..diagnostic.end as usize)
                    .unwrap_or_default();
                format!("{}: {text}", diagnostic.message)
            })
            .collect()
    }

    #[test]
    fn local_references_resolve_through_scope() {
        let emits = "<script setup lang=\"ts\">\nimport { imported } from './x'\nenum Names { Save = 'save' }\nnamespace NS { export const a = 'a' }\nconst draft = { id: 1 }\nlet mutable = 'm'\nconst emit = defineEmits({ a: (next: typeof draft) => imported(next), b: () => Names.Save, c: () => NS.a, d: (p = mutable) => p, e: function inner() { return inner } })\n</script>";
        assert_eq!(
            reported("script/valid-define-emits", emits),
            [
                "defineEmits() cannot reference locally-declared variables.: Names",
                "defineEmits() cannot reference locally-declared variables.: NS",
                "defineEmits() cannot reference locally-declared variables.: mutable",
            ]
        );
        let props = "<script setup lang=\"ts\">\nconst value = 1\nconst limit = 3\nconst { size } = defineProps({ size: { type: Number, validator: (value: number) => value > limit }, list: Array as PropType<Array<typeof limit>> })\n</script>";
        assert_eq!(
            reported("script/valid-define-props", props),
            ["defineProps() is referencing locally declared variables.: limit"]
        );
    }

    #[test]
    fn empty_and_duplicated_declarations() {
        let empty = "<script setup lang=\"ts\">\nconst emit = defineEmits()\nconst props = defineProps()\n</script>";
        assert_eq!(
            reported("script/valid-define-emits", empty),
            ["defineEmits() does not define any custom events.: defineEmits()"]
        );
        assert_eq!(
            reported("script/valid-define-props", empty),
            ["defineProps() does not define any props.: defineProps()"]
        );
        let options = "<script>\nexport default { emits: ['a'], props: ['b'] }\n</script>\n<script setup>\ndefineEmits()\ndefineProps(['b'])\n</script>";
        assert!(reported("script/valid-define-emits", options).is_empty());
        assert_eq!(
            reported("script/valid-define-props", options),
            ["Props are defined in both defineProps() and export default {}.: defineProps(['b'])"]
        );
        let twice = "<script setup>\ndefineEmits(['a'])\nfunction f() { return defineEmits(['c']) }\ndefineEmits(['b'])\n</script>";
        assert_eq!(reported("script/valid-define-emits", twice).len(), 2);
        let plain = "<script>\nconst x = 1\ndefineEmits([x])\ndefineEmits()\n</script>";
        assert!(reported("script/valid-define-emits", plain).is_empty());
        // A local binding in `<script setup>` is still the macro upstream.
        let shadowed = "<script setup>\nfunction defineProps() {}\ndefineProps()\n</script>";
        assert_eq!(
            reported("script/valid-define-props", shadowed),
            ["defineProps() does not define any props.: defineProps()"]
        );
        let shadowed_twice = "<script setup>\nfunction defineEmits() {}\ndefineEmits()\ndefineEmits(['a'])\n</script>";
        assert_eq!(
            reported("script/valid-define-emits", shadowed_twice).len(),
            2
        );
        // A nested binding does not shadow the top-level macro.
        let nested = "<script setup>\ndefineEmits(['a'])\ndefineEmits(['b'])\nfunction inner() { var defineEmits = () => {} }\n</script>";
        assert_eq!(reported("script/valid-define-emits", nested).len(), 2);
        // `src` does not hide an empty call or a duplicated option.
        let external = "<script>\nexport default { emits: [] }\n</script>\n<script setup src=\"./x.ts\">\ndefineEmits(['a'])\n</script>";
        assert_eq!(
            reported("script/valid-define-emits", external),
            [
                "Custom events are defined in both defineEmits() and export default {}.: defineEmits(['a'])"
            ]
        );
        let external_empty =
            "<script src=\"./opt.ts\"></script>\n<script setup>\ndefineEmits()\n</script>";
        assert_eq!(
            reported("script/valid-define-emits", external_empty),
            ["defineEmits() does not define any custom events.: defineEmits()"]
        );
        let module_source =
            "function defineProps() {}\ndefineProps()\ndefineProps({ a: String })\n";
        assert!(reported_script("script/valid-define-props", module_source).is_empty());
        let module_macro = "defineProps()\ndefineProps({ a: String })\n";
        assert_eq!(
            reported_script("script/valid-define-props", module_macro).len(),
            2
        );
    }

    fn reported_script(rule: &str, source: &str) -> Vec<String> {
        let mut result = Linter::new()
            .with_enabled_rules(Some(vec![rule.into()]))
            .lint_script(source, "probe.ts");
        result
            .diagnostics
            .sort_by_key(|diagnostic| diagnostic.start);
        result
            .diagnostics
            .iter()
            .map(|diagnostic| {
                let text = source
                    .get(diagnostic.start as usize..diagnostic.end as usize)
                    .unwrap_or_default();
                format!("{}: {text}", diagnostic.message)
            })
            .collect()
    }
}
