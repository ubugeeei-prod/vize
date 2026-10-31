//! The typed raw-child emitter owns array precedence independently of JSX parser spelling.
use vize_l0::{Allocator, Box, Span, Vec, side_table::SideTable};
use vize_l2::{
    artifact::{Artifact, ArtifactParts},
    expr::{ExprRef, JsExpr},
    op::{ElementOp, InterpolationOp, Namespace, Op, Region},
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
    let region = |ops| Region {
        ops: Vec::from_iter_in(ops, arena),
    };
    let child = Op::JsxChildValue(Box::new_in(
        InterpolationOp {
            expression: ExprRef::Js(js),
            span,
        },
        arena,
    ));
    let root = region([Op::Element(Box::new_in(
        ElementOp {
            tag: "main",
            namespace: Namespace::Html,
            attributes: Vec::new_in(arena),
            bindings: Vec::new_in(arena),
            children: region([child]),
            span: Span::new(0, source.len() as u32),
        },
        arena,
    ))]);
    let artifact = Artifact::try_new(ArtifactParts {
        source: &source,
        root,
        provenance: std::vec::Vec::new(),
        scopes: SideTable::new(),
    })
    .unwrap();
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
