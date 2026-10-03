use super::*;

#[test]
fn original_lint_categories_do_not_use_dom_or_runtime_tag_resolution() {
    for (body, expected) in [
        ("<div></div>", Kind::Element),
        ("<widget></widget>", Kind::Element),
        ("<my-element></my-element>", Kind::Element),
        ("<marquee></marquee>", Kind::Element),
        ("<blink></blink>", Kind::Element),
        ("<foo:bar></foo:bar>", Kind::Element),
        ("<teleport></teleport>", Kind::Element),
        ("<transition-group></transition-group>", Kind::Element),
        ("<component :is='view'></component>", Kind::Element),
        ("<input is='Foo' :is='view' />", Kind::Element),
        ("<div v-is='view'></div>", Kind::Element),
        ("<Foo is='input'/>", Kind::Component),
        ("<INPUT/>", Kind::Component),
        ("<Teleport/>", Kind::Component),
        ("<Suspense/>", Kind::Component),
        ("<KeepAlive/>", Kind::Component),
        ("<BaseTransition/>", Kind::Component),
        ("<Transition/>", Kind::Component),
        ("<TransitionGroup/>", Kind::Component),
        ("<AÉFoo/>", Kind::Component),
        ("<aÉFoo></aÉFoo>", Kind::Element),
    ] {
        let source = cstr!("<template>{body}</template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let output = tags(&owner);
        assert_eq!(output.len(), 1, "{body}");
        assert_eq!(output[0].1, Ok(expected), "{body}");
        let element = owner.children().next().unwrap().into_element().unwrap();
        assert!(!element.surface().open.is_verbatim(), "{body}");
    }
}

#[test]
fn kind_metadata_does_not_change_existing_attribute_quote_or_mode_facts() {
    let source = "<template><div single='日' double=\"中\" bare=value></div></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let div = owner.children().next().unwrap().into_element().unwrap();
    assert_eq!(div.lint_tag().unwrap().kind(), Kind::Element);
    assert!(!div.surface().open.is_verbatim());
    for (attribute, quote) in div.attributes().zip(["'", "\"", ""]) {
        let value = attribute.surface().value.as_ref().unwrap();
        assert_eq!(
            value.open_quote.as_ref().map_or("", |token| token.text),
            quote
        );
        assert_eq!(
            value.close_quote.as_ref().map_or("", |token| token.text),
            quote
        );
    }
    assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
}

#[test]
fn namespaces_do_not_replace_intrinsic_lint_categories() {
    let source = "<template><svg><g></g><foreignObject><button></button><Foo/></foreignObject></svg><math><mrow></mrow><INPUT/></math></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    assert_eq!(
        tags(&owner),
        [
            ("svg", Ok(Kind::Element)),
            ("g", Ok(Kind::Element)),
            ("foreignObject", Ok(Kind::Element)),
            ("button", Ok(Kind::Element)),
            ("Foo", Ok(Kind::Component)),
            ("math", Ok(Kind::Element)),
            ("mrow", Ok(Kind::Element)),
            ("INPUT", Ok(Kind::Component)),
        ]
    );
}

#[test]
fn complete_template_heads_and_slot_spellings_keep_exact_categories() {
    for (head, expected) in [
        ("", Kind::Element),
        ("v-if='ok'", Kind::Template),
        ("v-else-if='ok'", Kind::Template),
        ("v-else", Kind::Template),
        ("v-for='item in items'", Kind::Template),
        ("v-slot", Kind::Template),
        ("v-slot:name", Kind::Template),
        ("#name", Kind::Template),
        ("v-if:arg.mod='ok'", Kind::Template),
        ("v-iffoo='ok'", Kind::Element),
        ("v-IF='ok'", Kind::Element),
        (":if='ok'", Kind::Element),
        (".slot='value'", Kind::Element),
        ("@slot='handler'", Kind::Element),
        ("v-on:slot='handler'", Kind::Element),
        ("v-slotty='value'", Kind::Element),
    ] {
        let source = cstr!("<template><template {head}></template><slot/><Slot/></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        assert_eq!(
            tags(&owner),
            [
                ("template", Ok(expected)),
                ("slot", Ok(Kind::Slot)),
                ("Slot", Ok(Kind::Component)),
            ],
            "{head}"
        );
    }
}

#[test]
fn exact_pre_keeps_registered_template_kind_and_inherited_templates_refuse() {
    let source = "<template><template v-pre v-if='ok'><slot/><Foo/><div><template v-for='item in items'></template></div></template><template v-if='ok'></template></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    assert_eq!(
        tags(&owner),
        [
            ("template", Ok(Kind::Template)),
            ("slot", Ok(Kind::Slot)),
            ("Foo", Ok(Kind::Component)),
            ("div", Ok(Kind::Element)),
            ("template", Err(Refusal::InheritedTemplate)),
            ("template", Ok(Kind::Template)),
        ]
    );
}

#[test]
fn modified_pre_retains_its_lexical_scope_and_refuses_every_affected_kind() {
    for head in ["v-pre.foo", "v-pre:arg", "v-pre:[key]", "v-pre:"] {
        let source = cstr!(
            "<template><section {head}><template v-if='ok'></template><Foo/><slot/></section><template v-if='ok'></template></template>"
        );
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        assert_eq!(
            tags(&owner),
            [
                ("section", Err(Refusal::AmbiguousVerbatim)),
                ("template", Err(Refusal::AmbiguousVerbatim)),
                ("Foo", Err(Refusal::AmbiguousVerbatim)),
                ("slot", Err(Refusal::AmbiguousVerbatim)),
                ("template", Ok(Kind::Template)),
            ],
            "{head}"
        );
        let section = owner.children().next().unwrap().into_element().unwrap();
        assert!(section.surface().open.is_verbatim());
        assert_eq!(section.surface().open.attrs[0].name.text, head);
        assert!(owner.component().carrier().unsupported.is_empty());
    }
}

#[test]
fn exact_pre_on_the_same_complete_header_removes_ambiguity_in_both_orders() {
    for head in ["v-pre.foo", "v-pre:arg", "v-pre:[key]"] {
        for attributes in [cstr!("{head} v-pre"), cstr!("v-pre {head}")] {
            let source = cstr!(
                "<template><template {attributes} v-if='ok'><Foo/><slot/></template></template>"
            );
            let arena = Allocator::default();
            let owner = selected(&arena, &source);
            assert_eq!(
                tags(&owner),
                [
                    ("template", Ok(Kind::Template)),
                    ("Foo", Ok(Kind::Component)),
                    ("slot", Ok(Kind::Slot)),
                ],
                "{attributes}"
            );
        }
    }
}

#[test]
fn inherited_literal_headers_are_distinct_from_own_pre_freezing() {
    for header in ["v-pre :title.='x'", ":title.='x' v-pre"] {
        let source =
            cstr!("<template><div {header}><input :title.='x' /></div><input /></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let div = owner.children().next().unwrap().into_element().unwrap();
        assert_eq!(div.lint_tag().unwrap().kind(), Kind::Element);
        assert!(!div.lint_tag().unwrap().header_is_literal());
        assert!(div.surface().open.is_verbatim());
        let inherited = div.children().next().unwrap().into_element().unwrap();
        assert!(inherited.lint_tag().unwrap().header_is_literal());
        assert!(inherited.surface().open.is_verbatim());
        let sibling = owner.children().nth(1).unwrap().into_element().unwrap();
        assert!(!sibling.lint_tag().unwrap().header_is_literal());
        assert!(!sibling.surface().open.is_verbatim());
        assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
    }
}
