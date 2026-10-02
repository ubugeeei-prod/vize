use alloc::vec::Vec;
use vize_l0::{Allocator, Span, id::NodeId};
use vize_l2::artifact::Artifact;
use vize_l2::expr::ExprRef;
use vize_l2::resolution::{BindingId, BindingLookup, ResolutionTable, resolve_expression};
use vize_l3::decision::{
    build_decisions, build_dom_decisions,
    dom::{ContextOnly, DomUnsupported},
    policy::TargetPolicy,
};

use crate::expr::{
    ResolvedExpressions,
    vue::{Access, AccessStyle, Binding, VueAccess},
};
use crate::module::{ModuleParts, RenderPlacement, RenderProperty, assemble, assemble_template};
use crate::runtime::{Runtime, vocabulary};
use crate::write::{NoLinks, Recorded};

use super::{DomErrorKind, emit};

mod fixture;
mod props;

const CONTEXT: &[&str] = &[
    "msg", "x", "y", "tip", "count", "offset", "classes", "styles",
];

struct Bindings;

impl BindingLookup for Bindings {
    fn lookup(&self, name: &str) -> Option<BindingId> {
        CONTEXT
            .iter()
            .position(|candidate| *candidate == name)
            .map(|index| BindingId::new(index as u32))
    }
}

fn resolutions<'a>(artifact: &Artifact<'a>) -> Vec<ResolutionTable<'a>> {
    let mut resolutions = Vec::new();
    artifact
        .visit_nodes(&mut |_, node| {
            node.for_each_expression(&mut |expression| {
                let ExprRef::Js(expression) = expression else {
                    panic!("reference must retain JS")
                };
                resolutions.push(resolve_expression(expression, &Bindings).unwrap());
            });
        })
        .unwrap();
    resolutions.sort_by_key(|table| (table.expression().span.start, table.expression().span.end));
    resolutions
}

fn bindings() -> Vec<Binding<'static>> {
    CONTEXT
        .iter()
        .enumerate()
        .map(|(index, _)| Binding {
            id: BindingId::new(index as u32),
            access: Access::Context,
        })
        .collect()
}

#[test]
fn complete_native_template_modules_equal_pinned_vue_for_thirteen_inputs() {
    let pack: serde_json::Value =
        serde_json::from_str(include_str!("../../../tests/fixtures/dom-vue-3.5.35.json")).unwrap();
    for reference in pack["fixtures"].as_array().unwrap() {
        let arena = Allocator::default();
        let source = reference["source"].as_str().unwrap();
        let id = reference["id"].as_str().unwrap();
        let artifact = fixture::build(&arena, id, source);
        let tables = resolutions(&artifact);
        let bindings = bindings();
        let ids: Vec<_> = bindings.iter().map(|binding| binding.id).collect();
        let context = ContextOnly::new(&tables, &ids);
        let analysis = build_dom_decisions(&artifact, &context).unwrap();
        let expressions = ResolvedExpressions::checked(&tables).unwrap();
        let access = VueAccess::checked(
            &bindings,
            AccessStyle::Function,
            vocabulary(Runtime::VueDom),
        )
        .unwrap();
        let recorded = assemble_template(
            emit::<Recorded>(&analysis, expressions, &access).unwrap(),
            vocabulary(Runtime::VueDom),
        )
        .unwrap();
        let plain = assemble_template(
            emit::<NoLinks>(&analysis, expressions, &access).unwrap(),
            vocabulary(Runtime::VueDom),
        )
        .unwrap();
        assert_eq!(
            recorded.text.as_str(),
            reference["code"].as_str().unwrap(),
            "{id}"
        );
        assert_eq!(
            plain.text, recorded.text,
            "{id}: recording must preserve every byte"
        );
        assert_eq!(
            plain.helpers, recorded.helpers,
            "{id}: helper order must be unchanged"
        );
        let document = recorded.into_document();
        for link in document.links() {
            assert!(
                document
                    .as_str()
                    .get(link.generated.start as usize..link.generated.end as usize)
                    .is_some()
            );
            assert!(
                source
                    .get(link.authored.start as usize..link.authored.end as usize)
                    .is_some()
            );
        }
        assert_eq!(analysis.artifact().source(), source);
        assert_eq!(
            analysis.tables().nodes.len(),
            artifact.node_count() as usize
        );
    }
}

#[test]
fn native_dom_module_links_follow_real_late_imports_and_named_accessor_rewrites() {
    let arena = Allocator::default();
    let source = "<p :title=\"tip\">{{count + offset}}</p>";
    let artifact = fixture::build(&arena, "dynamic-prop-text", source);
    let tables = resolutions(&artifact);
    let bindings = bindings();
    let ids: Vec<_> = bindings.iter().map(|binding| binding.id).collect();
    let context = ContextOnly::new(&tables, &ids);
    let analysis = build_dom_decisions(&artifact, &context).unwrap();
    let access = VueAccess::checked(
        &bindings,
        AccessStyle::Function,
        vocabulary(Runtime::VueDom),
    )
    .unwrap();
    let render = emit::<Recorded>(
        &analysis,
        ResolvedExpressions::checked(&tables).unwrap(),
        &access,
    )
    .unwrap();
    let mut parts = ModuleParts::for_runtime(Runtime::VueDom, "3.5.35", "__sfc__").unwrap();
    parts.render = Some(render);
    parts.placement = RenderPlacement::Function {
        binding: "render",
        property: RenderProperty::Render,
    };
    let document = assemble(parts).unwrap().into_document();
    assert!(document.as_str().starts_with("import { toDisplayString as _toDisplayString, openBlock as _openBlock, createElementBlock as _createElementBlock } from \"vue\"\n"));
    assert!(
        document
            .as_str()
            .ends_with("__sfc__.render = render\nexport default __sfc__\n")
    );
    for name in ["tip", "count", "offset"] {
        let link = document
            .links()
            .iter()
            .find(|link| {
                link.name
                    .as_ref()
                    .is_some_and(|actual| actual.as_str() == name)
            })
            .unwrap();
        assert_eq!(
            source.get(link.authored.start as usize..link.authored.end as usize),
            Some(name)
        );
        let generated = document
            .as_str()
            .get(link.generated.start as usize..link.generated.end as usize)
            .unwrap();
        assert_eq!(generated, vize_l0::cstr!("_ctx.{name}"));
    }
    let map: serde_json::Value =
        serde_json::from_str(&document.source_map("NativeDom.vue", source)).unwrap();
    assert_eq!(map["sources"][0], "NativeDom.vue");
    assert_eq!(map["sourcesContent"][0], source);
    assert_eq!(map["names"], serde_json::json!(["tip", "count", "offset"]));
    assert!(!map["mappings"].as_str().unwrap().is_empty());
    assert_eq!(analysis.artifact().node_count(), 3);
    assert!(analysis.tables().nodes.get(NodeId::FIRST).is_some());
}

#[test]
fn target_policy_and_unsupported_events_fail_with_actual_source_and_diagnostics_retained() {
    let arena = Allocator::default();
    let bindings = bindings();
    let access = VueAccess::checked(
        &bindings,
        AccessStyle::Function,
        vocabulary(Runtime::VueDom),
    )
    .unwrap();
    let empty = ResolvedExpressions::checked(&[]).unwrap();
    let artifact = fixture::build(&arena, "empty-element", "<div/>");
    let server = build_decisions(&artifact, TargetPolicy::Ssr).unwrap();
    assert_eq!(
        emit::<Recorded>(&server, empty, &access).unwrap_err().kind,
        DomErrorKind::WrongPolicy
    );
    let artifact = fixture::rejected_event(&arena);
    let original = artifact.provenance().to_vec();
    let analysis = build_decisions(&artifact, TargetPolicy::Dom).unwrap();
    let error = emit::<Recorded>(&analysis, empty, &access).unwrap_err();
    assert_eq!(
        error.kind,
        DomErrorKind::Unsupported(DomUnsupported::Binding)
    );
    assert_eq!(error.node.unwrap().index(), 1);
    assert_eq!(error.span, Span::new(8, 24));
    assert_eq!(
        artifact
            .source()
            .get(error.span.start as usize..error.span.end as usize),
        Some("@click=\"handler\"")
    );
    assert_eq!(analysis.artifact().provenance(), original);
    assert_eq!(analysis.artifact().source(), "<button @click=\"handler\"/>");
}

#[test]
fn missing_expression_facts_fail_without_reparsing_or_substituting_source() {
    let arena = Allocator::default();
    let artifact = fixture::build(&arena, "root-text-run", "hello {{msg}}");
    let tables = resolutions(&artifact);
    let bindings = bindings();
    let ids: Vec<_> = bindings.iter().map(|binding| binding.id).collect();
    let context = ContextOnly::new(&tables, &ids);
    let analysis = build_dom_decisions(&artifact, &context).unwrap();
    let access = VueAccess::checked(
        &bindings,
        AccessStyle::Function,
        vocabulary(Runtime::VueDom),
    )
    .unwrap();
    let error = emit::<Recorded>(
        &analysis,
        ResolvedExpressions::checked(&[]).unwrap(),
        &access,
    )
    .unwrap_err();
    let DomErrorKind::Expression(error) = error.kind else {
        panic!("missing native facts must be explicit")
    };
    assert_eq!(error.kind, crate::expr::EmitErrorKind::MissingResolution);
    assert_eq!(error.span, Span::new(8, 11));
    assert_eq!(analysis.artifact().source(), "hello {{msg}}");
    assert_eq!(tables.len(), 1);

    let foreign = fixture::build(&arena, "root-text-run", "hello {{msg}}");
    let foreign_tables = resolutions(&foreign);
    let error = emit::<Recorded>(
        &analysis,
        ResolvedExpressions::checked(&foreign_tables).unwrap(),
        &access,
    )
    .unwrap_err();
    let DomErrorKind::Expression(error) = error.kind else {
        panic!("a foreign retained AST must be refused")
    };
    assert_eq!(error.kind, crate::expr::EmitErrorKind::SourceMismatch);
    assert_eq!(error.span, Span::new(8, 11));
}
