use super::create_on_name;
use vize_l0::{String, camelize, capitalize};

#[test]
fn component_event_keys_preserve_complete_camelization() {
    for (source, expected) in [
        ("", "on"),
        ("close", "onClose"),
        ("generatePdf", "onGeneratePdf"),
        ("generateImage", "onGenerateImage"),
        ("toggleAnnotator", "onToggleAnnotator"),
        ("copyAlt", "onCopyAlt"),
        ("update-thing", "onUpdateThing"),
        ("my-custom-event", "onMyCustomEvent"),
        ("update:modelValue", "onUpdate:modelValue"),
        ("update:my-prop", "onUpdate:myProp"),
        ("-thing", "onThing"),
        ("--thing", "on-Thing"),
        ("thing--value", "onThing-Value"),
        ("thing-", "onThing-"),
        ("-", "on-"),
        ("--", "on--"),
        ("-1", "on1"),
        ("-_value", "on_value"),
        ("thing-9value", "onThing9value"),
        ("thing-_value", "onThing_value"),
        ("thing-:value", "onThing-:value"),
        ("thing- value", "onThing- value"),
        ("é-event", "onéEvent"),
        ("-é-event", "on-éEvent"),
        ("日本語-更新", "on日本語-更新"),
        ("😀-event", "on😀Event"),
        ("ß-value", "onßValue"),
        ("a\0-b", "onA\0B"),
    ] {
        assert_eq!(create_on_name(source).as_str(), expected, "{source:?}");
    }
}

#[test]
fn component_event_keys_match_the_retained_whole_name_composition() {
    // This freezes the prior complete implementation as an independent oracle.
    fn retained_name(event: &str) -> String {
        let camel = camelize(event);
        let cap = capitalize(&camel);
        let mut result = String::with_capacity(2 + cap.len());
        result.push_str("on");
        result.push_str(&cap);
        result
    }

    let pieces = [
        "", "a", "Z", "9", "_", "-", ":", " ", "é", "日本", "😀", "\0",
    ];
    for first in pieces {
        for middle in pieces {
            for last in pieces {
                let source = format!("{first}{middle}{last}");
                assert_eq!(
                    create_on_name(&source),
                    retained_name(&source),
                    "{source:?}"
                );
            }
        }
    }
}
