//! Repeated authored expression text must retain each lexical scope and span.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "tests compare owned emitted modules and assert by panicking"
)]

mod support;

use support::{assert_transformed_sound, with_transformed};
use vize_s0::{Allocator, SourceRoot, Span, String};
use vize_s1_to_s2::{DomEmitOptions, emit_dom_with_options};

#[test]
fn repeated_text_preserves_loop_slot_and_root_resolution() {
    for source in [
        r#"<div :id="item.value"><p v-for="item in items" :title="item.value"></p><p :title="item.value"></p></div>"#,
        r#"<Foo :id="item.value"><template #default="{ item }"><p :title="item.value"></p></template></Foo><p :title="item.value"></p>"#,
        r#"<button :id="value" @click="read(value)" :title="value /* trailing */"></button>"#,
    ] {
        assert_transformed_sound(source, "repeated authored expressions");
        for (is_ts, prefix_identifiers) in
            [(false, false), (false, true), (true, false), (true, true)]
        {
            with_transformed(source, |lowered, _folio, facts, _budget| {
                let emitted = emit_dom_with_options(
                    lowered,
                    facts,
                    &DomEmitOptions {
                        prefix_identifiers,
                        is_ts,
                        ..DomEmitOptions::DEFAULT
                    },
                )
                .unwrap_or_else(|error| panic!("emission refused {source}: {error:?}"));
                let allocator = Allocator::new();
                let (_, errors, legacy) = vize_atelier_dom::compile_template_legacy_with_options(
                    &allocator,
                    source,
                    vize_atelier_dom::DomCompilerOptions {
                        prefix_identifiers,
                        is_ts,
                        ..vize_atelier_dom::DomCompilerOptions::default()
                    },
                );
                assert!(
                    errors.iter().all(|error| error.is_compatibility_notice()),
                    "legacy diagnostics: {errors:?}"
                );
                let mut expected = String::from(legacy.preamble.as_str());
                expected.push('\n');
                expected.push_str(legacy.code.as_str());
                assert_eq!(emitted.assembled().as_str(), expected, "{source}");
            });
        }
    }
}

#[test]
fn repeated_conditional_contributions_keep_file_absolute_positions() {
    let source =
        "é prefix\n<template><p :id=\"ok ? a : b\"></p><p :title=\"ok ? a : b\"></p></template>";
    let start = source.find("<p").expect("template body");
    let end = source.find("</template>").expect("template end");
    let template = source.get(start..end).expect("authored template");
    let root = SourceRoot::new(source).expect("small source");
    let block = root.block(template, start as u32).expect("authored block");
    let allocator = Allocator::new();
    let (tree, errors) = vize_s1::parse(&allocator, template);
    let lowered = vize_s1_to_s2::lower_source_block(&allocator, &tree, &errors, block);
    let facts = vize_s1_to_s2::pass::cfg::run(&lowered);
    assert_eq!(facts.contributions.len(), 2);
    let expression = "ok ? a : b";
    for (contribution, (offset, _)) in facts
        .contributions
        .iter()
        .zip(source.match_indices(expression))
    {
        assert_eq!(
            contribution.kind,
            vize_s1_to_s2::pass::cfg::DecisionKind::Conditional
        );
        assert_eq!(
            contribution.span,
            Span::new(offset as u32, (offset + expression.len()) as u32)
        );
    }
}
