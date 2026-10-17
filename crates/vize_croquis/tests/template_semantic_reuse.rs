//! A single test owns the process-global profiler. Every source and complete
//! output is retained before comparison; metrics describe only existing probes.
//! Native L1 parser invocations and the classifier's internal retry total remain
//! unobserved. These receipts must never be summed into an all-parser census.

#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "fixture helpers retain complete failure receipts before assertions"
)]

use oxc_ast::ast::Expression;
use oxc_span::GetSpan;
use serde_json::{Value, json};
use vize_armature::{Parser, parser::TemplateSyntax};
use vize_carton::{Allocator, CompactString, cstr};
use vize_croquis::{
    Drawer, DrawerOptions, drawer::collect_template_semantic_reuse, sfc::parse_sfc,
};
use vize_l0::profiler::global_profiler;
use vize_relief::{ExpressionNode, PropNode, RootNode, SimpleExpressionNode, TemplateChildNode};

const SCRIPT: &str = "<script setup lang=\"ts\">const holder = { value: 1, $listeners: {} }; const items = [holder]; const consume = (...args: unknown[]) => args; const handler = (event: unknown) => consume(event);</script>";
const CONTROLS: &[(&str, &str)] = &[
    ("simple-handler", "<button @click=\"handler\"></button>"),
    (
        "object-spread",
        "<button @click=\"({ ...holder.$listeners })\"></button>",
    ),
    (
        "function-callback",
        "<button @click=\"function named(event) { return consume(event) }\"></button>",
    ),
    (
        "statement-body",
        "<button @click=\"const local = holder; consume(local, $event)\"></button>",
    ),
    ("simple-for", "<p v-for=\"item in items\">{{ item }}</p>"),
    (
        "destructured-for",
        "<p v-for=\"({ value }, index) in items\">{{ value }}</p>",
    ),
    (
        "comment-rewritten-for",
        "<p v-for=\"item in items /* keep */\">{{ item }}</p>",
    ),
    ("refused-handler", "<button @click=\"const =\"></button>"),
    (
        "trailing-comment-reference",
        "<button @click=\"handler // trailing\"></button>",
    ),
    (
        "trailing-comment-callback",
        "<button @click=\"function named(event) { return consume(event) } // trailing\"></button>",
    ),
    (
        "slot-bound-callback-for",
        "<Panel v-slot=\"{ rows }\"><p v-for=\"item in items.map(() => rows)\">{{ item }}</p></Panel>",
    ),
];

fn metrics() -> Value {
    let profiler = global_profiler();
    let mut counters = profiler.counter_snapshot();
    counters.sort_unstable_by_key(|entry| entry.0);
    let counters: Vec<_> = counters
        .into_iter()
        .map(|(name, metrics)| {
            json!({
                "name": name, "samples": metrics.samples, "total": metrics.total,
                "min": metrics.min, "max": metrics.max,
            })
        })
        .collect();
    let mut spans = profiler.span_snapshot();
    spans.sort_by_key(|(name, attribution, _)| (*name, cstr!("{attribution:?}")));
    let spans: Vec<_> = spans
        .into_iter()
        .map(|(name, attribution, metrics)| {
            json!({
                "name": name, "count": metrics.count,
                "attributionDebug": cstr!("{attribution:?}"),
                "metricsDebug": cstr!("{metrics:?}"),
            })
        })
        .collect();
    json!({"counters": counters, "spans": spans})
}

fn draw(source: &str, demanded: bool) -> Value {
    let descriptor = parse_sfc(source, Default::default()).expect("original whole SFC");
    let template = descriptor
        .template
        .as_ref()
        .map(|block| block.content.as_ref())
        .unwrap_or("");
    let allocator = Allocator::new();
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let (root, errors, reuse) = if demanded {
        let (root, errors, syntax) =
            Parser::new(&allocator, template).parse_with_template_syntax_facts();
        let reuse = collect_template_semantic_reuse(&root, &syntax);
        (root, errors, reuse)
    } else {
        let (root, errors) = Parser::new(&allocator, template).parse();
        (root, errors, None)
    };
    let mut drawer = Drawer::with_options(DrawerOptions::full()).with_binding_occurrences();
    if let Some(reuse) = reuse {
        drawer = drawer.with_template_semantic_reuse(reuse);
    }
    if let Some(script) = &descriptor.script {
        drawer.draw_script_plain_with_jsx(
            &script.content,
            matches!(script.lang.as_deref(), Some("jsx" | "tsx")),
        );
    }
    if let Some(setup) = &descriptor.script_setup {
        drawer.draw_script_setup_with_generic_and_jsx(
            &setup.content,
            setup.attrs.get("generic").map(|value| value.as_ref()),
            matches!(setup.lang.as_deref(), Some("jsx" | "tsx")),
        );
    }
    drawer.draw_template(&root);
    let (croquis, occurrences) = drawer.finish_with_binding_occurrences();
    profiler.disable();
    json!({
        "output": {
            "descriptor": descriptor,
            "parserErrorsDebug": cstr!("{errors:#?}"),
            "croquisDebug": cstr!("{croquis:#?}"),
            "semanticSnapshot": croquis.semantic_snapshot(),
            "editorOccurrencesDebug": cstr!("{occurrences:#?}"),
        },
        // NativeFor replaces the ordinary producer's generic BinaryIn Expr
        // with an authentic collection/aliases owner. Preserve the complete
        // producer input separately; it is not a Croquis semantic output.
        "producerRootDebug": cstr!("{root:#?}"),
        "observedMetrics": metrics(),
    })
}

fn for_value<'n, 'a>(root: &'n RootNode<'a>) -> &'n SimpleExpressionNode<'a> {
    let TemplateChildNode::Element(element) = root.children.first().expect("original element")
    else {
        panic!("element")
    };
    let PropNode::Directive(directive) = element.props.first().expect("original v-for") else {
        panic!("directive")
    };
    let Some(ExpressionNode::Simple(value)) = &directive.exp else {
        panic!("original value")
    };
    value
}

fn check_intended_for_producer_representation() {
    let source = "<p v-for=\"item in items\"></p>";
    let allocator = Allocator::new();
    let (ordinary, errors) = Parser::new(&allocator, source).parse();
    assert!(errors.is_empty());
    let ordinary = for_value(&ordinary)
        .js_ast
        .expect("ordinary generic Expr")
        .as_expression()
        .expect("expression role");
    assert!(
        matches!(ordinary.ast, Expression::BinaryExpression(_)),
        "ordinary For is coincidentally a complete generic BinaryIn expression"
    );
    let (root, errors, syntax) = Parser::new(&allocator, source).parse_with_template_syntax_facts();
    assert!(errors.is_empty());
    let value = for_value(&root);
    assert!(
        value.js_ast.is_none(),
        "demanded For uses its actual native owner instead of a generic Expr trial"
    );
    let record = syntax
        .syntax_for(
            root.source,
            oxc_span::Span::new(value.loc.span.start, value.loc.span.end),
            value.content,
        )
        .expect("original native For record");
    let TemplateSyntax::For(owner) = record.syntax() else {
        panic!("actual For owner")
    };
    let collection = owner
        .collection()
        .expect("collection")
        .expect("admitted collection");
    let ast = collection.expression().expect("original collection AST");
    assert!(matches!(ast, Expression::Identifier(identifier) if identifier.name == "items"));
    assert_eq!(
        collection
            .authored_span(ast.span())
            .expect("actual owner projects its own AST")
            .slice(source),
        "items"
    );
}

fn span_count(receipt: &Value, name: &str) -> u64 {
    receipt["observedMetrics"]["spans"]
        .as_array()
        .expect("complete span receipts")
        .iter()
        .filter(|entry| entry["name"] == name)
        .map(|entry| entry["count"].as_u64().expect("observed count"))
        .sum()
}

fn pair(id: &str, source: &str, group: &str) -> Value {
    json!({"id": id, "group": group, "source": source,
        "ordinary": draw(source, false), "demanded": draw(source, true)})
}

fn check_foreign_source_falls_back() {
    let source = "<button @click=\"const local = 1; consume(local)\"></button>";
    let allocator = Allocator::new();
    let (root, errors, syntax) = Parser::new(&allocator, source).parse_with_template_syntax_facts();
    assert!(errors.is_empty());
    let reuse = collect_template_semantic_reuse(&root, &syntax).expect("original private packet");
    let copy = CompactString::new(source);
    assert_ne!(
        copy.as_ptr(),
        source.as_ptr(),
        "equal bytes, foreign source buffer"
    );
    let (foreign, errors) = Parser::new(&allocator, &copy).parse();
    assert!(errors.is_empty());
    let profiler = global_profiler();
    profiler.clear();
    profiler.enable();
    let mut drawer =
        Drawer::with_options(DrawerOptions::full()).with_template_semantic_reuse(reuse);
    drawer.draw_template(&foreign);
    profiler.disable();
    assert_eq!(
        profiler
            .get("croquis.template.event.classify")
            .expect("foreign packet must not bypass classifier")
            .count,
        1
    );
}

#[test]
fn ordinary_and_demand_semantics_keep_complete_outputs_and_observed_parse_receipts() {
    let original: Value = serde_json::from_str(include_str!(
        "../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/independent.json"
    ))
    .expect("all original seventy records");
    let boundaries: Value = serde_json::from_str(include_str!("../../../tests/_fixtures/differential/lint/legacy-instance-apis-9217/template-boundaries.json")).expect("all forty-two boundary records");
    assert_eq!(
        original["records"]
            .as_array()
            .expect("original records")
            .len(),
        70
    );
    assert_eq!(
        boundaries["records"]
            .as_array()
            .expect("boundary records")
            .len(),
        42
    );
    let mut receipts = Vec::new();
    for (group, corpus) in [("original70", &original), ("boundaries42", &boundaries)] {
        for case in corpus["records"].as_array().expect("complete corpus") {
            receipts.push(pair(
                case["id"].as_str().expect("original id"),
                case["source"].as_str().expect("complete source"),
                group,
            ));
        }
    }
    for &(id, template) in CONTROLS {
        let source = cstr!("{SCRIPT}<template>{template}</template>");
        receipts.push(pair(id, &source, "controls"));
    }
    let evidence = json!({
        "schema": 1,
        "limits": "Existing probes only: davinci.expr.parses counts the original Armature Expr attempt; named classifier span counts invocations, not its TS/TSX retries. Native L1 goals have no parse census. Complete Debug, semantic facade and editor packet are retained unfiltered; Debug itself intentionally hides some private runtime fields.",
        "records": receipts,
    });
    let directory = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/differential/template-semantic-reuse");
    std::fs::create_dir_all(&directory).expect("retain complete source/output receipts");
    std::fs::write(
        directory.join("receipts.json"),
        serde_json::to_vec_pretty(&evidence).expect("all source/output/metric fields"),
    )
    .expect("preserve receipts before assertions");
    // pr-rust-shard retains this one test's successful stdout in its uploaded
    // JUnit artifact. Keep the full evidence there even without a separate raw
    // target/differential upload; do not replace it with a summary or digest.
    println!(
        "TEMPLATE_SEMANTIC_REUSE_EVIDENCE_BEGIN\n{}\nTEMPLATE_SEMANTIC_REUSE_EVIDENCE_END",
        serde_json::to_string_pretty(&evidence)
            .expect("complete evidence for retained JUnit stdout")
    );

    let records = evidence["records"]
        .as_array()
        .expect("all retained comparisons");
    assert_eq!(records.len(), 112 + CONTROLS.len());
    for receipt in records {
        assert_eq!(
            receipt["ordinary"]["output"], receipt["demanded"]["output"],
            "{} complete source retained in receipts.json",
            receipt["id"]
        );
    }
    for id in [
        "statement-body",
        "simple-for",
        "destructured-for",
        "slot-bound-callback-for",
    ] {
        let receipt = records
            .iter()
            .find(|receipt| receipt["group"] == "controls" && receipt["id"] == id)
            .expect("positive reuse control");
        let metric = if id == "statement-body" {
            "croquis.template.event.classify"
        } else {
            "croquis.helpers.identifiers.oxc_parse"
        };
        let ordinary = span_count(&receipt["ordinary"], metric);
        let demanded = span_count(&receipt["demanded"], metric);
        assert!(
            ordinary > demanded,
            "{id} must remove the intended observed call, {metric}: {ordinary} -> {demanded}"
        );
        println!(
            "{id}: observed {metric} {ordinary} -> {demanded}; native/classifier parser totals unknown"
        );
    }
    for id in [
        "simple-handler",
        "object-spread",
        "function-callback",
        "refused-handler",
        "trailing-comment-reference",
        "trailing-comment-callback",
        "comment-rewritten-for",
    ] {
        let receipt = records
            .iter()
            .find(|receipt| receipt["group"] == "controls" && receipt["id"] == id)
            .expect("fallback control");
        let metric = if id == "comment-rewritten-for" {
            "croquis.helpers.identifiers.oxc_parse"
        } else {
            "croquis.template.event.classify"
        };
        assert_eq!(
            span_count(&receipt["ordinary"], metric),
            span_count(&receipt["demanded"], metric),
            "{id} must preserve its unproved historical fallback"
        );
    }
    check_intended_for_producer_representation();
    check_foreign_source_falls_back();
    global_profiler().clear();
    println!(
        "complete ordinary/demand outputs: all 112 corpus records + {} controls; full receipts: {}",
        CONTROLS.len(),
        directory.join("receipts.json").display()
    );
}
