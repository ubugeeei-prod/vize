//! Generated helper/parameter names cannot capture the actual mutable target.

use super::*;
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
            .filter(|symbol| scoping.symbol_name(*symbol) == name)
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
        if name == "__vizeModelRuntime" {
            assert!(
                output
                    .code()
                    .contains("const __vizeModelRuntime2 = { apply: _applyTextModel")
            );
        }
        if name == "__vizeModelValue" {
            assert!(output.code().contains("__vizeModelValue2 => ("));
        }
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
        assert!(compilation.observation().admitted().is_some());
        let Err(NativeVaporSetupSfcCompileError::Vapor(error)) = compilation.result() else {
            panic!("old generated-binding refusal remains a whole-output refusal")
        };
        assert_eq!(
            error.kind,
            vize_l4::targets::vapor::VaporErrorKind::GeneratedBindingCollision
        );
    }
}
