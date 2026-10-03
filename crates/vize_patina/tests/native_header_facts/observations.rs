use super::support::{selected, span};
use vize_l0::{Allocator, cstr, fact::FactGroup};
use vize_l1::markup::NativeLintTagKind;
use vize_patina::native::{
    NativeSyntaxLint,
    header_facts::{
        NativeBindingKind, NativeLintAttributes, NativeLintHeaders, NativeUnsupportedAria,
        UnsupportedAriaDemand,
    },
};

#[test]
fn actual_original_ordinals_ranges_and_opaque_values_are_retained_in_sdk_tables() {
    let source = "<script>const π = 1</script>\n<template><!-- before --><meta title='字' aria-hidden='&amp;' :role.prop='opaque' @click='handler' /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner
        .children()
        .find_map(|child| child.into_element())
        .unwrap();
    assert_eq!(element.ordinal(), 1);
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let receipt = lint.header_facts(&element).unwrap();
    let facts = receipt.facts::<UnsupportedAriaDemand>();
    let headers = facts.get::<NativeLintHeaders>().unwrap();
    assert_eq!(headers.len(), 1);
    let header = headers
        .get(&u32::try_from(element.ordinal()).unwrap())
        .unwrap();
    assert_eq!(header.tag(), "meta");
    assert_eq!(header.kind(), NativeLintTagKind::Element);
    assert_eq!(header.span(), span(source, "meta"));
    assert_eq!(
        header.opening(),
        span(
            source,
            "<meta title='字' aria-hidden='&amp;' :role.prop='opaque' @click='handler' />"
        )
    );
    assert!(!header.header_is_literal());
    let attributes = facts.get::<NativeLintAttributes>().unwrap();
    assert_eq!(attributes.len(), 4);
    for ((original, (key, fact)), spelling) in element.attributes().zip(attributes.iter()).zip([
        "title='字'",
        "aria-hidden='&amp;'",
        ":role.prop='opaque'",
        "@click='handler'",
    ]) {
        assert_eq!(usize::try_from(*key).unwrap(), original.ordinal());
        assert_eq!(fact.span(), span(source, spelling));
    }
    assert_eq!(attributes.get(&0).unwrap().name(), "title");
    assert_eq!(attributes.get(&1).unwrap().name(), "aria-hidden");
    assert_eq!(attributes.get(&2).unwrap().name(), "role");
    assert_eq!(attributes.get(&2).unwrap().kind(), NativeBindingKind::Bind);
    assert_eq!(attributes.get(&3).unwrap().kind(), NativeBindingKind::Other);
    let counterexamples = facts.get::<NativeUnsupportedAria>().unwrap();
    let actual: Vec<_> = counterexamples
        .iter()
        .map(|(key, fact)| (*key, fact.header_key(), fact.span()))
        .collect();
    assert_eq!(
        actual,
        [
            (1, 1, span(source, "aria-hidden='&amp;'")),
            (2, 1, span(source, ":role.prop='opaque'"))
        ]
    );
    assert_eq!(
        cstr!("{:?}", counterexamples),
        cstr!("{:?}", receipt.unsupported_aria().unwrap())
    );
    assert!(core::ptr::eq(
        counterexamples,
        receipt.unsupported_aria().unwrap()
    ));
    assert_eq!(
        NativeUnsupportedAria::DEPENDS,
        vize_l0::fact::Demand::NONE
            .with(NativeLintHeaders::ID)
            .with(NativeLintAttributes::ID)
    );
}

#[test]
fn exact_authored_tag_and_attribute_grammar_determine_counterexamples() {
    for (tag, count) in [
        ("meta", 5),
        ("html", 5),
        ("script", 5),
        ("style", 5),
        ("META", 0),
        ("Meta", 0),
        ("div", 0),
        ("x-meta", 0),
        ("slot", 0),
        ("template", 0),
    ] {
        let ending = if matches!(tag, "script" | "style") {
            cstr!("></{tag}>")
        } else {
            " />".into()
        };
        let source = cstr!(
            "<template><{tag} aria-label role :aria-hidden.camel='x' v-bind:role.prop='x' .aria-x='x' ARIA-hidden ARole @role='x'{ending}</template>"
        );
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let receipt = lint.header_facts(&element).unwrap();
        assert_eq!(receipt.unsupported_aria().unwrap().len(), count, "{source}");
        assert_eq!(
            receipt
                .facts::<UnsupportedAriaDemand>()
                .get::<NativeLintAttributes>()
                .unwrap()
                .len(),
            8
        );
        for (key, _) in receipt.unsupported_aria().unwrap().iter() {
            assert_eq!(
                receipt.verify(&receipt.unsupported_aria_chain(*key).unwrap().unwrap()),
                Ok(())
            );
        }
        assert_eq!(receipt.unsupported_aria_chain(99).unwrap(), None);
    }
}

#[test]
fn exact_own_and_inherited_pre_keep_original_literal_head_semantics() {
    for source in [
        "<template><meta v-pre aria-hidden :role='x' .aria-x='x' /></template>",
        "<template><div v-pre><meta aria-hidden :role='x' .aria-x='x' /></div></template>",
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let root = owner.children().next().unwrap().into_element().unwrap();
        let inherited = root.surface().tag() == "div";
        let element = if inherited {
            root.children().next().unwrap().into_element().unwrap()
        } else {
            root
        };
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let receipt = lint.header_facts(&element).unwrap();
        let facts = receipt.facts::<UnsupportedAriaDemand>();
        assert_eq!(receipt.unsupported_aria().unwrap().len(), 1);
        assert_eq!(
            facts
                .get::<NativeLintHeaders>()
                .unwrap()
                .iter()
                .next()
                .unwrap()
                .1
                .header_is_literal(),
            inherited
        );
        assert!(
            facts
                .get::<NativeLintAttributes>()
                .unwrap()
                .iter()
                .all(|(_, fact)| fact.kind() == NativeBindingKind::Static)
        );
    }
}
