//! Whole original File/body association at genuine canonical binding events.

use vize_l0::{
    Allocator, Span,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{Vue, vue::DescriptorOptions},
    markup::NativeTemplateComponent,
};
use vize_l2::{
    lang::js::{NativeTemplateFile, NativeTemplateOwner},
    op::{BindingOp, Op},
};
use vize_l3::decision::{
    build_dom_decisions, build_dom_file_decisions,
    dom::{DomUnsupported, LiteralExpressions},
};

fn completed<'a>(arena: &'a Allocator, source: &'a str) -> NativeTemplateFile<'a> {
    let descriptor = Vue.observe_descriptor(
        arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let selected = NativeTemplateComponent::parse_in(arena, descriptor.admitted().unwrap())
        .unwrap()
        .unwrap();
    let mut original =
        NativeTemplateOwner::new(selected).unwrap_or_else(|_| panic!("original File owner"));
    {
        let mut walk = original.begin().unwrap();
        for child in walk.selected().children() {
            walk.child(child).unwrap();
        }
        walk.complete().unwrap();
    }
    core::hint::black_box(original.finish())
}

#[test]
fn sole_binding_walk_retains_same_file_on_body_source_and_complete_resolver() {
    let arena = Allocator::default();
    let source = "<template><button @click='/*original*/ var x=$event; return x;'/></template>";
    let original = completed(&arena, source);
    let file = original.view().unwrap().file().unwrap();
    let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
        panic!("element")
    };
    let [BindingOp::On(on)] = element.bindings.as_slice() else {
        panic!("actual On")
    };
    let lower = file.handler_for(on).unwrap();
    let analysis = build_dom_file_decisions(file).unwrap();
    let dom = analysis.dom().unwrap();
    let row = dom.file_handler(lower.id().node()).unwrap();
    assert!(core::ptr::eq(row.handler().file(), file));
    assert!(core::ptr::eq(row.on(), &**on));
    assert!(row.accepts_on(on));
    assert_eq!(row.handler().scope(), lower.scope());
    assert!(core::ptr::eq(row.resolution(), lower.resolution().unwrap()));
    assert!(core::ptr::eq(
        row.resolution().input().body(),
        lower.resolution().unwrap().input().body()
    ));
    assert_eq!(row.resolution().references().len(), 2);
    assert_eq!(row.resolution().declarations().len(), 1);
    let syntax = row.resolution().input().operand().syntax();
    assert!(core::ptr::eq(syntax.source().authored_root(), source));
    assert_eq!(
        syntax.source().text(),
        "/*original*/ var x=$event; return x;"
    );
    assert_eq!(syntax.comments().count(), 1);
    // A stock FunctionBody return is retained, but Vue's authored directive
    // grammar refuses it before any generated arrow can make it executable.
    let refusal = dom.unsupported().first().unwrap();
    assert_eq!(refusal.reason, DomUnsupported::HandlerSyntax);
    assert_eq!(refusal.node, lower.id().node());
    assert_eq!(refusal.span.slice(source), "return x;");
}

#[test]
fn comments_empty_input_and_entities_keep_original_complete_observation() {
    let arena = Allocator::default();
    for body in ["", "/*empty*/", "return $event &amp;&amp; 1;"] {
        let source = format!("<template><button @click='{body}'/></template>");
        let original = completed(&arena, &source);
        let file = original.view().unwrap().file().unwrap();
        let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
            panic!("element")
        };
        let [BindingOp::On(on)] = element.bindings.as_slice() else {
            panic!("On")
        };
        let lower = file.handler_for(on).unwrap();
        let analysis = build_dom_file_decisions(file).unwrap();
        let row = analysis
            .dom()
            .unwrap()
            .file_handler(lower.id().node())
            .unwrap();
        let syntax = row.resolution().input().operand().syntax();
        assert!(core::ptr::eq(
            syntax,
            lower.resolution().unwrap().input().operand().syntax()
        ));
        assert_eq!(row.resolution().input().operand().raw_value(), body);
        if body.contains("&amp;") {
            assert_eq!(syntax.source().text(), "return $event && 1;");
            let authored = syntax.source().authored_span(Span::new(14, 16)).unwrap();
            assert_eq!(authored.slice(&source), "&amp;&amp;");
        } else {
            assert!(row.resolution().references().is_empty());
        }
    }
}

#[test]
fn equal_ids_and_copied_on_do_not_replace_the_actual_joined_allocation() {
    let arena = Allocator::default();
    let source = "<template><button @click='return $event;'/></template>";
    let first = completed(&arena, source);
    let second = completed(&arena, source);
    let file = first.view().unwrap().file().unwrap();
    let foreign = second.view().unwrap().file().unwrap();
    let [Op::Element(first_element)] = file.artifact().root().ops.as_slice() else {
        panic!("first")
    };
    let [Op::Element(second_element)] = foreign.artifact().root().ops.as_slice() else {
        panic!("second")
    };
    let [BindingOp::On(on)] = first_element.bindings.as_slice() else {
        panic!("first On")
    };
    let [BindingOp::On(other)] = second_element.bindings.as_slice() else {
        panic!("second On")
    };
    let first_handler = file.handler_for(on).unwrap();
    let second_handler = foreign.handler_for(other).unwrap();
    assert_eq!(first_handler.id(), second_handler.id());
    let first_analysis = build_dom_file_decisions(file).unwrap();
    let second_analysis = build_dom_file_decisions(foreign).unwrap();
    let row = first_analysis
        .dom()
        .unwrap()
        .file_handler(first_handler.id().node())
        .unwrap();
    let other_row = second_analysis
        .dom()
        .unwrap()
        .file_handler(second_handler.id().node())
        .unwrap();
    assert!(row.accepts_on(on));
    assert!(!row.accepts_on(other));
    let copied = vize_l2::op::OnOp {
        name: on.name,
        modifiers: vize_l0::Vec::new_in(&&arena),
        handler: on.handler,
        span: on.span,
    };
    assert!(!row.accepts_on(&copied));
    assert!(!row.handler().same_owner(other_row.handler()));
}

#[test]
fn bare_artifact_context_has_no_original_file_handler_authority() {
    let arena = Allocator::default();
    let source = "<template><button @click='return $event;'/></template>";
    let original = completed(&arena, source);
    let file = original.view().unwrap().file().unwrap();
    let [Op::Element(element)] = file.artifact().root().ops.as_slice() else {
        panic!("element")
    };
    let [BindingOp::On(on)] = element.bindings.as_slice() else {
        panic!("On")
    };
    let id = file.handler_for(on).unwrap().id().node();
    let bare = build_dom_decisions(file.artifact(), &LiteralExpressions).unwrap();
    assert!(bare.dom().unwrap().file_handler(id).is_none());
    assert!(
        bare.dom()
            .unwrap()
            .unsupported()
            .iter()
            .any(|refusal| refusal.node == id)
    );
}
