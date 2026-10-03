use super::*;

#[test]
fn actual_registered_sfc_classification_matches_case_custom_and_opaque_is_controls() {
    for (body, kind, warnings) in [
        ("<input autofocus />", MarkupElementKind::Element, 1),
        ("<widget autofocus></widget>", MarkupElementKind::Element, 1),
        (
            "<my-element autofocus></my-element>",
            MarkupElementKind::Element,
            1,
        ),
        (
            "<marquee autofocus></marquee>",
            MarkupElementKind::Element,
            1,
        ),
        ("<blink autofocus></blink>", MarkupElementKind::Element, 1),
        (
            "<foo:bar autofocus></foo:bar>",
            MarkupElementKind::Element,
            1,
        ),
        (
            "<teleport autofocus></teleport>",
            MarkupElementKind::Element,
            1,
        ),
        (
            "<component :is='view' autofocus></component>",
            MarkupElementKind::Element,
            1,
        ),
        (
            "<input is='Foo' :is='view' autofocus />",
            MarkupElementKind::Element,
            1,
        ),
        (
            "<div v-is='view' autofocus></div>",
            MarkupElementKind::Element,
            1,
        ),
        (
            "<Foo is='input' autofocus/>",
            MarkupElementKind::Component,
            0,
        ),
        ("<INPUT autofocus/>", MarkupElementKind::Component, 0),
        ("<Teleport autofocus/>", MarkupElementKind::Component, 0),
        ("<Suspense autofocus/>", MarkupElementKind::Component, 0),
        ("<KeepAlive autofocus/>", MarkupElementKind::Component, 0),
        (
            "<BaseTransition autofocus/>",
            MarkupElementKind::Component,
            0,
        ),
        ("<Transition autofocus/>", MarkupElementKind::Component, 0),
        (
            "<TransitionGroup autofocus/>",
            MarkupElementKind::Component,
            0,
        ),
        ("<AÉFoo autofocus/>", MarkupElementKind::Component, 0),
        ("<aÉfoo autofocus></aÉfoo>", MarkupElementKind::Element, 1),
    ] {
        let source = cstr!("<template>{body}</template>");
        for locale in [Locale::En, Locale::Ja, Locale::Zh] {
            let (output, result) = parity(&source, locale);
            assert_eq!(output.len(), 1, "{body}");
            assert_eq!(output[0].kind, kind, "{body}");
            assert_eq!(result.error_count, 0, "{body}");
            assert_eq!(result.warning_count, warnings, "{body}");
        }
    }
}

#[test]
fn foreign_namespaces_keep_the_registered_component_exemptions() {
    let source = "<template><svg><g autofocus></g><foreignObject><input autofocus /><Foo autofocus/></foreignObject></svg><math><mrow autofocus></mrow><INPUT autofocus/></math></template>";
    let (output, result) = parity(source, Locale::En);
    assert_eq!(
        output
            .iter()
            .map(|item| (item.tag.as_str(), item.kind))
            .collect::<Vec<_>>(),
        [
            ("svg", MarkupElementKind::Element),
            ("g", MarkupElementKind::Element),
            ("foreignObject", MarkupElementKind::Element),
            ("input", MarkupElementKind::Element),
            ("Foo", MarkupElementKind::Component),
            ("math", MarkupElementKind::Element),
            ("mrow", MarkupElementKind::Element),
            ("INPUT", MarkupElementKind::Component),
        ]
    );
    assert_eq!(result.error_count, 0);
    assert_eq!(result.warning_count, 3);
}

#[test]
fn actual_template_and_slot_observations_keep_structural_heads_and_lookalikes() {
    for (head, kind) in [
        ("", MarkupElementKind::Element),
        ("v-if='ok'", MarkupElementKind::Template),
        ("v-else-if='ok'", MarkupElementKind::Template),
        ("v-else", MarkupElementKind::Template),
        ("v-for='item in items'", MarkupElementKind::Template),
        ("v-slot", MarkupElementKind::Template),
        ("v-slot:name", MarkupElementKind::Template),
        ("#name", MarkupElementKind::Template),
        ("v-if:arg.mod='ok'", MarkupElementKind::Template),
        ("v-iffoo='ok'", MarkupElementKind::Element),
        ("v-IF='ok'", MarkupElementKind::Element),
        (":if='ok'", MarkupElementKind::Element),
        (".slot='value'", MarkupElementKind::Element),
        ("@slot='handler'", MarkupElementKind::Element),
        ("v-slotty='value'", MarkupElementKind::Element),
    ] {
        let source = cstr!(
            "<template><template {head} autofocus></template><slot autofocus></slot><Slot autofocus/></template>"
        );
        let (output, _) = parity(&source, Locale::En);
        assert_eq!(
            output
                .iter()
                .map(|item| (item.tag.as_str(), item.kind))
                .collect::<Vec<_>>(),
            [
                ("template", kind),
                ("slot", MarkupElementKind::Slot),
                ("Slot", MarkupElementKind::Component),
            ],
            "{head}"
        );
    }
}
