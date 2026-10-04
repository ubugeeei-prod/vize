//! Frozen whole template-block output, separate from enclosing SFC completion.

use vize_glyph::native_doc::{LineEnding, observed_native_template_document};
use vize_l0::Allocator;
use vize_l1::embed::Lang;

use super::{SCRIPTS, WIDTHS, assert_fixed, format, newline, options, selected};

#[test]
fn empty_and_zero_operand_blocks_keep_complete_original_text_and_comments() {
    for (body, expected) in [
        ("", ""),
        (
            " text&amp;y<!--keep--><b></b>",
            " text&amp;y<!--keep--><b></b>",
        ),
    ] {
        for script in SCRIPTS {
            let source = vize_l0::cstr!(
                "<!--outside--><template>{body}</template><style>p{{color:red}}</style>{script}"
            );
            let arena = Allocator::default();
            let owner = selected(&arena, &source);
            let document = observed_native_template_document(&owner, &arena).unwrap();
            assert!(document.operands().is_empty());
            assert!(core::ptr::eq(document.original(), &owner));
            for width in WIDTHS {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    assert_eq!(format(&source, options(width, ending)), expected);
                }
            }
            assert_fixed(&source, script);
        }
    }
}

#[test]
fn root_nested_and_multiple_interpolations_pin_complete_js_ts_width_and_ending_outputs() {
    for (body, flat, narrow, count) in [
        ("{{1n}}", "{{ 1n }}", "{{\n  1n\n}}", 1),
        (
            "<p>{{1n+2n}}</p>",
            "<p>{{ 1n + 2n }}</p>",
            "<p>{{\n    1n +\n      2n\n  }}</p>",
            1,
        ),
        (
            "<!--keep-->{{a}}<p>{{b}}</p><section><b>{{c}}</b></section>",
            "<!--keep-->{{ a }}<p>{{ b }}</p><section><b>{{ c }}</b></section>",
            "<!--keep-->{{\n  a\n}}<p>{{\n    b\n  }}</p><section><b>{{\n      c\n    }}</b></section>",
            3,
        ),
    ] {
        for script in SCRIPTS {
            let source = vize_l0::cstr!("<!--outside--><template>{body}</template>{script}");
            let arena = Allocator::default();
            let owner = selected(&arena, &source);
            let carrier = observed_native_template_document(&owner, &arena).unwrap();
            assert_eq!(carrier.operands().len(), count);
            for operand in carrier.operands() {
                assert_eq!(
                    operand.syntax().grammar().lang,
                    if script.is_empty() {
                        Lang::Js
                    } else {
                        Lang::Ts
                    }
                );
                assert!(operand.syntax().source_type().is_module());
                assert_eq!(
                    operand.syntax().source_type().is_typescript(),
                    !script.is_empty()
                );
                assert_eq!(operand.content_span().slice(&source), operand.raw_content());
            }
            for width in WIDTHS {
                for ending in [LineEnding::Lf, LineEnding::CrLf] {
                    let expected = if width < 80 {
                        narrow.replace('\n', newline(ending))
                    } else {
                        flat.to_owned()
                    };
                    assert_eq!(
                        format(&source, options(width, ending)),
                        expected,
                        "{source} / {width}"
                    );
                }
            }
            assert_fixed(&source, script);
        }
    }
}

#[test]
fn generated_frame_endings_keep_original_entity_literal_and_physical_lf_gaps() {
    for script in SCRIPTS {
        let source = vize_l0::cstr!("<template>{{{{1n,&#39;//x&#39;\n,2n}}}}</template>{script}");
        for width in WIDTHS {
            for ending in [LineEnding::Lf, LineEnding::CrLf] {
                let generated = newline(ending);
                assert_eq!(
                    format(&source, options(width, ending)),
                    vize_l0::cstr!("{{{{{generated}  1n, &#39;//x&#39;\n, 2n{generated}}}}}")
                );
            }
        }
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let document = observed_native_template_document(&owner, &arena).unwrap();
        let syntax = document.operands()[0].syntax();
        assert!(syntax.source().decode_map().is_some());
        assert_eq!(syntax.comments().count(), 0);
        assert_eq!(syntax.source().text(), "1n,'//x'\n,2n");
        assert_fixed(&source, script);
    }
}

#[test]
fn native_v_pre_suppresses_broken_and_nested_raw_interpolations_without_observations() {
    let body = "<p v-pre>{{broken( }} &amp; <b>{{raw}}</b></p>{{1n}}";
    for script in SCRIPTS {
        let source = vize_l0::cstr!("<template>{body}</template>{script}");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let document = observed_native_template_document(&owner, &arena).unwrap();
        assert_eq!(document.operands().len(), 1);
        assert_eq!(document.operands()[0].raw_content(), "1n");
        for width in WIDTHS {
            for ending in [LineEnding::Lf, LineEnding::CrLf] {
                let generated = newline(ending);
                let expected = if width < 80 {
                    vize_l0::cstr!(
                        "<p{generated}  v-pre{generated}>{{{{broken( }}}} &amp; <b>{{{{raw}}}}</b></p>{{{{{generated}  1n{generated}}}}}"
                    )
                } else {
                    vize_l0::String::from("<p v-pre>{{broken( }} &amp; <b>{{raw}}</b></p>{{ 1n }}")
                };
                assert_eq!(format(&source, options(width, ending)), expected);
            }
        }
        assert_fixed(&source, script);
    }
}
