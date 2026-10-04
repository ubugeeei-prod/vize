use vize_l2::{
    lang::js::NativeSelectedSetup,
    op::{BindingOp, Op},
    resolution::{HandlerBindingRef, Usage},
};
use vize_l3::decision::ssr::{NativeSelectedSetupSsrAnalysis, SsrPart};

pub fn setup(
    setup: &NativeSelectedSetup<'_, '_>,
    analysis: &NativeSelectedSetupSsrAnalysis<'_, '_, '_>,
    handler: bool,
) -> serde_json::Value {
    let source = setup.file().artifact().source();
    let descriptor_source = setup.syntax().source().authored_root();
    assert!(core::ptr::eq(descriptor_source, source));
    assert!(core::ptr::eq(
        setup.program().source(),
        setup.source().source()
    ));
    assert_eq!(
        setup.source().span().slice(source),
        setup.program().source()
    );
    assert_eq!(setup.syntax().diagnostics().count(), 0);
    assert!(core::ptr::eq(
        setup.owner().retained_setup().unwrap(),
        setup.syntax()
    ));
    assert!(core::ptr::eq(
        setup.syntax().program().unwrap().body.as_ptr(),
        setup.program().program().body.as_ptr()
    ));
    assert!(setup.file().is_complete());
    assert!(setup.unit_record().interruption().is_none());
    assert_eq!(setup.unit_record().span, setup.source().span());
    let bindings: Vec<_> = setup.bindings().map(|binding| {
        assert!(core::ptr::eq(binding.file(), setup.file()));
        assert!(binding.same_owner(setup.binding(binding).unwrap()));
        let declaration = binding.declaration().unwrap();
        assert_eq!(declaration.unit, setup.unit());
        assert_eq!(declaration.scope, setup.scope());
        assert!(declaration.is_direct_program());
        serde_json::json!({"id":binding.id().index(),"name":declaration.name.as_str(),
            "kind":format!("{:?}",declaration.kind),"initializer":format!("{:?}",declaration.initializer),
            "span":{"start":declaration.span.start,"end":declaration.span.end}})
    }).collect();
    let annotations: Vec<_> = setup
        .type_annotations()
        .map(|annotation| {
            assert!(core::ptr::eq(annotation.file(), setup.file()));
            let span = annotation.span();
            assert!(setup.source().contains_block_span(span));
            serde_json::json!({"span":{"start":span.start,"end":span.end},
            "kind":format!("{:?}",annotation.kind())})
        })
        .collect();
    let interpolations: Vec<_> = analysis.ssr().unwrap().parts().iter().filter_map(|part| {
        let SsrPart::Interpolation { node, interpolation } = part else { return None; };
        let record = setup.file().native_interpolation(*node).unwrap();
        let operand = record.input().operand();
        assert_eq!(operand.full_span(), interpolation.span);
        assert_eq!(operand.content_span().slice(source), operand.raw_content());
        let syntax = operand.syntax();
        assert_eq!(syntax.diagnostics().count(), 0);
        assert!(core::ptr::eq(syntax.source().authored_root(), source));
        let row = analysis.expression(*node).unwrap();
        let resolution = row.resolution();
        assert!(core::ptr::eq(resolution.file(), setup.file()));
        assert_eq!(resolution.node(), *node);
        let region_scope = resolution.scope().unwrap();
        assert_eq!(setup.file().scopes()[region_scope.index() as usize].id, region_scope);
        assert_ne!(region_scope, setup.scope());
        let table = resolution.table().unwrap();
        assert_eq!(row.reads().len(), table.occurrences().len());
        let reads: Vec<_> = row.reads().iter().zip(table.occurrences()).map(|(read, occurrence)| {
            assert!(core::ptr::eq(read.occurrence(), occurrence));
            assert_eq!(occurrence.usage, Usage::Read);
            let binding = resolution.binding(occurrence.binding).unwrap();
            assert!(read.binding().same_owner(binding));
            assert_eq!(read.binding().id(), binding.id());
            assert!(resolution.accepts(binding));
            assert!(setup.binding(binding).is_ok());
            let declaration = binding.declaration().unwrap();
            assert_eq!(declaration.unit, setup.unit());
            assert_eq!(declaration.scope, setup.scope());
            assert_eq!(declaration.name.as_str(), occurrence.name);
            let authored = table.expression().authored_span(occurrence.span).unwrap();
            assert_eq!(authored.slice(source), occurrence.name);
            serde_json::json!({"name":occurrence.name,"span":{"start":authored.start,"end":authored.end},
                "decodedSpan":{"start":occurrence.span.start,"end":occurrence.span.end},
                "binding":binding.id().index(),"kind":format!("{:?}",read.kind()),
                "declarationScope":declaration.scope.index()})
        }).collect();
        Some(serde_json::json!({"node":node.index(),"span":{"start":operand.content_span().start,
            "end":operand.content_span().end},"raw":operand.raw_content(),"decoded":syntax.source().text(),
            "regionScope":region_scope.index(),"reads":reads}))
    }).collect();
    let handlers = if handler {
        let Some(Op::Element(element)) = setup.file().artifact().root().ops.first() else {
            panic!("original button")
        };
        let [BindingOp::On(on)] = element.bindings.as_slice() else {
            panic!("original On")
        };
        let handler = setup.file().handler_for(on).unwrap();
        assert!(handler.accepts_on(on));
        assert!(core::ptr::eq(handler.file(), setup.file()));
        assert_eq!(handler.scope(), Some(setup.scope()));
        let resolution = handler.resolution().unwrap();
        let input = resolution.input();
        let syntax = input.operand().syntax();
        assert_eq!(syntax.diagnostics().count(), 0);
        assert!(core::ptr::eq(syntax.source().authored_root(), source));
        let [reference] = resolution.references() else {
            panic!("original handler read")
        };
        let HandlerBindingRef::Outer(id) = reference.binding else {
            panic!("actual outer binding")
        };
        let binding = setup.file().binding(id).unwrap();
        assert!(handler.accepts(binding));
        assert!(setup.binding(binding).is_ok());
        vec![
            serde_json::json!({"raw":input.operand().raw_value(),"decoded":syntax.source().text(),
            "span":{"start":syntax.source().span().start,"end":syntax.source().span().end},
            "binding":id.index(),"name":binding.declaration().unwrap().name.as_str()}),
        ]
    } else {
        Vec::new()
    };
    let span = setup.source().span();
    serde_json::json!({"rawProgram":setup.program().source(),"sourceSpan":{"start":span.start,"end":span.end},
        "unit":setup.unit().index(),"scope":setup.scope().index(),"bindings":bindings,"annotations":annotations,
        "interpolations":interpolations,"handlers":handlers})
}
