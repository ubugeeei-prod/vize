use super::{Allocator, ForHeadInput, Lang, NativeForRefusal, observe};

#[test]
fn unsupported_native_families_preserve_the_whole_head_and_both_owners() {
    let arena = Allocator::default();
    for (text, kind) in [
        ("(item,key,index) in items", NativeForRefusal::AliasCount(3)),
        ("item in items.values", NativeForRefusal::CollectionShape),
        ("item in /*kept*/items", NativeForRefusal::Comment),
        ("item in _ctx", NativeForRefusal::RenderContextName),
        (r"\u0061 in items", NativeForRefusal::EscapedSpelling),
        ("item in it&#101;ms", NativeForRefusal::EntityOutput),
    ] {
        let syntax = observe(&arena, text, Lang::Js);
        assert_eq!(syntax.native_refusal(), Some(kind), "{text}");
        let aliases = syntax
            .aliases()
            .unwrap()
            .unwrap()
            .parameters()
            .unwrap()
            .as_ptr();
        let collection = syntax.collection().unwrap().unwrap().expression().unwrap() as *const _;
        let map = syntax
            .source()
            .decode_map()
            .map(|map| map.segments().as_ptr());
        let comments = syntax.comments().count();
        let rejected = ForHeadInput::new(syntax).unwrap_err();
        assert_eq!(rejected.kind, kind);
        assert_eq!(
            rejected
                .syntax()
                .aliases()
                .unwrap()
                .unwrap()
                .parameters()
                .unwrap()
                .as_ptr(),
            aliases
        );
        assert_eq!(
            rejected
                .syntax()
                .collection()
                .unwrap()
                .unwrap()
                .expression()
                .unwrap() as *const _,
            collection
        );
        assert_eq!(
            rejected
                .syntax()
                .source()
                .decode_map()
                .map(|map| map.segments().as_ptr()),
            map
        );
        assert_eq!(rejected.syntax().comments().count(), comments);
        assert_eq!(rejected.into_syntax().native_refusal(), Some(kind));
    }
}

#[test]
fn original_diagnostics_and_comments_are_retained_on_syntax_refusal() {
    let arena = Allocator::default();
    let text = "item in /x/uv /*kept*/";
    let syntax = observe(&arena, text, Lang::Js);
    let kind = syntax.native_refusal().unwrap();
    let before: alloc::vec::Vec<_> = syntax
        .diagnostics()
        .map(|(part, diagnostic)| {
            (
                part,
                diagnostic.message().as_ptr(),
                diagnostic.message().len(),
                diagnostic.severity(),
            )
        })
        .collect();
    assert!(!before.is_empty());
    let comments: alloc::vec::Vec<_> = syntax
        .comments()
        .map(|(part, comment)| {
            (
                part,
                comment.text().unwrap().as_ptr(),
                comment.text().unwrap().len(),
            )
        })
        .collect();
    let rejected = ForHeadInput::new(syntax).unwrap_err();
    assert_eq!(rejected.kind, kind);
    let after: alloc::vec::Vec<_> = rejected
        .syntax()
        .diagnostics()
        .map(|(part, diagnostic)| {
            (
                part,
                diagnostic.message().as_ptr(),
                diagnostic.message().len(),
                diagnostic.severity(),
            )
        })
        .collect();
    assert_eq!(after, before);
    let after: alloc::vec::Vec<_> = rejected
        .syntax()
        .comments()
        .map(|(part, comment)| {
            (
                part,
                comment.text().unwrap().as_ptr(),
                comment.text().unwrap().len(),
            )
        })
        .collect();
    assert_eq!(after, comments);
    assert_eq!(rejected.into_syntax().source().text(), text);
}
