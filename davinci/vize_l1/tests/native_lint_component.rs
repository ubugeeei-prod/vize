//! Bare lint custody preserves the original default Vue 3 parser observations.

#[cfg(test)]
mod native_lint_component {
    use vize_l0::{Allocator, ErrorCode, SourceRoot, Span, cstr};
    use vize_l1::{
        ElementClose, SurfaceChild, check_fidelity,
        markup::{
            DirectiveNameError, NativeComponent, NativeLintComponent, NativeLintTagKind,
            NativeLintTagRefusal,
        },
        render,
    };

    fn rendered(owner: &NativeLintComponent<'_>) -> vize_l0::String {
        let mut source = vize_l0::String::default();
        render(&owner.component().carrier().tree, &mut |piece| {
            source.push_str(piece)
        });
        source
    }

    #[test]
    fn bare_source_has_original_root_element_and_lint_custody() {
        let arena = Allocator::default();
        let source = "<div>Content</div>";
        let block = SourceRoot::new(source).unwrap().whole_block();
        let owner = NativeLintComponent::parse_in(&arena, block).unwrap();
        let component = owner.component();
        assert!(core::ptr::eq(component.allocator(), &arena));
        assert!(core::ptr::eq(component.block().source(), source));
        assert!(core::ptr::eq(component.carrier().tree.source, source));
        assert!(component.carrier().errors.is_empty());
        assert!(component.carrier().unsupported.is_empty());
        assert!(component.carrier().authored.is_none());
        let child = owner.children().next().unwrap();
        assert!(core::ptr::eq(child.component(), component));
        assert!(child.parent_element().is_none());
        assert_eq!(child.ordinal(), 0);
        let element = child.into_element().unwrap();
        let tag = element.lint_tag().unwrap();
        assert!(core::ptr::eq(tag.component(), component));
        assert!(core::ptr::eq(tag.element(), element.surface()));
        assert_eq!(tag.kind(), NativeLintTagKind::Element);
        assert_eq!(tag.span(), Span::new(1, 4));
        assert!(!tag.header_is_literal());
        assert!(!tag.in_table_context());
        assert!(!tag.in_recovery_context());
        assert_eq!(rendered(&owner), source);
        assert_eq!(check_fidelity(&component.carrier().tree), Ok(()));
    }

    #[test]
    fn lint_header_span_retains_original_absolute_unicode_block() {
        let arena = Allocator::default();
        let source = "前置<div title='値'>内容</div>後置";
        let root = SourceRoot::new(source).unwrap();
        let content = source
            .strip_prefix("前置")
            .unwrap()
            .strip_suffix("後置")
            .unwrap();
        let block = root.block(content, "前置".len() as u32).unwrap();
        let owner = NativeLintComponent::parse_in(&arena, block).unwrap();
        let element = owner.children().next().unwrap().into_element().unwrap();
        let receipt = element.lint_tag().unwrap();
        assert_eq!(receipt.span(), Span::new(7, 10));
        assert!(core::ptr::eq(receipt.component(), owner.component()));
        let attribute = element.attributes().next().unwrap();
        assert!(core::ptr::eq(attribute.component(), owner.component()));
        assert!(core::ptr::eq(attribute.element(), element.surface()));
        assert_eq!(
            block.span_of(attribute.surface().name.text),
            Some(Span::new(11, 16))
        );
        assert_eq!(owner.component().block().root_source(), source);
        assert_eq!(rendered(&owner), content);
        assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
    }

    #[test]
    fn ordinary_constructor_does_not_gain_lint_receipts() {
        let arena = Allocator::default();
        let block = SourceRoot::new("<MyComponent />").unwrap().whole_block();
        let bare = NativeLintComponent::parse_in(&arena, block).unwrap();
        let ordinary = NativeComponent::parse_in(&arena, block).unwrap();
        let before = ordinary.children().next().unwrap().into_element().unwrap();
        assert_eq!(
            before.lint_tag().err(),
            Some(NativeLintTagRefusal::Unavailable)
        );
        let lint = bare.children().next().unwrap().into_element().unwrap();
        assert_eq!(
            lint.lint_tag().unwrap().kind(),
            NativeLintTagKind::Component
        );
        let ordinary_again = NativeComponent::parse_in(&arena, block).unwrap();
        let after = ordinary_again
            .children()
            .next()
            .unwrap()
            .into_element()
            .unwrap();
        assert_eq!(
            after.lint_tag().err(),
            Some(NativeLintTagRefusal::Unavailable)
        );
    }

    #[test]
    fn bare_owner_preserves_modified_pre_refusal_and_original_literal_child() {
        let arena = Allocator::default();
        let source = "<div v-pre.camel><textarea>{{ value }}</textarea></div>";
        let owner =
            NativeLintComponent::parse_in(&arena, SourceRoot::new(source).unwrap().whole_block())
                .unwrap();
        let parent = owner.children().next().unwrap().into_element().unwrap();
        assert_eq!(
            parent.lint_tag().err(),
            Some(NativeLintTagRefusal::AmbiguousVerbatim)
        );
        let child = parent.children().next().unwrap().into_element().unwrap();
        assert_eq!(
            child.lint_tag().err(),
            Some(NativeLintTagRefusal::AmbiguousVerbatim)
        );
        assert!(child.surface().open.is_verbatim());
        assert!(matches!(
            child.children().next().unwrap().surface(),
            SurfaceChild::Text(_)
        ));
        assert_eq!(rendered(&owner), source);
    }

    #[test]
    fn bare_owner_keeps_holes_and_unexpected_source_without_success_credit() {
        let arena = Allocator::default();
        let source = "</stray><div>";
        let owner =
            NativeLintComponent::parse_in(&arena, SourceRoot::new(source).unwrap().whole_block())
                .unwrap();
        let mut children = owner.children();
        assert!(matches!(
            children.next().unwrap().surface(),
            SurfaceChild::Unexpected(_)
        ));
        let element = children.next().unwrap().into_element().unwrap();
        assert!(matches!(element.surface().close, ElementClose::Missing));
        assert_eq!(rendered(&owner), source);
        assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
    }

    #[test]
    fn bare_original_comments_keep_authored_order_and_source_identity() {
        let arena = Allocator::default();
        let source = "<!-- @vize:ignore-start --><div>内容</div><!-- @vize:ignore-end -->";
        let owner =
            NativeLintComponent::parse_in(&arena, SourceRoot::new(source).unwrap().whole_block())
                .unwrap();
        let component = owner.component();
        let children: Vec<_> = owner.children().collect();
        assert_eq!(children.len(), 3);
        for (ordinal, child) in children.iter().enumerate() {
            assert_eq!(child.ordinal(), ordinal);
            assert!(core::ptr::eq(child.component(), component));
        }
        for child in [&children[0], &children[2]] {
            assert!(matches!(child.surface(), SurfaceChild::Comment(_)));
            if let SurfaceChild::Comment(token) = child.surface() {
                assert!(component.block().span_of(token.text).is_some());
            }
        }
        assert_eq!(rendered(&owner), source);
    }

    #[test]
    fn nonempty_original_lexer_errors_are_preserved_without_admission() {
        let arena = Allocator::default();
        let source = "<div";
        let block = SourceRoot::new(source).unwrap().whole_block();
        let bare = NativeLintComponent::parse_in(&arena, block).unwrap();
        let ordinary = NativeComponent::parse_in(&arena, block).unwrap();
        let observations = |component: &NativeComponent<'_>| {
            component
                .carrier()
                .errors
                .iter()
                .map(|error| (error.code, error.offset))
                .collect::<Vec<_>>()
        };
        assert_eq!(
            observations(bare.component()),
            vec![(ErrorCode::EofInTag, 4)]
        );
        assert_eq!(observations(bare.component()), observations(&ordinary));
        assert_eq!(rendered(&bare), source);
        let element = bare.children().next().unwrap().into_element().unwrap();
        assert!(element.surface().open.gt.is_missing());
        assert_eq!(
            element.lint_tag().err(),
            Some(NativeLintTagRefusal::IncompleteHeader)
        );
    }

    #[test]
    fn original_unsupported_directive_record_and_source_are_preserved() {
        let arena = Allocator::default();
        let opens: String = (0..64)
            .map(|i| if i % 2 == 0 { '(' } else { '[' })
            .collect();
        let closes: String = opens
            .chars()
            .rev()
            .map(|c| if c == '(' { ')' } else { ']' })
            .collect();
        let head = cstr!("v-pre:[{opens}key{closes}]");
        let source = cstr!("<div {head}>original</div>");
        let block = SourceRoot::new(&source).unwrap().whole_block();
        let bare = NativeLintComponent::parse_in(&arena, block).unwrap();
        let ordinary = NativeComponent::parse_in(&arena, block).unwrap();
        let unsupported = &bare.component().carrier().unsupported;
        assert_eq!(unsupported.len(), 1);
        assert_eq!(unsupported[0].error, DirectiveNameError::NestingLimit);
        assert_eq!(unsupported[0].span, Span::new(5, 5 + head.len() as u32));
        assert_eq!(
            unsupported.as_slice(),
            ordinary.carrier().unsupported.as_slice()
        );
        assert_eq!(rendered(&bare), source);
        assert_eq!(check_fidelity(&bare.component().carrier().tree), Ok(()));
    }
}
