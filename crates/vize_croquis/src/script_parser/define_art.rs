//! A `defineArt()`-only reader for Musea (Davinci P4-13).
//!
//! [`parse_script_setup`](super::parse_script_setup) runs the whole
//! script-setup analysis — bindings, the scope chain with its global scopes,
//! reactivity, props and emits — to answer every consumer at once. Musea
//! needs one macro out of it, and the analysis costs several times the parse
//! itself, so this reader parses once and visits only the statement shapes
//! at which the full analysis registers a `defineArt()` call, in the same
//! order, with the same import-source bookkeeping and the same argument
//! extraction ([`extract_define_art`]). `define_art_tests` pins
//! `parse_define_art(s) == parse_script_setup(s).macros.define_art()` over a
//! battery of both registered and ignored shapes.

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    Argument, BindingPattern, CallExpression, Expression, ImportDeclaration,
    ImportDeclarationSpecifier, Statement,
};
use oxc_span::SourceType;

use super::extract::{extract_call_expression, extract_define_art};
use super::recovery::parse_program_for_analysis;
use crate::macros::{ArtDefinition, DEFINE_ART, MacroKind};
use vize_carton::{CompactString, FxHashMap};

/// The `defineArt()` metadata of script-setup `source`, exactly as
/// `parse_script_setup(source).macros.define_art()` reports it.
#[must_use]
pub fn parse_define_art(source: &str) -> Option<ArtDefinition> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let ret = parse_program_for_analysis(&allocator, source, source_type);
    if ret.panicked {
        return None;
    }
    let mut reader = Reader::default();
    for statement in ret.program.body.iter() {
        match statement {
            Statement::ImportDeclaration(import) => reader.record_imports(import),
            // The full analysis registers a macro only when the statement's
            // expression *is* the call (no unwrapping here).
            Statement::ExpressionStatement(statement) => {
                if let Expression::CallExpression(call) = &statement.expression {
                    reader.visit_call(call);
                }
            }
            // Identifier and array-pattern declarators register their
            // (unwrapped) initializer call; object patterns register only a
            // `defineProps` destructure, which can never hold `defineArt`.
            Statement::VariableDeclaration(declaration) => {
                for declarator in declaration.declarations.iter() {
                    let registers = matches!(
                        declarator.id,
                        BindingPattern::BindingIdentifier(_) | BindingPattern::ArrayPattern(_)
                    );
                    if let Some(call) = declarator
                        .init
                        .as_ref()
                        .filter(|_| registers)
                        .and_then(extract_call_expression)
                    {
                        reader.visit_call(call);
                    }
                }
            }
            _ => {}
        }
    }
    reader.art
}

#[derive(Default)]
struct Reader {
    import_sources: FxHashMap<CompactString, CompactString>,
    art: Option<ArtDefinition>,
}

impl Reader {
    /// Value imports map their local name to the module specifier; type-only
    /// imports and specifiers are not bindings and are skipped.
    fn record_imports(&mut self, import: &ImportDeclaration<'_>) {
        if import.import_kind.is_type() {
            return;
        }
        let source = import.source.value.as_str();
        for specifier in import.specifiers.iter().flatten() {
            let (local, is_type) = match specifier {
                ImportDeclarationSpecifier::ImportSpecifier(s) => {
                    (s.local.name.as_str(), s.import_kind.is_type())
                }
                ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                    (s.local.name.as_str(), false)
                }
                ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                    (s.local.name.as_str(), false)
                }
            };
            if !is_type {
                self.import_sources
                    .insert(CompactString::new(local), CompactString::new(source));
            }
        }
    }

    /// The macro dispatch of `process_call_expression`, reduced to the two
    /// arms that can reach `defineArt`: the macro itself (the last call
    /// wins), and `withDefaults(inner)`, which processes its inner call.
    fn visit_call(&mut self, call: &CallExpression<'_>) {
        let Expression::Identifier(callee) = &call.callee else {
            return;
        };
        let name = callee.name.as_str();
        match MacroKind::from_name(name) {
            Some(MacroKind::Custom) if name == DEFINE_ART => {
                if let Some(art) = extract_define_art(&self.import_sources, call) {
                    self.art = Some(art);
                }
            }
            Some(MacroKind::WithDefaults) => {
                if let Some(Argument::CallExpression(inner)) = call.arguments.first() {
                    self.visit_call(inner);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    //! `parse_define_art` agrees with the full script-setup analysis on every
    //! statement shape — the ones that register `defineArt()` and the ones that
    //! do not.

    use super::parse_define_art;
    use crate::script_parser::parse_script_setup;

    const BATTERY: &[&str] = &[
        // No macro at all, and a file that fails to parse.
        "const a = 1;",
        "const = ;",
        "",
        // Identifier component resolved through a default import.
        "import Button from './Button.vue';\ndefineArt(Button, { title: 'Button' });",
        // Named, namespace and aliased imports.
        "import { Card as Panel } from '@/ui';\ndefineArt(Panel, { category: 'Layout' });",
        "import * as UI from '@/ui';\ndefineArt(UI, {});",
        // Type-only imports are not bindings: the component stays unresolved.
        "import type Button from './Button.vue';\ndefineArt(Button);",
        "import { type Button } from './Button.vue';\ndefineArt(Button);",
        // A later value import overrides an earlier one of the same name.
        "import A from './a.vue';\nimport A from './b.vue';\ndefineArt(A);",
        // An import after the call is not yet recorded at the call.
        "defineArt(Late);\nimport Late from './Late.vue';",
        // String-literal component with every option.
        "defineArt('./base-button.vue', { title: 'Base', description: 'd', category: 'c', \
     tags: ['a', 'b'], status: 'draft', order: 3 });",
        // Registered declarator shapes, with the unwrapping casts.
        "import B from './B.vue';\nconst art = defineArt(B, { title: 'T' });",
        "import B from './B.vue';\nconst art = defineArt(B) as any;",
        "import B from './B.vue';\nlet art = (defineArt(B)!);",
        "import B from './B.vue';\nconst [art] = defineArt(B);",
        // withDefaults processes its inner call.
        "import B from './B.vue';\nwithDefaults(defineArt(B, { title: 'W' }), {});",
        "import B from './B.vue';\nconst w = withDefaults(defineArt(B), {});",
        // The last registered call wins.
        "defineArt('./a.vue', { title: 'first' });\ndefineArt('./b.vue', { title: 'second' });",
        // Shapes the full analysis never registers.
        "import B from './B.vue';\nconst { a } = defineArt(B);",
        "import B from './B.vue';\n(defineArt(B));",
        "import B from './B.vue';\nvoid defineArt(B);",
        "import B from './B.vue';\nif (true) { defineArt(B); }",
        "import B from './B.vue';\nfunction f() { defineArt(B); }",
        "import B from './B.vue';\nexport const art = defineArt(B);",
        "import B from './B.vue';\nfoo.defineArt(B);",
        "defineArt();",
        "defineArt(42);",
        // TypeScript syntax the analysis parses.
        "import B from './B.vue';\ninterface P { a: string }\ndefineArt<P>(B, { order: 1 });",
    ];

    #[test]
    fn agrees_with_the_full_analysis_on_every_shape() {
        for source in BATTERY {
            let full = parse_script_setup(source);
            assert_eq!(
                parse_define_art(source).as_ref(),
                full.macros.define_art(),
                "{source}"
            );
        }
    }

    #[test]
    fn registered_shapes_read_exactly() {
        let art = parse_define_art(
        "import { Card as Panel } from '@/ui';\nconst art = defineArt(Panel, { title: 'P', tags: ['x'] }) as any;",
    )
    .expect("registered");
        assert_eq!(
            (
                art.component_name.as_str(),
                art.component_source.as_deref(),
                art.title.as_deref(),
                art.tags.iter().map(|tag| tag.as_str()).collect::<Vec<_>>()
            ),
            ("Panel", Some("@/ui"), Some("P"), vec!["x"])
        );
        assert_eq!(parse_define_art("const { a } = defineArt(B);"), None);
    }
}
