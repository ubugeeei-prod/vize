//! The typed raw-child emitter owns array precedence independently of JSX parser spelling.
#![expect(clippy::disallowed_macros, reason = "fixture source uses format")]
#![expect(clippy::unreachable, reason = "test shape assertions panic")]

use vize_l0::{Allocator, Span, Vec};
use vize_l2::{
    artifact::{Artifact, Builder},
    expr::{ExprRef, JsExpr},
    op::{Namespace, Op},
    resolution::{BindingId, BindingLookup, resolve_expression},
};
use vize_l3::decision::{build_dom_decisions, dom::ContextOnly};
use vize_l4::{
    expr::{
        ResolvedExpressions,
        vue::{Access, AccessStyle, Binding, VueAccess},
    },
    module::assemble_template,
    runtime::{Runtime, vocabulary},
    targets::dom::emit,
    write::{NoLinks, Recorded},
};

struct State;
impl BindingLookup for State {
    fn lookup(&self, name: &str) -> Option<BindingId> {
        (name == "state").then(|| BindingId::new(0))
    }
}
#[test]
fn typed_sequence_child_preserves_the_real_expression_and_complete_emission_packet() {
    let owned = Allocator::new();
    let arena = &owned;
    let expression = "state.record(\"left\"),state.record(\"right\"),state.value";
    let source = format!("// 雪🌸\r\n{expression}");
    let span = Span::new(source.find(expression).unwrap() as u32, source.len() as u32);
    let js = JsExpr::parse_in(arena, expression, span).unwrap();
    let mut builder = Builder::new(arena, &source).unwrap();
    builder
        .element(
            "main",
            Namespace::Html,
            Vec::new_in(&arena),
            Span::new(0, source.len() as u32),
            |children, _| {
                children.interpolation(ExprRef::Js(js), span).unwrap();
            },
        )
        .unwrap();
    let mut parts = builder.finish().unwrap().into_parts();
    let mut roots = parts.root.ops.into_iter();
    let Op::Element(mut element) = roots.next().unwrap() else {
        unreachable!()
    };
    assert!(roots.next().is_none());
    let mut children =
        core::mem::replace(&mut element.children.ops, Vec::new_in(&arena)).into_iter();
    let Op::Interpolation(child) = children.next().unwrap() else {
        unreachable!()
    };
    assert!(children.next().is_none());
    element.children.ops = Vec::from_iter_in([Op::JsxChildValue(child)], &arena);
    parts.root.ops = Vec::from_iter_in([Op::Element(element)], &arena);
    let artifact = Artifact::try_new(parts).unwrap();
    let tables = [resolve_expression(js, &State).unwrap()];
    let ids = [BindingId::new(0)];
    let bindings = [Binding {
        id: ids[0],
        access: Access::Context,
    }];
    let context = ContextOnly::new(&tables, &ids);
    let analysis = build_dom_decisions(&artifact, &context).unwrap();
    let access = VueAccess::checked(
        &bindings,
        AccessStyle::Function,
        vocabulary(Runtime::VueDom),
    )
    .unwrap();
    let recorded = assemble_template(
        emit::<Recorded>(
            &analysis,
            ResolvedExpressions::checked(&tables).unwrap(),
            &access,
        )
        .unwrap(),
        vocabulary(Runtime::VueDom),
    )
    .unwrap();
    let plain = assemble_template(
        emit::<NoLinks>(
            &analysis,
            ResolvedExpressions::checked(&tables).unwrap(),
            &access,
        )
        .unwrap(),
        vocabulary(Runtime::VueDom),
    )
    .unwrap();
    assert_eq!(recorded.text, plain.text);
    assert_eq!(recorded.helpers, plain.helpers);
    let document = recorded.into_document();
    let map: serde_json::Value =
        serde_json::from_str(&document.source_map("RawChild.js", &source)).unwrap();
    assert_eq!(map["sourcesContent"], serde_json::json!([source]));
    assert!(
        document
            .links()
            .iter()
            .any(|link| link.authored.start == span.start)
    );
    let packet = serde_json::json!({"source":source,"expression":expression,"expressionSpan":[span.start,span.end],"code":document.as_str(),"map":map,"links":document.links().iter().map(|link|serde_json::json!({"authored":[link.authored.start,link.authored.end],"generated":[link.generated.start,link.generated.end]})).collect::<std::vec::Vec<_>>(),"scope":"native typed raw-child emitter array cardinality; no JSX parser or DOM runtime qualification"});
    if let Ok(path) = std::env::var("VIZE_JSX_RAW_IR_CAPTURE") {
        std::fs::write(path, serde_json::to_vec_pretty(&packet).unwrap()).unwrap();
    }
}
