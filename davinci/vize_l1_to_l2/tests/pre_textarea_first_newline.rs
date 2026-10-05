//! #7889: native lowering keeps complete authored spans while changing text.
mod support;

use vize_l2::dump::Op;

use support::{assert_transformed_sound, with_transformed};

#[test]
fn native_first_newline_rule_keeps_second_newline_and_source_spans() {
    for (source, expected) in [
        ("<pre>\nline</pre>", "line"),
        ("<pre>\n\ntwo</pre>", "\ntwo"),
        ("<textarea>\n\ntwo</textarea>", "\ntwo"),
        ("<pre>\r\nline\r\nend</pre>", "line\nend"),
        ("<pre>\nline&#13;&#10;end</pre>", "line\nend"),
        ("<pre>\nline\r&NewLine;end</pre>", "line\nend"),
        (
            "<pre>\nline&amp;#13;&#10;end</pre>",
            "line&amp;#13;&#10;end",
        ),
        ("<pre>&#10;line</pre>", "line"),
        ("<textarea>&#x0A;value</textarea>", "value"),
        ("<pre>&#13;&#10;line</pre>", "line"),
        ("<pre>&NewLine;line</pre>", "line"),
        ("<pre>&amp;#10;line</pre>", "&amp;#10;line"),
        ("<pre> \nline</pre>", " \nline"),
        ("<textarea> \nvalue</textarea>", " \nvalue"),
        ("<textarea>\n  value   end\n</textarea>", "  value   end\n"),
        ("<pre>\rline</pre>", "\rline"),
        ("<pre v-pre>\n{{ a }}</pre>", "{{ a }}"),
    ] {
        with_transformed(source, |lowered, folio, _, _| {
            assert!(
                lowered.diagnostics.is_empty(),
                "{source}: {:?}",
                lowered.diagnostics
            );
            let Some(Op::Element(element)) = folio.ops.first() else {
                panic!("expected native element");
            };
            let [Op::Text(text)] = element.children.as_slice() else {
                panic!("expected native text: {:?}", element.children);
            };
            assert_eq!(text.content, expected, "{source}");
            assert_eq!(
                source.get(text.span.start as usize..text.span.end as usize),
                source
                    .split_once('>')
                    .and_then(|(_, rest)| rest.split_once("</"))
                    .map(|(text, _)| text),
                "rendered content must retain its entire original authored range"
            );
        });
        assert_transformed_sound(source, "first-newline");
    }
}

#[test]
fn native_comment_option_controls_first_visible_text_without_changing_spans() {
    let source = "<pre><!-- keep -->\nline</pre>";
    let allocator = vize_l0::Allocator::new();
    let (tree, errors) = vize_l1::parse(&allocator, source);
    assert!(errors.is_empty());
    for comments in [false, true] {
        let lowered = if comments {
            vize_l1_to_l2::lower_preserving_comments(&allocator, &tree, &errors)
        } else {
            vize_l1_to_l2::lower(&allocator, &tree, &errors)
        };
        assert!(lowered.diagnostics.is_empty());
        let page = vize_l2::dump::Page::of(&lowered.root.ops);
        let [Op::Element(element)] = page.ops.as_slice() else {
            panic!("expected pre");
        };
        let Some(Op::Text(text)) = element.children.last() else {
            panic!("expected text");
        };
        assert_eq!(text.content, if comments { "\nline" } else { "line" });
        assert_eq!(
            &source[text.span.start as usize..text.span.end as usize],
            "\nline"
        );
        assert_eq!(
            matches!(element.children.first(), Some(Op::Comment(_))),
            comments
        );
    }
}
