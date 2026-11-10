//! Inspect complete emitted declarations in both helper modes.
use oxc_ast::ast::{BindingPattern, TSTypeAliasDeclaration, VariableDeclaration};
use oxc_ast_visit::{
    Visit,
    walk::{walk_ts_type_alias_declaration, walk_variable_declaration},
};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};
use vize_carton::{Allocator, cstr};
use vize_croquis::{Analyzer, AnalyzerOptions};

const TEMPLATE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/standalone-constructor-correlation-11023/Template.html.txt"
));

#[derive(Default)]
struct Declarations {
    pairs: Vec<Span>,
    props: Vec<Span>,
}

impl<'a> Visit<'a> for Declarations {
    fn visit_variable_declaration(&mut self, declaration: &VariableDeclaration<'a>) {
        if declaration.declarations.iter().any(|declarator| {
            matches!(&declarator.id, BindingPattern::BindingIdentifier(name)
                if name.name == "__vize_slot_host_1_instances")
        }) {
            self.pairs.push(declaration.span);
        }
        walk_variable_declaration(self, declaration);
    }

    fn visit_ts_type_alias_declaration(&mut self, declaration: &TSTypeAliasDeclaration<'a>) {
        if declaration.id.name == "__VProps0" {
            self.props.push(declaration.span);
        }
        walk_ts_type_alias_declaration(self, declaration);
    }
}

#[test]
fn standalone_and_shared_props_use_the_actual_constructor_pair() {
    for hoisted in [false, true] {
        let allocator = Allocator::new();
        let (root, _) = vize_armature::parse(&allocator, TEMPLATE);
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();
        let code = crate::virtual_ts::generate_virtual_ts_with_offsets_and_checks(
            &summary,
            None,
            Some(&root),
            0,
            0,
            &crate::virtual_ts::VirtualTsOptions::default(),
            crate::virtual_ts::VirtualTsGenerationOptions {
                hoist_shared_preamble: hoisted,
                ..Default::default()
            },
        )
        .code;
        let parsed = Parser::new(&allocator, &code, SourceType::tsx()).parse();
        assert_eq!(parsed.panicked, false);
        assert_eq!(parsed.diagnostics.len(), 0);
        let mut declarations = Declarations::default();
        declarations.visit_program(&parsed.program);
        let prefix = if hoisted {
            "import(\"vize:virtual-ts\")."
        } else {
            ""
        };
        let pair = cstr!(
            "const __vize_slot_host_1_instances = (undefined as unknown as <__Z, __P>(withoutArguments: __Z, withArguments: __P) => {{ readonly withoutArguments: __Z; readonly withArguments: __P }})(new (undefined as unknown as {prefix}$SlotConstructorWithoutArguments<{prefix}$SlotConstructor<typeof ElBadge>>)(), new (undefined as unknown as {prefix}$SlotConstructorWithArguments<{prefix}$SlotConstructor<typeof ElBadge>>)({{\n    \"value\": 1,\n  }}, ...([] as unknown as {prefix}$SlotConstructorArgs<{prefix}$SlotConstructor<typeof ElBadge>>)));"
        );
        assert_eq!(
            declarations
                .pairs
                .iter()
                .map(|span| &code[span.start as usize..span.end as usize])
                .collect::<Vec<_>>(),
            vec![pair.as_str()],
            "the emitted pair must retain both original constructor calls and the authored argument",
        );
        let selected = "(typeof __vize_slot_host_1_instances.withArguments extends { $props: unknown } ? typeof __vize_slot_host_1_instances.withArguments : typeof __vize_slot_host_1_instances.withoutArguments)";
        let raw = cstr!(
            "typeof ElBadge extends {{ readonly __vizeComponentMarker: true }} ? {prefix}$ComponentRawProps<typeof ElBadge> : 0 extends (1 & {selected}) ? {prefix}$ComponentRawProps<typeof ElBadge> : {selected} extends {{ $props: infer __P }} ? unknown extends __P ? Record<string, unknown> : __P : {prefix}$ComponentRawProps<typeof ElBadge>"
        );
        let is_any = if hoisted {
            "import(\"vize:virtual-ts\").IsAny"
        } else {
            "$IsAny"
        };
        let props = cstr!(
            "type __VProps0 = typeof ElBadge extends {{ __vizeCheck: infer __F }} ? ({is_any}<__F> extends true ? {raw} : Record<string, unknown>) : {raw};"
        );
        assert_eq!(
            declarations
                .props
                .iter()
                .map(|span| &code[span.start as usize..span.end as usize])
                .collect::<Vec<_>>(),
            vec![props.as_str()],
            "public props must come from the same captured instance in both emission modes",
        );
    }
}
