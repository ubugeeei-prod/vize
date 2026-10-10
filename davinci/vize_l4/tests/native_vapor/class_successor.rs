//! Exact original static class succeeds through L3 but remains refused by Vapor L4.
use vize_l0::{Span, id::NodeId};
use vize_l2::{file::NativeFileAttributeValueState, lang::js::NativeTemplateFile, op::Op};
use vize_l3::decision::vapor::{
    VaporRejection, VaporUnsupported, build_native_vapor_file_decisions,
};
use vize_l4::{
    targets::vapor::{VaporError, VaporErrorKind, emit_template},
    write::{NoLinks, Recorded},
};

pub(crate) fn assert_current(owner: &NativeTemplateFile<'_>, source: &str) {
    assert_eq!(source, "<template><span class=\"x\">x</span></template>");
    let view = owner.view().unwrap();
    let file = view.file().unwrap();
    assert!(file.is_complete());
    let analysis = build_native_vapor_file_decisions(view).unwrap();
    assert!(core::ptr::eq(analysis.owner(), owner));
    assert!(core::ptr::eq(analysis.file(), file));
    assert!(core::ptr::eq(analysis.artifact(), file.artifact()));
    assert!(core::ptr::eq(analysis.artifact().source(), source));
    assert_eq!(analysis.artifact().node_count(), 2);
    let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
        panic!("complete original class root");
    };
    assert_eq!(element.tag, "span");
    assert_eq!(element.span, Span::new(10, 34));
    assert_eq!(element.attributes.len(), 1);
    let facts = analysis.original_attributes().unwrap();
    assert!(core::ptr::eq(facts.file(), file));
    assert_eq!(facts.len(), 1);
    let [record] = file.native_attribute_values() else {
        panic!("one whole original class observation");
    };
    assert_eq!(
        record.state(),
        NativeFileAttributeValueState::Attached {
            node: NodeId::FIRST,
            slot: 0
        }
    );
    let joined = facts.value(0, element, 0).unwrap();
    assert!(core::ptr::eq(joined.file(), file));
    let value = joined.observation().unwrap();
    assert!(core::ptr::eq(value, record.observation().unwrap()));
    let attribute = joined.attribute().unwrap();
    assert!(core::ptr::eq(attribute, &element.attributes[0]));
    assert_eq!(
        (attribute.name, attribute.value, attribute.span),
        ("class", Some("x"), Span::new(16, 25))
    );
    assert!(core::ptr::eq(
        attribute.value.unwrap(),
        value.source().text()
    ));
    assert!(core::ptr::eq(value.source().authored_root(), source));
    assert_eq!(value.name_span(), Span::new(16, 21));
    assert_eq!(value.equals_span(), Span::new(21, 22));
    assert_eq!(value.full_value_span(), Span::new(22, 25));
    assert_eq!(
        value.quote_spans(),
        Some((Span::new(22, 23), Span::new(24, 25)))
    );
    assert_eq!(value.value_span(), Span::new(23, 24));
    assert_eq!(value.source().span(), Span::new(23, 24));
    assert_eq!(value.raw_value(), "x");
    assert_eq!(value.source().text(), "x");
    assert!(value.source().decode_map().is_none());
    let vapor = analysis.vapor().unwrap();
    assert_eq!(vapor.roots().len(), 1);
    assert_eq!(vapor.roots()[0].node(), NodeId::FIRST);
    assert_eq!(vapor.roots()[0].span(), Span::new(10, 34));
    assert_eq!(vapor.parts(&vapor.roots()[0]).unwrap().len(), 3);
    assert_eq!(
        vapor.unsupported(),
        [VaporRejection {
            node: NodeId::FIRST,
            span: Span::new(16, 25),
            reason: VaporUnsupported::AttributeSemantics
        }]
    );
    let error = VaporError {
        node: Some(NodeId::FIRST),
        span: Span::new(16, 25),
        kind: VaporErrorKind::Unsupported(VaporUnsupported::AttributeSemantics),
    };
    assert_eq!(emit_template::<Recorded>(&analysis).unwrap_err(), error);
    assert_eq!(emit_template::<NoLinks>(&analysis).unwrap_err(), error);
    if let Ok(path) = std::env::var("VIZE_NATIVE_VAPOR_CAPTURE") {
        std::fs::write(format!("{path}.class-successor.json"), serde_json::to_vec_pretty(&serde_json::json!({
            "source":source,"file":format!("{file:#?}"),"tables":format!("{:#?}",analysis.tables()),
            "value":format!("{value:#?}"),"vapor":format!("{vapor:#?}"),"error":format!("{error:#?}")
        })).unwrap()).unwrap();
    }
}
