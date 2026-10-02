//! Actual File/Program/Vue factories feed the sole-file DOM target.
#[path = "native_dom_file/support.rs"]
mod support;

use support::{construct, script};
use vize_l0::Allocator;
use vize_l2::{
    file::{Namespace, TemplateScope},
    lang::js::{FileProducer, ProgramInput, ProgramScope},
};
use vize_l3::decision::{build_dom_file_decisions, dom::DomUnsupported};
use vize_l4::module::assemble_template;
use vize_l4::runtime::{Runtime, vocabulary};
use vize_l4::targets::dom::{DomErrorKind, emit_file};
use vize_l4::write::{NoLinks, Recorded};

#[test]
fn complete_static_and_literal_modules_keep_real_file_owners_and_link_sinks() {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/native-dom-file-vue-3.5.35.json")).unwrap();
    let mut captured = Vec::new();
    for fixture in pack["fixtures"].as_array().unwrap() {
        let arena = Allocator::default();
        let source = fixture["source"].as_str().unwrap();
        let template = fixture["template"].as_str().unwrap();
        let mut producer = FileProducer::new(&arena, source).unwrap();
        let syntax = fixture["setup"]
            .as_str()
            .map(|content| script(&arena, source, content));
        if let Some((syntax, block)) = &syntax {
            producer
                .program(
                    ProgramInput::checked(syntax.admitted_program().unwrap(), *block, 0).unwrap(),
                    ProgramScope::Nested,
                )
                .unwrap();
        }
        let native = construct(
            &arena,
            source,
            template,
            &mut producer,
            if syntax.is_some() {
                TemplateScope::LastUnit
            } else {
                TemplateScope::Root
            },
        );
        let vue = producer.finish().unwrap();
        let analysis = build_dom_file_decisions(&vue).unwrap();
        assert!(core::ptr::eq(analysis.file(), &vue));
        assert!(core::ptr::eq(analysis.artifact(), vue.artifact()));
        assert!(analysis.dom().unwrap().unsupported().is_empty());
        let recorded = assemble_template(
            emit_file::<Recorded>(&analysis).unwrap(),
            vocabulary(Runtime::VueDom),
        )
        .unwrap();
        let plain = assemble_template(
            emit_file::<NoLinks>(&analysis).unwrap(),
            vocabulary(Runtime::VueDom),
        )
        .unwrap();
        assert_eq!(
            recorded.text.as_str(),
            fixture["code"].as_str().unwrap(),
            "{}",
            fixture["id"]
        );
        assert_eq!(plain.text, recorded.text);
        assert_eq!(plain.helpers, recorded.helpers);
        let document = recorded.into_document();
        for embed in &native.embeds {
            let node = embed.node.unwrap();
            let resolution = analysis
                .dom()
                .unwrap()
                .file_expression(node)
                .unwrap()
                .resolution();
            assert!(core::ptr::eq(resolution.file(), &vue));
            assert_eq!(resolution.node(), node);
            let scope = resolution.scope().unwrap();
            assert_eq!(vue.scopes().get(scope.index() as usize).unwrap().id, scope);
            let expression = resolution.table().unwrap().expression();
            assert!(core::ptr::eq(
                expression.ast,
                embed.syntax.expression().unwrap()
            ));
            assert!(resolution.table().unwrap().occurrences().is_empty());
            assert!(document.links().iter().any(|link| {
                link.authored == expression.span
                    && link.name.is_none()
                    && document
                        .as_str()
                        .get(link.generated.start as usize..link.generated.end as usize)
                        == Some(expression.source)
            }));
        }
        for link in document.links() {
            assert!(
                source
                    .get(link.authored.start as usize..link.authored.end as usize)
                    .is_some()
            );
            assert!(
                document
                    .as_str()
                    .get(link.generated.start as usize..link.generated.end as usize)
                    .is_some()
            );
        }
        let map: serde_json::Value =
            serde_json::from_str(&document.source_map("FileDom.vue", source)).unwrap();
        assert_eq!(map["sourcesContent"], serde_json::json!([source]));
        assert_eq!(map["names"], serde_json::json!([]));
        captured.push(serde_json::json!({
            "id": fixture["id"], "source": source,
            "code": document.as_str(), "map": map,
            "nodes": analysis.artifact().node_count(),
            "expressions": native.embeds.len(),
        }));
    }
    if let Ok(path) = std::env::var("VIZE_L4_FILE_DOM_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&captured).unwrap()).unwrap();
    }
}

#[test]
fn const_initializers_and_nested_unit_shadowing_never_authorize_context_access() {
    let arena = Allocator::default();
    let source = "<!--雪🌸--><script>const msg = 'outer';</script><script setup>const msg = 'inner';</script><template><p :title=\"'first'\">{{msg}}</p></template>";
    let (ordinary, ordinary_block) = script(&arena, source, "const msg = 'outer';");
    let (setup, setup_block) = script(&arena, source, "const msg = 'inner';");
    let mut producer = FileProducer::new(&arena, source).unwrap();
    producer
        .program(
            ProgramInput::checked(ordinary.admitted_program().unwrap(), ordinary_block, 0).unwrap(),
            ProgramScope::Module,
        )
        .unwrap();
    producer
        .program(
            ProgramInput::checked(setup.admitted_program().unwrap(), setup_block, 1).unwrap(),
            ProgramScope::Nested,
        )
        .unwrap();
    let native = construct(
        &arena,
        source,
        "<p :title=\"'first'\">{{msg}}</p>",
        &mut producer,
        TemplateScope::LastUnit,
    );
    let vue = producer.finish().unwrap();
    let analysis = build_dom_file_decisions(&vue).unwrap();
    assert!(analysis.dom().unwrap().unsupported().is_empty());
    let embed = native
        .embeds
        .iter()
        .find(|embed| embed.syntax.source().text() == "msg")
        .unwrap();
    let node = embed.node.unwrap();
    let row = analysis
        .dom()
        .unwrap()
        .file_expression(node)
        .unwrap()
        .resolution();
    assert_eq!(row.scope(), Some(vue.units().last().unwrap().scope));
    let use_site = row.table().unwrap().occurrences().first().unwrap();
    let declaration = row
        .binding(use_site.binding)
        .unwrap()
        .declaration()
        .unwrap();
    assert_eq!(declaration.scope, vue.units().last().unwrap().scope);
    assert_eq!(
        row.binding(use_site.binding).unwrap().id(),
        vue.lookup(vue.units().last().unwrap().scope, "msg", Namespace::Value)
            .unwrap()
            .id()
    );
    assert_ne!(
        use_site.binding,
        vue.lookup(vue.units().first().unwrap().scope, "msg", Namespace::Value)
            .unwrap()
            .id()
    );
    assert!(declaration.span.start >= setup_block.span().start);
    assert!(declaration.span.end <= setup_block.span().end);
    assert_eq!(
        source.get(declaration.span.start as usize..declaration.span.end as usize),
        Some("msg")
    );
    let recorded = emit_file::<Recorded>(&analysis).unwrap_err();
    let plain = emit_file::<NoLinks>(&analysis).unwrap_err();
    assert_eq!(recorded, plain);
    assert_eq!(recorded.kind, DomErrorKind::RuntimeAccessUnavailable);
    assert_eq!(recorded.node, Some(node));
    assert_eq!(recorded.span, row.table().unwrap().expression().span);
    assert_eq!(
        source.get(recorded.span.start as usize..recorded.span.end as usize),
        Some("msg")
    );
    assert_eq!(
        native.embeds.len(),
        2,
        "earlier literal output cannot escape as partial success"
    );
    assert_eq!(analysis.artifact().source(), source);
}

#[test]
fn zero_reference_nonliteral_keeps_the_real_l3_refusal_and_full_source() {
    let arena = Allocator::default();
    let source = "<p :title=\"[]\"/>";
    let mut producer = FileProducer::new(&arena, source).unwrap();
    let native = construct(&arena, source, source, &mut producer, TemplateScope::Root);
    let vue = producer.finish().unwrap();
    let analysis = build_dom_file_decisions(&vue).unwrap();
    let error = emit_file::<Recorded>(&analysis).unwrap_err();
    assert_eq!(
        error.kind,
        DomErrorKind::Unsupported(DomUnsupported::Expression)
    );
    assert_eq!(error.node, native.embeds.first().unwrap().node);
    assert_eq!(
        source.get(error.span.start as usize..error.span.end as usize),
        Some("[]")
    );
    assert_eq!(analysis.artifact().source(), source);
    assert_eq!(
        analysis.dom().unwrap().unsupported().first().unwrap().span,
        error.span
    );
}

#[test]
fn special_attribute_refusal_keeps_exact_binding_diagnostics_before_any_output() {
    let arena = Allocator::default();
    let source = "<p :key=\"'identity'\">text</p>";
    let mut producer = FileProducer::new(&arena, source).unwrap();
    let native = construct(&arena, source, source, &mut producer, TemplateScope::Root);
    let vue = producer.finish().unwrap();
    let analysis = build_dom_file_decisions(&vue).unwrap();
    let error = emit_file::<NoLinks>(&analysis).unwrap_err();
    assert_eq!(
        error.kind,
        DomErrorKind::Unsupported(DomUnsupported::BindingName)
    );
    assert_eq!(error.node, native.embeds.first().unwrap().node);
    assert_eq!(
        source.get(error.span.start as usize..error.span.end as usize),
        Some(":key=\"'identity'\"")
    );
    assert_eq!(analysis.artifact().source(), source);
}
