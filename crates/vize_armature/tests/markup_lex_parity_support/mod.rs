//! Complete authored event packets; parity alone would compare the same lexer.

use super::{Component, Document, Profile, traces_with};

fn assert_events<P: Profile>(source: &str, expected: &[&str]) {
    let (published, native) = traces_with::<P>(source, false);
    for (entry, trace) in [("published", published), ("native", native)] {
        let actual: Vec<_> = trace.events.iter().map(|event| event.as_str()).collect();
        assert_eq!(actual, expected, "{entry}: {source}");
    }
}

#[test]
fn document_rawtext_names_fold_both_leading_bytes() {
    for name in ["script", "Script", "sCript", "SCRIPT"] {
        assert_events::<Document>(
            &format!("<{name}><b/>{{{{x}}}}&amp;</sCrIpT><p/>"),
            &[
                "open-name:1..7",
                "open-end:7",
                "text:8..22",
                "close-name:24..30",
                "open-name:32..33",
                "self-close:34",
                "end",
            ],
        );
    }
    for name in ["style", "Style", "sTyle", "STYLE"] {
        assert_events::<Document>(
            &format!("<{name}><b/>{{{{x}}}}&amp;</sTyLe><p/>"),
            &[
                "open-name:1..6",
                "open-end:6",
                "text:7..21",
                "close-name:23..28",
                "open-name:30..31",
                "self-close:32",
                "end",
            ],
        );
    }
}

#[test]
fn document_rcdata_names_preserve_entities_interpolations_and_source_spans() {
    for name in ["title", "Title", "tItle", "TITLE"] {
        assert_events::<Document>(
            &format!("<{name}><b/>{{{{x}}}}&amp;</tItLe><p/>"),
            &[
                "open-name:1..6",
                "open-end:6",
                "text:7..11",
                "interpolation:13..14",
                "text-entity:'&':16..21",
                "close-name:23..28",
                "open-name:30..31",
                "self-close:32",
                "end",
            ],
        );
    }
    for name in ["textarea", "Textarea", "tExtarea", "TEXTAREA"] {
        assert_events::<Document>(
            &format!("<{name}><b/>{{{{x}}}}&amp;</tExTaReA><p/>"),
            &[
                "open-name:1..9",
                "open-end:9",
                "text:10..14",
                "interpolation:16..17",
                "text-entity:'&':19..24",
                "close-name:26..34",
                "open-name:36..37",
                "self-close:38",
                "end",
            ],
        );
    }
}

#[test]
fn component_uppercase_names_remain_ordinary_markup() {
    for name in ["Script", "sCript", "SCRIPT"] {
        assert_events::<Component>(
            &format!("<{name}><b/>{{{{x}}}}&amp;</{name}><p/>"),
            &[
                "open-name:1..7",
                "open-end:7",
                "open-name:9..10",
                "self-close:11",
                "interpolation:14..15",
                "text-entity:'&':17..22",
                "close-name:24..30",
                "open-name:32..33",
                "self-close:34",
                "end",
            ],
        );
    }
    // The existing component machine accepts case variants after its initial
    // two lowercase bytes. This correction does not broaden that legacy rule.
    assert_events::<Component>(
        "<scrIPT><b/>{{x}}&amp;</script><p/>",
        &[
            "open-name:1..7",
            "open-end:7",
            "text:8..22",
            "close-name:24..30",
            "open-name:32..33",
            "self-close:34",
            "end",
        ],
    );
}

#[test]
fn document_special_name_prefixes_remain_ordinary_markup() {
    for source in ["<SCRIPTx><b/></SCRIPTx>", "<TEXTARE><b/></TEXTARE>"] {
        assert_events::<Document>(
            source,
            &[
                "open-name:1..8",
                "open-end:8",
                "open-name:10..11",
                "self-close:12",
                "close-name:15..22",
                "end",
            ],
        );
    }
}

#[test]
fn document_rawtext_keeps_authored_utf8_byte_offsets() {
    assert_events::<Document>(
        "<SCRIPT>日本<b/></script>",
        &[
            "open-name:1..7",
            "open-end:7",
            "text:8..18",
            "close-name:20..26",
            "end",
        ],
    );
}
