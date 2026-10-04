//! Independently authored complete outputs, including outer continuation layout.

use super::{assert_fixed, assert_original, newline, options};
use vize_glyph::native_doc::{LineEnding, observe_native_sfc_in};
use vize_l0::Allocator;

#[test]
fn empty_and_zero_operand_whole_sources_preserve_bom_prologue_tags_tail_and_unchanged() {
    for source in [
        "<template></template>",
        "\u{feff}<!--前-->\r\n<template lang='html'>plain &amp;\n<!--inside--></template><!--尾-->\r\n",
        "<template><p>plain</p></template>",
    ] {
        for width in [0, 1, 7, 80, 200] {
            for ending in [LineEnding::Lf, LineEnding::CrLf] {
                let arena = Allocator::default();
                let options = options(width, 2, ending);
                let owner = observe_native_sfc_in(&arena, source, options);
                assert_original(&owner, source, options);
                assert!(owner.operands().is_empty());
                let result = owner.format().unwrap();
                assert_eq!(result.code, source);
                assert!(!result.changed);
                assert_fixed(source, options);
            }
        }
    }
}

#[test]
fn root_and_nested_full_output_pin_width_endings_and_all_indent_options() {
    for (source, flat, depth, binary) in [
        (
            "<template>{{1n}}</template>",
            "<template>{{ 1n }}</template>",
            0,
            false,
        ),
        (
            "<template><p>{{1n+2n}}</p></template>",
            "<template><p>{{ 1n + 2n }}</p></template>",
            1,
            true,
        ),
    ] {
        for width in [0, 1, 6, 7, 10, 11, 14, 15, 28, 29, 40, 41, 80, 200] {
            for indent in [0, 2, 4] {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    let options = options(width, indent, ending);
                    let arena = Allocator::default();
                    let owner = observe_native_sfc_in(&arena, source, options);
                    assert_original(&owner, source, options);
                    let nl = newline(ending);
                    let pad = " ".repeat((depth + 1) * indent);
                    let inner = " ".repeat((depth + 2) * indent);
                    let close = " ".repeat(depth * indent);
                    // Authored flat references measure Unicode scalars. After the
                    // outer frame breaks, the seven-scalar infix fits exactly at
                    // its original indented column, until the pending broken line.
                    assert_eq!(flat.chars().count(), if binary { 41 } else { 29 });
                    assert_eq!("1n + 2n".chars().count(), 7);
                    let expected = if width >= flat.chars().count() {
                        flat.to_owned()
                    } else if binary && width >= pad.chars().count() + 7 {
                        format!("<template><p>{{{{{nl}{pad}1n + 2n{nl}{close}}}}}</p></template>")
                    } else if binary {
                        format!(
                            "<template><p>{{{{{nl}{pad}1n +{nl}{inner}2n{nl}{close}}}}}</p></template>"
                        )
                    } else {
                        format!("<template>{{{{{nl}{pad}1n{nl}}}}}</template>")
                    };
                    let result = owner.format().unwrap();
                    assert_eq!(result.code, expected, "{source} / {options:?}");
                    assert!(result.changed);
                    let repeated = owner.format().unwrap();
                    assert_eq!(repeated.code, result.code);
                    assert_eq!(repeated.changed, result.changed);
                    assert_fixed(source, options);
                }
            }
        }
    }
}

#[test]
fn actual_whole_prefix_column_and_suffix_lookahead_choose_the_complete_output() {
    for (source, width, expected) in [
        (
            "<template>{{a}}</template>",
            28,
            "<template>{{ a }}</template>",
        ),
        (
            "<template>{{a}}</template>",
            27,
            "<template>{{\n  a\n}}</template>",
        ),
        (
            "<template>\n{{a}}\n</template>",
            7,
            "<template>\n{{ a }}\n</template>",
        ),
        (
            "<template>\n{{a}}\n</template>",
            6,
            "<template>\n{{\n  a\n}}\n</template>",
        ),
        (
            "<template>{{a}}</template><!--tail-->",
            39,
            "<template>{{ a }}</template><!--tail-->",
        ),
        (
            "<template>{{a}}</template><!--tail-->",
            38,
            "<template>{{\n  a\n}}</template><!--tail-->",
        ),
        (
            "<template>\n{{a}}</template>",
            18,
            "<template>\n{{ a }}</template>",
        ),
        (
            "<template>\n{{a}}</template>",
            17,
            "<template>\n{{\n  a\n}}</template>",
        ),
    ] {
        let arena = Allocator::default();
        let options = options(width, 2, LineEnding::Lf);
        let owner = observe_native_sfc_in(&arena, source, options);
        assert_eq!(owner.format().unwrap().code, expected, "{source} / {width}");
        assert_fixed(source, options);
    }
}

#[test]
fn generated_endings_do_not_change_outer_crlf_or_original_entity_quote_gap_bytes() {
    let source = "\u{feff}<!--前-->\r\n<template>{{1n,&#39;//x&#39;\n,2n}}</template><!--尾-->\r\n";
    for width in [0, 1, 7, 80, 200] {
        for ending in [LineEnding::Lf, LineEnding::CrLf] {
            let nl = newline(ending);
            let expected = format!(
                "\u{feff}<!--前-->\r\n<template>{{{{{nl}  1n, &#39;//x&#39;\n, 2n{nl}}}}}</template><!--尾-->\r\n"
            );
            let arena = Allocator::default();
            let options = options(width, 2, ending);
            let owner = observe_native_sfc_in(&arena, source, options);
            assert_eq!(owner.format().unwrap().code, expected);
            assert_eq!(owner.operands()[0].syntax().comments().count(), 0);
            assert_eq!(
                owner.operands()[0].syntax().source().text(),
                "1n,'//x'\n,2n"
            );
            assert_fixed(source, options);
        }
    }
}

#[test]
fn literal_physical_tails_and_encoded_comment_spelling_survive_the_complete_owner() {
    for (content, expression, tail) in [
        ("1n//x\n", "1n//x", "\n"),
        ("1n &#47;*//x*&#47;\r\n", "1n &#47;*//x*&#47;", "\r\n"),
        ("1n,&#39;//x&#39;\n \t", "1n, &#39;//x&#39;", "\n \t"),
    ] {
        let source = format!("<!--pre--><template>{{{{{content}}}}}</template><!--post-->");
        for ending in [LineEnding::Lf, LineEnding::CrLf] {
            let nl = newline(ending);
            let options = options(200, 2, ending);
            let expected = format!(
                "<!--pre--><template>{{{{{nl}  {expression}{tail}}}}}</template><!--post-->"
            );
            let arena = Allocator::default();
            let owner = observe_native_sfc_in(&arena, &source, options);
            assert_eq!(owner.format().unwrap().code, expected);
            assert_fixed(&source, options);
        }
    }
}

#[test]
fn complete_v_pre_and_multiple_nested_observations_keep_the_original_source_order() {
    let source = "<!--pre--><template><p v-pre>{{broken( }} &amp; <b>{{raw}}</b></p>{{1n}}<section>{{2n}}</section></template><!--post-->";
    let arena = Allocator::default();
    let expected = "<!--pre--><template><p v-pre>{{broken( }} &amp; <b>{{raw}}</b></p>{{ 1n }}<section>{{ 2n }}</section></template><!--post-->";
    // Width is explicit because the full suffix and opening column participate.
    let options = super::options(200, 2, LineEnding::Lf);
    let owner = observe_native_sfc_in(&arena, source, options);
    assert_eq!(owner.format().unwrap().code, expected);
    assert_eq!(
        owner
            .operands()
            .iter()
            .map(|o| o.raw_content())
            .collect::<std::vec::Vec<_>>(),
        ["1n", "2n"]
    );
    assert_fixed(source, options);
}
