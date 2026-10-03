use super::{format, operands, selected};
use vize_glyph::native_doc::{LineEnding, PrintOptions, native_template_document, print};
use vize_l0::Allocator;
use vize_l1::embed::Lang;

#[test]
fn selected_block_keeps_attributes_text_entities_and_original_interpolation_order() {
    let source = "<!--前--><template><!--keep--><p title = 'a&amp;b'> text {{a+b}}!{{ (日本&amp;&amp;ok) }}</p></template><script setup>const a=1</script>";
    assert_eq!(
        format(source, PrintOptions::default()),
        "<!--keep--><p title='a&amp;b'> text {{ a + b }}!{{ (日本 &amp;&amp; ok) }}</p>"
    );
}

#[test]
fn actual_selected_typescript_profile_formats_supported_native_families() {
    let arena = Allocator::default();
    let source = "<template>{{ (a+0xCA_FE) &amp;&amp; true }}</template><script setup lang=ts>const a=1</script>";
    let selected = selected(&arena, source);
    let original = operands(&selected);
    assert_eq!(original.first().unwrap().syntax().grammar().lang, Lang::Ts);
    let refs = original.iter().collect::<std::vec::Vec<_>>();
    let doc = native_template_document(&selected, &refs, &arena).unwrap();
    assert_eq!(
        print(doc.document(), &PrintOptions::default()),
        "{{ (a + 0xCA_FE) &amp;&amp; true }}"
    );
}

#[test]
fn interpolation_layout_uses_structural_depth_and_width_zero_one_and_seven() {
    for width in [0, 1] {
        assert_eq!(
            format(
                "<template>{{a+b}}</template>",
                PrintOptions {
                    width,
                    ..PrintOptions::default()
                }
            ),
            "{{\n  a +\n    b\n}}"
        );
        assert_eq!(
            format(
                "<template><p>{{a+b}}</p></template>",
                PrintOptions {
                    width,
                    ..PrintOptions::default()
                }
            ),
            "<p>{{\n    a +\n      b\n  }}</p>"
        );
    }
    assert_eq!(
        format(
            "<template>{{a+b}}</template>",
            PrintOptions {
                width: 7,
                ..PrintOptions::default()
            }
        ),
        "{{\n  a + b\n}}"
    );
}

#[test]
fn generated_crlf_retains_original_comment_newlines_and_complete_entities() {
    assert_eq!(
        format(
            "<template>{{a /*x\ny*/ +b}}</template>",
            PrintOptions {
                width: 0,
                line_ending: LineEnding::CrLf,
                ..PrintOptions::default()
            }
        ),
        "{{\r\n  a /*x\ny*/ +\r\n    b\r\n}}"
    );
    assert_eq!(
        format(
            "<template>{{&#32;a&#32;+&#9;b&#32;}}</template>",
            PrintOptions::default()
        ),
        "{{ &#32;a&#32;+&#9;b&#32; }}"
    );
}

#[test]
fn native_verbatim_regions_stay_raw_and_need_no_expression_operand() {
    assert_eq!(
        format(
            "<template><p v-pre title = 'keep'>{{broken( }} &amp; <b>{{raw}}</b></p>{{a+b}}</template>",
            PrintOptions::default()
        ),
        "<p v-pre title='keep'>{{broken( }} &amp; <b>{{raw}}</b></p>{{ a + b }}"
    );
}

#[test]
fn complete_selected_blocks_are_fixed_points_at_extreme_widths() {
    for source in [
        "<template><p z = 'é&amp;' disabled>{{ (a+b) &amp;&amp; ok }}</p></template>",
        "<template>{{&#32;a&#32;+&#9;b&#32;}}<!--kept-->{{'&amp;amp;'}}</template>",
        "<template><p v-pre>{{raw}}</p>{{a /*kept*/ +(b&amp;&amp;c)}}</template>",
        "<template>{{a + //inside\n b}}</template>",
        "<template>{{a //tail\n}}</template>",
    ] {
        for width in [0, 1, 7, 20, 80] {
            let options = PrintOptions {
                width,
                ..PrintOptions::default()
            };
            let output = format(source, options);
            let wrapped = vize_l0::cstr!("<template>{output}</template>");
            assert_eq!(format(&wrapped, options), output);
        }
    }
}

#[test]
fn authored_framing_tails_keep_lf_or_crlf_independently_of_decoded_ast_kind() {
    for (tail, expression) in [
        ("\n  ", "a //tail"),
        ("\r\n  ", "a //tail"),
        ("\n  ", "a //tail&#10;"),
        ("\n  ", "a &#47;*//x*&#47;"),
        ("\n  ", "&#39;//x&#39;"),
    ] {
        let source = vize_l0::cstr!("<template><p>{{{{{expression}{tail}}}}}</p></template>");
        for width in [0, 1, 7, 80] {
            for line_ending in [LineEnding::Lf, LineEnding::CrLf] {
                let options = PrintOptions {
                    width,
                    line_ending,
                    ..PrintOptions::default()
                };
                let opening = if line_ending == LineEnding::Lf {
                    "\n"
                } else {
                    "\r\n"
                };
                let expected = vize_l0::cstr!("<p>{{{{{opening}    {expression}{tail}}}}}</p>");
                let output = format(&source, options);
                assert_eq!(output, expected);
                let replay = vize_l0::cstr!("<template>{output}</template>");
                assert_eq!(format(&replay, options), output);
            }
        }
    }
}
