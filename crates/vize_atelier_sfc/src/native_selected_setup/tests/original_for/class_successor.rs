//! Named class+For source completes original custody, then refuses its body.
use super::*;
use vize_l0::Span;
use vize_l2::file::NativeFileAttributeValueState;
use vize_l3::decision::dom::{DomRejection, DomUnsupported};
use vize_l4::{
    targets::dom::{DomError, DomErrorKind, emit_selected_setup_template},
    write::{NoLinks, Recorded},
};

pub(super) fn assert_current(class: &crate::NativeSelectedSetupSfcDomCompilation<'_>) {
    let source = class.observation().original().descriptor().source();
    assert_eq!(
        source,
        "<script setup>let count=2</script><template><i class='fixed' v-for='item in count'>fixed</i></template>"
    );
    let error = DomError {
        node: Some(NodeId::from_index(1).unwrap()),
        span: Span::new(44, 92),
        kind: DomErrorKind::Unsupported(DomUnsupported::ForBody),
    };
    assert_eq!(
        class.result().err(),
        Some(NativeSelectedSetupSfcDomError::Dom(error))
    );
    let owner = class.observation().original().template().unwrap();
    let view = owner.view().unwrap();
    let admitted = class.observation().admitted().unwrap();
    let setup = admitted.setup();
    let file = setup.file();
    assert!(file.is_complete());
    assert!(core::ptr::eq(file, view.file().unwrap()));
    assert!(owner.retained_setup().is_some());
    assert!(core::ptr::eq(file.artifact().source(), source));
    assert_eq!(file.artifact().node_count(), 3);
    let [Op::OriginalFor(original)] = file.artifact().root().ops.as_slice() else {
        panic!("whole original For root");
    };
    assert_eq!(original.id().node(), NodeId::FIRST);
    assert_eq!(original.span, Span::new(44, 92));
    let head = file.for_head_for(original).unwrap();
    assert_eq!(
        head.resolution().unwrap().input().operand().raw_value(),
        "item in count"
    );
    let [Op::Element(element)] = original.region.ops.as_slice() else {
        panic!("whole original class body");
    };
    assert_eq!(element.tag, "i");
    assert_eq!(element.span, Span::new(44, 92));
    let [Op::Text(text)] = element.children.ops.as_slice() else {
        panic!("whole original body text");
    };
    assert_eq!(text.content, "fixed");
    let analysis = build_native_selected_setup_dom_decisions(setup).unwrap();
    assert!(core::ptr::eq(analysis.owner(), owner));
    assert!(core::ptr::eq(analysis.setup(), setup));
    assert!(core::ptr::eq(analysis.file(), file));
    let facts = analysis.original_attributes().unwrap();
    assert!(core::ptr::eq(facts.file(), file));
    assert_eq!(facts.len(), 1);
    let [record] = file.native_attribute_values() else {
        panic!("whole original class value");
    };
    assert_eq!(
        record.state(),
        NativeFileAttributeValueState::Attached {
            node: NodeId::from_index(1).unwrap(),
            slot: 0
        }
    );
    let joined = facts.value(0, element, 0).unwrap();
    let value = joined.observation().unwrap();
    assert!(core::ptr::eq(joined.file(), file));
    assert!(core::ptr::eq(value, record.observation().unwrap()));
    assert!(core::ptr::eq(value.source().authored_root(), source));
    assert_eq!(
        (value.raw_value(), value.source().text()),
        ("fixed", "fixed")
    );
    assert_eq!(value.name_span(), Span::new(47, 52));
    assert_eq!(value.equals_span(), Span::new(52, 53));
    assert_eq!(value.full_value_span(), Span::new(53, 60));
    assert_eq!(value.value_span(), Span::new(54, 59));
    assert_eq!(
        value.quote_spans(),
        Some((Span::new(53, 54), Span::new(59, 60)))
    );
    assert!(value.source().decode_map().is_none());
    let attribute = joined.attribute().unwrap();
    assert_eq!(
        (attribute.name, attribute.value, attribute.span),
        ("class", Some("fixed"), Span::new(47, 60))
    );
    assert!(core::ptr::eq(
        attribute.value.unwrap(),
        value.source().text()
    ));
    assert_eq!(
        analysis.dom().unwrap().unsupported(),
        [
            DomRejection {
                node: NodeId::from_index(1).unwrap(),
                span: Span::new(44, 92),
                reason: DomUnsupported::ForBody
            },
            DomRejection {
                node: NodeId::FIRST,
                span: Span::new(44, 92),
                reason: DomUnsupported::ForBody
            },
        ]
    );
    assert_eq!(
        emit_selected_setup_template::<Recorded>(&analysis).unwrap_err(),
        error
    );
    assert_eq!(
        emit_selected_setup_template::<NoLinks>(&analysis).unwrap_err(),
        error
    );
    if let Ok(path) = std::env::var("VIZE_NATIVE_SELECTED_STATIC_CLASS_CAPTURE") {
        let capture = serde_json::json!({"source":source,"file":format!("{file:#?}"),"tables":format!("{:#?}",analysis.tables()),"error":format!("{error:#?}")});
        std::fs::write(
            format!("{path}.for-refusal.json"),
            serde_json::to_vec_pretty(&capture).unwrap(),
        )
        .unwrap();
    }
}
