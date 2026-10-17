//! One test owns the process-global profiler in this integration-test binary.
//! No parallel unit-test recorder can change the observed parse counts.
//!
//! The existing expression counter and legacy parse spans are the dynamic
//! witnesses. Native L1 goals have no equivalent counter: the narrow source
//! contract below additionally excludes parser/fallback entry points from the
//! demand-only collection and token walkers; it does not invent an L1 metric.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "fixture helpers assert retained parser ownership by panicking"
)]

use vize_armature::{
    Parser,
    parser::{TemplateSyntax, TemplateSyntaxFacts},
};
use vize_croquis::drawer::{TemplateReferenceOwner, collect_template_references};
use vize_l0::{Allocator, profiler::global_profiler};
use vize_relief::{ExpressionNode, PropNode, RootNode, TemplateChildNode};

const SOURCE: &str = concat!(
    "<Panel v-slot=\"{ slot = fallback }\" :value=\"&#36;listeners\" ",
    ":[key]=\"\\u0024listeners\">",
    "{{ /[/*]/.test($listeners) ? slot : outer }}",
    "<p v-for=\"($listeners, index) in list\" :title=\"$listeners\" ",
    "@click=\"const local = slot; consume(local, $event, &#36;listeners)\"/>",
    "<template slot-scope=\"{ local = fallback }\">",
    "{{ local }}{{ $listeners }}</template></Panel>",
);

// This is a bounded architectural contract, not a claim that textual matching
// can prove an arbitrary future call graph. These are the current complete
// collection/lexical leaves. A new delegated analysis module must join this list.
const COLLECTION_SOURCES: &[(&str, &str)] = &[
    (
        "packet",
        include_str!("../src/drawer/template_references.rs"),
    ),
    (
        "roles",
        include_str!("../src/drawer/template_references/roles.rs"),
    ),
    (
        "template walk",
        include_str!("../src/drawer/template_references/walk.rs"),
    ),
    (
        "token walk",
        include_str!("../src/drawer/helpers/identifiers/ast/walk/retained.rs"),
    ),
    (
        "token scopes",
        include_str!("../src/drawer/helpers/identifiers/ast/walk/retained_scopes.rs"),
    ),
    (
        "token targets",
        include_str!("../src/drawer/helpers/identifiers/ast/walk/retained_assignments.rs"),
    ),
];

fn assert_no_parser_or_fallback_entry_points() {
    for &(module, source) in COLLECTION_SOURCES {
        for forbidden in [
            "oxc_parser",
            "Parser::",
            "parse_once(",
            ".parse_expression(",
            ".parse_slot_parameters(",
            ".parse_v_for_parameters(",
            ".parse()",
            "parse_program_for_analysis(",
            "extract_identifiers_oxc(",
            "extract_identifier_refs_oxc(",
            "extract_identifier_refs_checked(",
            "extract_identifier_refs_with_witness(",
            "extract_identifiers_checked(",
        ] {
            assert!(
                !source.contains(forbidden),
                "{module} introduced a parser/fallback entry point: {forbidden}"
            );
        }
    }
}

fn counters() -> Vec<(&'static str, u64, u64)> {
    let mut counters: Vec<_> = global_profiler()
        .counter_snapshot()
        .into_iter()
        .map(|(name, metrics)| (name, metrics.samples, metrics.total))
        .collect();
    counters.sort_unstable_by_key(|entry| entry.0);
    counters
}

fn parse_spans() -> Vec<(&'static str, u64)> {
    let mut spans: Vec<_> = global_profiler()
        .all()
        .into_iter()
        .filter(|(name, _)| name.contains("parse"))
        .map(|(name, metrics)| (name, metrics.count))
        .collect();
    spans.sort_unstable_by_key(|entry| entry.0);
    spans
}

fn owners(facts: &TemplateSyntaxFacts<'_>, source: &str) -> Vec<(&'static str, *const ())> {
    let mut roots = Vec::new();
    for record in facts.records_for(source).expect("original source custody") {
        match record.syntax() {
            TemplateSyntax::Expression(Some(js)) => {
                roots.push(("expression", core::ptr::from_ref(js.ast).cast()));
            }
            TemplateSyntax::Slot(Some(js)) => {
                roots.push(("slot", core::ptr::from_ref(js.ast).cast()));
            }
            TemplateSyntax::For(owner) => {
                roots.push(("for owner", core::ptr::from_ref(owner.as_ref()).cast()));
                let collection = owner
                    .collection()
                    .expect("collection")
                    .expect("admitted collection");
                roots.push((
                    "collection",
                    core::ptr::from_ref(collection.expression().expect("original collection AST"))
                        .cast(),
                ));
                let aliases = owner.aliases().expect("aliases").expect("admitted aliases");
                let admitted = aliases
                    .admitted_parameters()
                    .expect("original alias admission");
                roots.push(("aliases", core::ptr::from_ref(admitted.parameters()).cast()));
            }
            TemplateSyntax::Handler(owner) => {
                roots.push(("handler owner", core::ptr::from_ref(owner.as_ref()).cast()));
                roots.push((
                    "handler body",
                    core::ptr::from_ref(owner.body().expect("original handler AST")).cast(),
                ));
            }
            TemplateSyntax::LegacySlot(owner) => {
                roots.push(("legacy owner", core::ptr::from_ref(owner.as_ref()).cast()));
                let admitted = owner
                    .admitted_parameters()
                    .expect("original legacy parameter admission");
                roots.push((
                    "legacy parameters",
                    core::ptr::from_ref(admitted.parameters()).cast(),
                ));
            }
            other => panic!("fixture must retain complete original syntax: {other:?}"),
        }
    }
    roots
}

fn assert_original_expression_and_slot_roots(root: &RootNode<'_>, facts: &TemplateSyntaxFacts<'_>) {
    let TemplateChildNode::Element(panel) = root.children.first().expect("original root child")
    else {
        panic!("original Panel")
    };
    for prop in &panel.props {
        let PropNode::Directive(directive) = prop else {
            panic!("original directive")
        };
        let Some(ExpressionNode::Simple(node)) = &directive.exp else {
            panic!("original value")
        };
        let retained = node.js_ast.expect("existing producer-owned retained root");
        let record = facts
            .syntax_for(
                root.source,
                oxc_span::Span::new(node.loc.span.start, node.loc.span.end),
                node.content,
            )
            .expect("same original goal");
        match record.syntax() {
            TemplateSyntax::Expression(Some(captured)) => {
                assert!(core::ptr::eq(
                    retained.as_expression().expect("expression").ast,
                    captured.ast
                ));
            }
            TemplateSyntax::Slot(Some(captured)) => {
                assert!(core::ptr::eq(retained.ast, captured.ast))
            }
            _ => panic!("Panel's values are expression/slot goals"),
        }
    }
}

#[test]
fn collection_keeps_original_owners_and_never_reenters_expression_parsing() {
    assert_no_parser_or_fallback_entry_points();
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let allocator = Allocator::new();
    let (root, errors, facts) = Parser::with_options(
        &allocator,
        SOURCE,
        vize_relief::options::ParserOptions {
            dialect: vize_l0::config::VueVersion::V2,
            ..Default::default()
        },
    )
    .parse_with_template_syntax_facts();
    profiler.disable();
    assert!(errors.is_empty(), "{errors:?}");
    assert!(core::ptr::eq(root.source, SOURCE));
    assert_eq!(root.source, SOURCE, "retain the complete original template");
    assert_original_expression_and_slot_roots(&root, &facts);
    let original_owners = owners(&facts, root.source);
    for role in [
        "expression",
        "slot",
        "for owner",
        "collection",
        "aliases",
        "handler owner",
        "handler body",
        "legacy owner",
        "legacy parameters",
    ] {
        assert!(
            original_owners.iter().any(|(name, _)| *name == role),
            "fixture must exercise {role}"
        );
    }
    let before_counters = counters();
    assert!(
        before_counters
            .iter()
            .any(|(name, samples, total)| *name == "davinci.expr.parses"
                && *samples > 0
                && *samples == *total),
        "original expression producer must be observed"
    );
    let before_parse_spans = parse_spans();

    profiler.enable();
    let packet = collect_template_references(&root, &facts);
    for _ in 0..8 {
        let repeated = collect_template_references(&root, &facts);
        assert_eq!(repeated.references(), packet.references());
        assert_eq!(repeated.gaps(), packet.gaps());
    }
    profiler.disable();
    assert!(packet.gaps().is_empty(), "{:?}", packet.gaps());
    assert_eq!(
        counters(),
        before_counters,
        "collection cannot increment the existing expression parse counter"
    );
    assert_eq!(
        parse_spans(),
        before_parse_spans,
        "collection cannot enter instrumented legacy expression/program parsers"
    );
    assert_eq!(
        owners(&facts, root.source),
        original_owners,
        "all roots remain the exact original arena allocations"
    );
    assert_original_expression_and_slot_roots(&root, &facts);

    for spelling in ["&#36;listeners", "\\u0024listeners"] {
        let start = SOURCE.find(spelling).expect("authored spelling") as u32;
        let reference = packet
            .references()
            .iter()
            .find(|reference| reference.span.start == start)
            .expect("complete original token");
        assert_eq!(reference.name.as_str(), "$listeners");
        assert_eq!(reference.span.end, start + spelling.len() as u32);
        assert_eq!(reference.owner, TemplateReferenceOwner::ComponentContext);
    }
    assert!(
        packet
            .references()
            .iter()
            .any(|reference| reference.name == "$listeners"
                && matches!(
                    reference.owner,
                    TemplateReferenceOwner::TemplateLocal {
                        declaration: Some(_)
                    }
                )),
        "loop alias is witnessed, not a component global"
    );
    assert!(
        packet
            .references()
            .iter()
            .any(|reference| reference.name == "$event"
                && reference.owner == TemplateReferenceOwner::TemplateLocal { declaration: None }),
        "the actual handler owns its implicit event parameter"
    );
    profiler.clear();
}
