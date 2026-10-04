//! Genuine original child/Doc joins; no synthetic invalid sealed views.

use super::*;
use crate::native_doc::NativeVue2SfcOptions;
use vize_l1::container::Vue;

#[test]
fn consumed_text_doc_refuses_foreign_same_bytes_and_different_original_occurrence() {
    let arena = Allocator::default();
    let source = std::string::String::from("<template>{{a}}{{a}}</template>");
    let copied = source.clone();
    assert_ne!(source.as_ptr(), copied.as_ptr());
    let options = NativeVue2SfcOptions::default();
    let descriptor = Vue.observe_vue2_descriptor(&arena, &source, options.descriptor);
    let component = descriptor.selected().unwrap().component();
    let expected = component.children().next().unwrap();
    let event = component
        .text_for(expected.reborrow())
        .unwrap()
        .binding()
        .span();
    let root = component.bindings()[0]
        .admitted()
        .unwrap()
        .base()
        .expression()
        .unwrap() as *const _;
    for foreign in [
        Vue.observe_vue2_descriptor(&arena, &source, options.descriptor),
        Vue.observe_vue2_descriptor(&arena, &copied, options.descriptor),
    ] {
        let child = foreign.component().unwrap().children().next().unwrap();
        let doc = vue2_text_document(
            foreign.component().unwrap().text_for(child).unwrap(),
            &arena,
        )
        .unwrap();
        let mut parts = Vec::new_in(&&arena);
        assert_eq!(
            append_text(&expected, event, doc, &mut parts, &arena, 0),
            Err(NativeVue2SfcRefusal::Frame {
                span: event,
                error: SourceFrameError::BlockNotRootSlice
            })
        );
        assert!(parts.is_empty());
    }
    let second = component.children().nth(1).unwrap();
    let doc = vue2_text_document(component.text_for(second).unwrap(), &arena).unwrap();
    let mut parts = Vec::new_in(&&arena);
    assert_eq!(
        append_text(&expected, event, doc, &mut parts, &arena, 0),
        Err(NativeVue2SfcRefusal::Frame {
            span: event,
            error: SourceFrameError::BlockNotRootSlice
        })
    );
    assert!(parts.is_empty());
    let doc = vue2_text_document(component.text_for(expected.reborrow()).unwrap(), &arena).unwrap();
    append_text(&expected, event, doc, &mut parts, &arena, 0).unwrap();
    assert_eq!(parts.len(), 1);
    assert_eq!(
        component.bindings()[0]
            .admitted()
            .unwrap()
            .base()
            .expression()
            .unwrap() as *const _,
        root
    );
}
