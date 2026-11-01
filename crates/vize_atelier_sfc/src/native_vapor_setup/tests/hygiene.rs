//! Generated helper/parameter names cannot capture the actual mutable target.

use super::*;
use oxc_ast::ast::{BindingPattern, FormalParameter};
use oxc_ast_visit::Visit;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;

#[test]
fn actual_authored_helper_and_parameter_names_keep_distinct_resolved_symbols() {
    for name in [
        "_applyTextModel",
        "_isRef",
        "$event",
        "_value",
        "__vizeModelRuntime",
        "__vizeModelValue",
    ] {
        let source = cstr!(
            "<script setup>let {name}='x';const __vizeModelRuntime1=2;const __vizeModelValue1=3</script><template><input v-model='{name}'/></template>"
        );
        let arena = Allocator::default();
        let compilation =
            compile_native_vapor_setup_sfc(&arena, source.as_str(), Default::default());
        let output = compilation
            .result()
            .expect("genuine normally completed Model");
        eprintln!("Vapor Model hygiene {name}: {}", output.code());
        let parsed = Parser::new(&arena, output.code(), SourceType::mjs()).parse();
        assert!(
            !parsed.panicked && parsed.diagnostics.is_empty(),
            "{name}: {:?}",
            parsed.diagnostics
        );
        let built = SemanticBuilder::new()
            .with_check_syntax_error(true)
            .build(&parsed.program);
        assert!(
            built.diagnostics.is_empty(),
            "{name}: {:?}",
            built.diagnostics
        );
        let scoping = built.semantic.scoping();
        let symbols: Vec<_> = scoping
            .symbol_ids()
            .filter(|symbol| {
                scoping.symbol_name(*symbol) == name && !scoping.symbol_flags(*symbol).is_import()
            })
            .collect();
        let [symbol] = symbols.as_slice() else {
            panic!("one original target, never generated shadow")
        };
        let references: Vec<_> = scoping.get_resolved_references(*symbol).collect();
        assert_eq!(
            references.len(),
            4,
            "original getter/isRef/member-set/bare-set target"
        );
        assert_eq!(
            references
                .iter()
                .filter(|reference| reference.is_write())
                .count(),
            1,
            "exact one direct lexical assignment, independently resolved"
        );
        for imported in scoping.symbol_ids().filter(|candidate| {
            scoping.symbol_name(*candidate) == name && scoping.symbol_flags(*candidate).is_import()
        }) {
            assert_ne!(
                scoping.symbol_scope_id(imported),
                scoping.symbol_scope_id(*symbol)
            );
            assert_eq!(
                scoping.get_resolved_references(imported).count(),
                1,
                "the original module helper alias is captured once outside the setup scope"
            );
        }
        let expected_helper = if name == "__vizeModelRuntime" {
            "__vizeModelRuntime2"
        } else {
            "__vizeModelRuntime"
        };
        let module_bindings: Vec<_> = scoping
            .symbol_ids()
            .filter(|symbol| {
                !scoping.symbol_flags(*symbol).is_import()
                    && scoping.symbol_scope_id(*symbol) == scoping.root_scope_id()
            })
            .map(|symbol| scoping.symbol_name(symbol))
            .collect();
        assert_eq!(
            module_bindings,
            [expected_helper, "t0", "_sfc_main"],
            "the complete module declaration vector chooses the first free helper name"
        );
        let mut parameters = Parameters::default();
        parameters.visit_program(&parsed.program);
        let expected_parameter = if name == "__vizeModelValue" {
            "__vizeModelValue2"
        } else {
            "__vizeModelValue"
        };
        assert_eq!(
            parameters.0,
            ["__props", expected_parameter],
            "the complete original setup and generated setter parameter vector"
        );
    }
}

#[derive(Default)]
struct Parameters(Vec<String>);

impl<'arena> Visit<'arena> for Parameters {
    fn visit_formal_parameter(&mut self, parameter: &FormalParameter<'arena>) {
        let BindingPattern::BindingIdentifier(identifier) = &parameter.pattern else {
            panic!("this complete inline setup family has only original bare parameters")
        };
        self.0.push(identifier.name.as_str().into());
    }
}

#[test]
fn existing_generated_binding_refusals_are_not_widened_by_model_hygiene() {
    for name in [
        "_sfc_main",
        "__props",
        "_template",
        "_defineVaporComponent",
        "_unref",
        "n0",
        "t0",
    ] {
        let source = cstr!(
            "<script setup>let {name}='x'</script><template><input v-model='{name}'/></template>"
        );
        let arena = Allocator::default();
        let compilation =
            compile_native_vapor_setup_sfc(&arena, source.as_str(), Default::default());
        if name == "__props" {
            // The existing selected-setup policy rejects this actual authored
            // reserved declaration before a target view or output can exist.
            assert!(compilation.observation().admitted().is_none());
            assert!(!compilation.observation().original().issues().is_empty());
            assert!(matches!(
                compilation.result(),
                Err(NativeVaporSetupSfcCompileError::Source(_))
            ));
            continue;
        }
        assert!(compilation.observation().admitted().is_some(), "{name}");
        let Err(NativeVaporSetupSfcCompileError::Vapor(error)) = compilation.result() else {
            panic!("old generated-binding refusal remains a whole-output refusal")
        };
        assert_eq!(
            error.kind,
            vize_l4::targets::vapor::VaporErrorKind::GeneratedBindingCollision
        );
    }
}
