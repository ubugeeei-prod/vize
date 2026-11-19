use super::*;
use vize_l1::dialect::vue2::text::TextBoundaryKind;
use vize_l1::embed::syntax::EmbedHole;

#[test]
fn same_buffer_reparse_copied_source_and_other_occurrences_cannot_supply_foreign_children() {
    let source = "{{ value | upper }}{{ value | upper }}";
    let copied = source.to_owned();
    let arena = Allocator::default();
    let owner = surface::parse_component(&arena, source).unwrap();
    let reparse = surface::parse_component(&arena, source).unwrap();
    let copy = surface::parse_component(&arena, &copied).unwrap();
    for foreign in [&reparse, &copy] {
        for child in foreign.children() {
            assert_eq!(
                owner.text_for(child).unwrap_err(),
                TextRefusal::ForeignComponent
            );
        }
    }
    let mut children = owner.children();
    let first = owner.text_for(children.next().unwrap()).unwrap();
    let second = owner.text_for(children.next().unwrap()).unwrap();
    assert!(core::ptr::eq(first.binding(), &owner.bindings()[0]));
    assert!(core::ptr::eq(second.binding(), &owner.bindings()[1]));
    assert_eq!((first.child().ordinal(), second.child().ordinal()), (0, 1));
    assert!(!core::ptr::eq(
        first.child().surface(),
        second.child().surface()
    ));
    assert_ne!(first.binding().span(), second.binding().span());
}

#[test]
fn global_lexical_and_encoded_delimiter_observations_refuse_without_discarding_syntax() {
    for (source, refusal) in [
        (
            "<!--bad--!>{{ value | upper }}",
            TextRefusal::RecoveredComponent,
        ),
        (
            "&#123;&#123; x &#125;&#125; {{ value | upper }}",
            TextRefusal::Boundary(TextBoundaryKind::EncodedDelimiter),
        ),
    ] {
        let arena = Allocator::default();
        let owner = surface::parse_component(&arena, source).unwrap();
        assert_eq!(check_fidelity(owner.tree()), Ok(()));
        assert_eq!(owner.bindings().len(), 1);
        assert!(owner.bindings()[0].admitted().is_some());
        let mut observed = Vec::new();
        visits(&owner, owner.children(), &mut observed);
        assert_eq!(observed.len(), 1);
        assert_eq!(observed.pop().unwrap().unwrap_err(), refusal);
        assert!(!owner.errors().is_empty() || !owner.unsupported().is_empty());
    }
}

#[test]
fn local_chain_boundaries_and_actual_language_holes_keep_later_original_callbacks_eligible() {
    for (source, refusal) in [
        (
            "{{ x | add(,2) }}{{ next | upper }}",
            TextRefusal::Boundary(TextBoundaryKind::UnsupportedArgumentList),
        ),
        (
            "{{ &#13;value }}{{ next | upper }}",
            TextRefusal::Boundary(TextBoundaryKind::HistoricalLineSeparator),
        ),
        (
            "{{ 雪 + }}{{ next | upper }}",
            TextRefusal::NativeHole(EmbedHole::Syntax),
        ),
        (
            "{{ value | wrap(1 +) }}{{ next | upper }}",
            TextRefusal::NativeHole(EmbedHole::Syntax),
        ),
    ] {
        let arena = Allocator::default();
        let owner = surface::parse_component(&arena, source).unwrap();
        assert_eq!(check_fidelity(owner.tree()), Ok(()));
        assert_eq!(owner.bindings().len(), 2);
        assert!(owner.bindings()[0].admitted().is_none());
        let mut children = owner.children();
        assert_eq!(
            owner.text_for(children.next().unwrap()).unwrap_err(),
            refusal
        );
        let clean = owner.text_for(children.next().unwrap()).unwrap();
        assert!(core::ptr::eq(clean.binding(), &owner.bindings()[1]));
        assert_eq!(clean.chain().base().source().text(), "next");
        assert_eq!(clean.chain().filters().len(), 1);
        assert_eq!(clean.chain().filters()[0].name().text(), "upper");
        assert!(clean.chain().filters()[0].arguments().is_empty());
    }
}

#[test]
fn original_vue2_verbatim_mode_and_modified_pre_keep_distinct_callback_membership() {
    let source = "<div v-pre><b>{{ value | upper }}</b></div><div v-pre.foo>{{ value | upper }}</div><div v-pre:arg>{{ value | upper }}</div>";
    let arena = Allocator::default();
    let owner = surface::parse_component(&arena, source).unwrap();
    assert_eq!(owner.bindings().len(), 2);
    assert_eq!(owner.children().len(), 3);
    let mut roots = owner.children();
    let verbatim = roots.next().unwrap();
    let inner = verbatim.children().unwrap().next().unwrap();
    let text = inner.children().unwrap().next().unwrap();
    assert!(matches!(text.surface(), SurfaceChild::Text(_)));
    assert_eq!(
        owner.text_for(text).unwrap_err(),
        TextRefusal::NotInterpolation
    );
    for (index, original) in roots.enumerate() {
        let view = owner
            .text_for(original.children().unwrap().next().unwrap())
            .unwrap();
        assert!(core::ptr::eq(view.binding(), &owner.bindings()[index]));
        assert_eq!(view.chain().base().source().text(), "value");
        assert_eq!(view.chain().filters()[0].name().text(), "upper");
    }
    assert_eq!(check_fidelity(owner.tree()), Ok(()));
}

#[test]
fn recovered_ancestor_is_latched_even_when_the_direct_parent_has_complete_tokens() {
    let arena = Allocator::default();
    let owner = surface::parse_component(&arena, "<a><b>{{ value | upper }}</b>").unwrap();
    assert!(owner.errors().is_empty());
    assert!(owner.bindings()[0].admitted().is_some());
    let outer = owner.children().next().unwrap();
    let inner = outer.children().unwrap().next().unwrap();
    let text = inner.children().unwrap().next().unwrap();
    assert_eq!(text.parent_element().unwrap().tag(), "b");
    assert_eq!(
        owner.text_for(text).unwrap_err(),
        TextRefusal::RecoveredComponent
    );
    assert_eq!(check_fidelity(owner.tree()), Ok(()));
}

#[test]
fn genuine_interactive_authored_membership_refuses_while_clean_original_sibling_remains_eligible() {
    let source = "<a>{{ before }}<a>{{ inner }}</a>{{ tail }}</a><i>{{ clean | upper }}</i>";
    let arena = Allocator::default();
    let block = SourceRoot::new(source).unwrap().whole_block();
    let owner = surface::parse_component_with_authored_block(&arena, block);
    assert!(owner.errors().is_empty());
    assert_eq!(check_fidelity(owner.tree()), Ok(()));
    let authored = owner.authored().expect("actual interactive repair witness");
    assert_eq!(check_fidelity(authored), Ok(()));
    let mut alternative = Vec::new();
    visits(&owner, owner.authored_children().unwrap(), &mut alternative);
    assert_eq!(alternative.len(), owner.bindings().len());
    for refused in alternative {
        assert_eq!(refused.unwrap_err(), TextRefusal::RecoveredComponent);
    }
    let clean = owner
        .children()
        .find(|child| {
            child.parent_element().is_none()
                && matches!(child.surface(), SurfaceChild::Element(node) if node.tag()=="i")
        })
        .unwrap();
    let clean = owner
        .text_for(clean.children().unwrap().next().unwrap())
        .unwrap();
    assert_eq!(clean.chain().base().source().text(), "clean");
    assert!(core::ptr::eq(
        clean.binding(),
        owner.bindings().last().unwrap()
    ));
    assert_eq!(clean.child().parent_element().unwrap().tag(), "i");
    assert_eq!(clean.child().ordinal(), 0);
    let normal_pointer = clean.child().surface() as *const SurfaceChild<'_>;
    let authored_clean = owner
        .authored_children()
        .unwrap()
        .find(|child| matches!(child.surface(), SurfaceChild::Element(node) if node.tag()=="i"))
        .unwrap();
    let authored_clean = authored_clean.children().unwrap().next().unwrap();
    assert!(!core::ptr::eq(authored_clean.surface(), normal_pointer));
    assert_eq!(
        owner.text_for(authored_clean).unwrap_err(),
        TextRefusal::RecoveredComponent
    );
}
