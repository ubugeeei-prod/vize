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
