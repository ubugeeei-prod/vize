//! Selected Vue 3 lint facts retain original custody and bounded grammar.

use vize_l0::{
    Allocator, SourceRoot, Span,
    config::{VueDialect, VueVersion},
    cstr,
};
use vize_l1::{
    Element, ElementClose, OpenTag, SurfaceParseOptions, SurfaceTree, Token, check_fidelity,
    container::{Vue, vue::DescriptorOptions},
    markup::{
        NativeChildren, NativeComponent, NativeLintTagKind as Kind,
        NativeLintTagRefusal as Refusal, NativeTemplateComponent, NativeTemplateGrammar,
    },
    render,
};

fn selected<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateComponent<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap()
}

fn rendered(tree: &SurfaceTree<'_>) -> vize_l0::String {
    let mut text = vize_l0::String::default();
    render(tree, &mut |piece| text.push_str(piece));
    text
}

fn collect<'a>(
    component: &NativeComponent<'a>,
    children: NativeChildren<'_, 'a>,
    output: &mut Vec<(&'a str, Result<Kind, Refusal>)>,
) {
    for child in children {
        if let Some(element) = child.into_element() {
            let kind = element.lint_tag().map(|receipt| {
                assert!(core::ptr::eq(receipt.component(), component));
                assert!(core::ptr::eq(receipt.element(), element.surface()));
                let block = component.block();
                let opening = block.span_of(element.surface().open.lt_name.text).unwrap();
                assert_eq!(receipt.span(), Span::new(opening.start + 1, opening.end));
                assert_eq!(
                    receipt.span(),
                    block.span_of(element.surface().tag()).unwrap()
                );
                assert!(block.contains_block_span(receipt.span()));
                receipt.kind()
            });
            output.push((element.surface().tag(), kind));
            collect(component, element.children(), output);
        }
    }
}

fn tags<'a>(owner: &NativeTemplateComponent<'a>) -> Vec<(&'a str, Result<Kind, Refusal>)> {
    let component = owner.component();
    let before = rendered(&component.carrier().tree);
    let mut output = Vec::new();
    collect(component, owner.children(), &mut output);
    assert_eq!(rendered(&component.carrier().tree), before);
    assert_eq!(before.as_str(), component.block().source());
    assert_eq!(check_fidelity(&component.carrier().tree), Ok(()));
    output
}

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
fn exact_pre_freezes_structural_templates_but_preserves_slot_and_component_kinds() {
    let source = "<template><template v-pre v-if='ok'><slot/><Foo/><div><template v-for='item in items'></template></div></template><template v-if='ok'></template></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    assert_eq!(
        tags(&owner),
        [
            ("template", Ok(Kind::Element)),
            ("slot", Ok(Kind::Slot)),
            ("Foo", Ok(Kind::Component)),
            ("div", Ok(Kind::Element)),
            ("template", Ok(Kind::Element)),
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
                    ("template", Ok(Kind::Element)),
                    ("Foo", Ok(Kind::Component)),
                    ("slot", Ok(Kind::Slot)),
                ],
                "{attributes}"
            );
        }
    }
}

#[test]
fn absolute_original_tag_spans_keep_selected_roles_and_unicode_prefixes() {
    let source = "<!--🦀--><script lang=ts>const fake='<INPUT autofocus />'</script><template>日本語<div><input autofocus /><AÉFoo/></div></template><script setup lang=ts>const other='<slot />'</script>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    assert_eq!(owner.grammar(), NativeTemplateGrammar::TypeScriptModule);
    assert!(core::ptr::eq(
        owner.component().block().root_source(),
        source
    ));
    assert_eq!(
        tags(&owner),
        [
            ("div", Ok(Kind::Element)),
            ("input", Ok(Kind::Element)),
            ("AÉFoo", Ok(Kind::Component)),
        ]
    );
    let div = owner.children().nth(1).unwrap().into_element().unwrap();
    let input = div.children().next().unwrap().into_element().unwrap();
    let receipt = input.lint_tag().unwrap();
    let start = u32::try_from(source.find("<input autofocus").unwrap() + 1).unwrap();
    assert_eq!(receipt.span(), Span::new(start, start + 5));
    assert_eq!(
        input.attributes().next().unwrap().surface().name.text,
        "autofocus"
    );
}

#[test]
fn equal_sources_keep_distinct_owners_and_short_receipts_survive_reborrowing() {
    let source = "<template><Foo/><input autofocus /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let foreign = selected(&arena, source);
    let original_surface = {
        let first = owner.children().next().unwrap().into_element().unwrap();
        let other = foreign.children().next().unwrap().into_element().unwrap();
        let receipt = first.lint_tag().unwrap();
        let other_receipt = other.lint_tag().unwrap();
        assert_eq!(receipt.kind(), other_receipt.kind());
        assert_eq!(receipt.span(), other_receipt.span());
        assert!(!core::ptr::eq(
            receipt.component(),
            other_receipt.component()
        ));
        assert!(!core::ptr::eq(receipt.element(), other_receipt.element()));
        assert!(core::ptr::eq(receipt.component(), owner.component()));
        receipt.element() as *const Element<'_>
    };
    let moved = core::hint::black_box(owner);
    let first = moved.children().next().unwrap().into_element().unwrap();
    let receipt = first.lint_tag().unwrap();
    assert!(core::ptr::eq(receipt.element(), original_surface));
    assert!(core::ptr::eq(receipt.component(), moved.component()));
    assert_eq!(
        tags(&moved),
        [("Foo", Ok(Kind::Component)), ("input", Ok(Kind::Element))]
    );
}

#[test]
fn arbitrary_native_blocks_and_raw_compatibility_construction_cannot_mint_selected_facts() {
    let source = "<template><div v-pre><Foo/></div></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let block = owner.component().block();
    let arbitrary = NativeComponent::parse_in(&arena, block).unwrap();
    let arbitrary_div = arbitrary.children().next().unwrap().into_element().unwrap();
    assert!(matches!(
        arbitrary_div.lint_tag(),
        Err(Refusal::Unavailable)
    ));
    assert_eq!(arbitrary.block(), block);
    assert_eq!(
        rendered(&arbitrary.carrier().tree),
        rendered(&owner.component().carrier().tree)
    );
    let compat = vize_l1::parse(&arena, block.source());
    assert_eq!(
        cstr!("{:?}", owner.component().carrier().tree.children),
        cstr!("{:?}", compat.0.children)
    );
    let raw = OpenTag {
        lt_name: Token::present("", "<div"),
        attrs: vize_l0::Vec::new_in(&&arena),
        slash: None,
        gt: Token::present("", ">"),
    };
    assert!(!raw.is_verbatim());
    assert_eq!(
        cstr!("{:?}", raw.lt_name),
        "Token { leading: \"\", text: \"<div\", status: Present }"
    );
    if cfg!(target_pointer_width = "64") {
        assert_eq!(core::mem::size_of::<Token<'_>>(), 40);
        assert_eq!(core::mem::size_of::<Option<Token<'_>>>(), 40);
        assert_eq!(core::mem::size_of::<OpenTag<'_>>(), 144);
        assert_eq!(core::mem::size_of::<Element<'_>>(), 248);
    }
}

#[test]
fn incomplete_headers_refuse_without_erasing_recovery_or_claiming_body_completion() {
    let source = "<template><div title='unfinished</template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let div = owner.children().next().unwrap().into_element().unwrap();
    assert!(matches!(div.lint_tag(), Err(Refusal::IncompleteHeader)));
    assert!(div.surface().open.gt.is_missing());
    assert!(!owner.component().carrier().errors.is_empty());
    assert_eq!(owner.component().block().source(), "<div title='unfinished");
    assert_eq!(
        rendered(&owner.component().carrier().tree).as_str(),
        owner.component().block().source()
    );
    let arbitrary =
        NativeComponent::parse_in(&arena, SourceRoot::new("<div").unwrap().whole_block()).unwrap();
    let incomplete = arbitrary.children().next().unwrap().into_element().unwrap();
    assert!(matches!(
        incomplete.lint_tag(),
        Err(Refusal::IncompleteHeader)
    ));
    let complete_header = selected(&arena, "<template><div></template>");
    let div = complete_header
        .children()
        .next()
        .unwrap()
        .into_element()
        .unwrap();
    assert!(matches!(div.surface().close, ElementClose::Missing));
    assert_eq!(div.lint_tag().unwrap().kind(), Kind::Element);
    assert_eq!(
        check_fidelity(&complete_header.component().carrier().tree),
        Ok(())
    );
}

#[test]
fn live_recovery_ends_ambiguous_scope_without_rewriting_original_observations() {
    let source =
        "<template><a v-pre.foo><span><a></a></span></a><input :autofocus='outside' /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let arbitrary = NativeComponent::parse_in(&arena, owner.component().block()).unwrap();
    assert_eq!(
        cstr!("{:?}", owner.component().carrier().tree.children),
        cstr!("{:?}", arbitrary.carrier().tree.children)
    );
    assert_eq!(
        cstr!("{:?}", owner.component().carrier().errors),
        cstr!("{:?}", arbitrary.carrier().errors)
    );
    assert_eq!(
        tags(&owner),
        [
            ("a", Err(Refusal::AmbiguousVerbatim)),
            ("span", Err(Refusal::AmbiguousVerbatim)),
            ("a", Ok(Kind::Element)),
            ("input", Ok(Kind::Element)),
        ]
    );
    let outer = owner.children().next().unwrap().into_element().unwrap();
    assert!(matches!(outer.surface().close, ElementClose::Implicit));
    assert!(outer.surface().open.is_verbatim());
    let inner = owner.children().nth(1).unwrap().into_element().unwrap();
    assert!(!inner.surface().open.is_verbatim());
    assert_eq!(
        rendered(&owner.component().carrier().tree).as_str(),
        owner.component().block().source()
    );
}

#[test]
fn non_ascii_initial_tag_bytes_remain_text_without_fabricated_elements() {
    let arena = Allocator::default();
    let owner = selected(&arena, "<template><ÉFoo /><éfoo /></template>");
    assert!(tags(&owner).is_empty());
    let ascii_initial = selected(&arena, "<template><AÉFoo/><aÉfoo></aÉfoo></template>");
    assert_eq!(
        tags(&ascii_initial),
        [("AÉFoo", Ok(Kind::Component)), ("aÉfoo", Ok(Kind::Element))]
    );
}
