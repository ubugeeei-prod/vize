#![cfg(all(feature = "original-scss-products", not(target_arch = "wasm32")))]
#![expect(
    clippy::unwrap_used,
    reason = "independent style oracles and fixed source fixtures"
)]
use davinci_harness::alloc::{CountingAllocator, mark_installed};
use std::alloc::System;
use vize_atelier_sfc::scss::ScssProducts;
use vize_l0::{
    Allocator, SourceBlock, SourceRoot,
    config::{VueDialect, VueVersion},
};
use vize_l1::container::vue::{DescriptorOptions, Vue};
use vize_l2::lang::scss::ScssRefusal;

#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::system();

mod observed_scss;

fn whole(source: &str) -> SourceBlock<'_> {
    SourceRoot::new(source).unwrap().whole_block()
}

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: Default::default(),
    }
}

#[test]
fn source_definitions_select_the_visible_assignment_across_nested_rules_and_aliases() {
    let source = "<template>😀</template>\r\n<style scoped module='s' lang='scss'>\r\n$main_color: #abc; $alias: $main-color; .card { color: $alias; &.active { margin: 2px; } color: $main-color; }\n$main-color: red; .final { color: $main_color; }</style>";
    let arena = Allocator::default();
    let descriptor = Vue.observe_descriptor(&arena, source, options());
    let style = descriptor.admitted().unwrap().styles().next().unwrap();
    let products = ScssProducts::from_style(style).unwrap();
    let program = products.program().unwrap();
    assert_eq!(
        products.css().unwrap(),
        ".card {\n  color: #abc;\n}\n.card.active {\n  margin: 2px;\n}\n.card {\n  color: #abc;\n}\n\n.final {\n  color: red;\n}"
    );
    assert_eq!(program.variables().len(), 3);
    let expected = ["$main_color", "$alias", "$main_color", "$main-color"];
    assert_eq!(program.variable_uses().len(), expected.len());
    for (use_, name) in program.variable_uses().iter().zip(expected) {
        let definition = program.definition_at(use_.span.start).unwrap();
        assert_eq!(definition.span.slice(source), name);
        assert_eq!(
            program.definition_at(use_.span.end - 1).unwrap().span,
            definition.span
        );
        assert!(program.definition_at(use_.span.end).is_none());
    }
    assert_eq!(
        program.variables().last().unwrap().span.start,
        source.rfind("$main-color:").unwrap() as u32
    );
    assert!(products.classes().unwrap().is_complete());
    assert_eq!(
        products
            .classes()
            .unwrap()
            .names()
            .iter()
            .map(|name| name.as_str())
            .collect::<Vec<_>>(),
        ["active", "card", "final"]
    );
    let css = "<template><p/></template><style lang='less'>.c{color:red}</style>";
    let seen = Vue.observe_descriptor(&arena, css, options());
    let style = seen.admitted().unwrap().styles().next().unwrap();
    assert!(matches!(
        ScssProducts::from_style(style),
        Err(ScssRefusal::Language)
    ));
}

#[test]
fn the_entire_program_refuses_unsupported_or_unresolved_statements() {
    for suffix in [
        ".b { width: $missing; }",
        "$alias: $later; $later: red;",
        ".b {x:01px;}",
        ".b {x: 2 * 4;}",
        ".b { $local: 3; }",
        "@use 'sheet';",
        "/*kept*/",
        ".c {x:var(--theme);}",
        ".c { --x: $a; }",
        ".c {x: null;}",
        ".a, .b {color: red}",
        ".a{#{$name}: red}",
        "&.missing {x:y}",
        ".b{color: y",
        ".b { content: '😀'; }",
    ] {
        let source = format!(".good{{color: red;}} {suffix}");
        let products = ScssProducts::read(whole(&source));
        assert!(products.css().is_err(), "{suffix}");
    }
}
