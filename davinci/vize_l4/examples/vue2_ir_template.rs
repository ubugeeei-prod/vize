//! Actual retained-owner → L2 → shared L3 → Vue 2 L4 template capture.
//! The fixture host supplies explicit context/global identities. This is not an
//! ordinary-script binder, whole-SFC compiler, or product capability claim.

extern crate alloc;

use alloc::vec::Vec;
use std::{io::Write, time::Instant};
use vize_l0::Allocator;
use vize_l1::dialect::vue2::surface::parse_component;
use vize_l1_to_l2::native_vue2::lower_vue2_component;
use vize_l2::{
    artifact::Artifact,
    expr::ExprRef,
    resolution::{BindingId, BindingLookup, ResolutionTable, resolve_expression},
};
use vize_l3::decision::{build_dom_decisions, dom::ContextOnly};
use vize_l4::{
    expr::{
        ResolvedExpressions,
        vue::{Access, Binding},
    },
    targets::vue2::{Vue2Access, emit},
    write::Recorded,
};

mod vue2_ir_template_lexical;

const CONTEXT: &[&str] = &["a", "b", "日本", "Math"];
const SOURCES: &[&str] = &[
    "<div>{{ a }}</div>",
    "<div>Hello {{ a }} 世界</div>",
    "<div id='雪' class='x y'>{{ 日本 }}</div>",
    "<div>{{ ({a,b}) }}</div>",
    "<p>{{ Math.round(a / b) }}</p>",
    "<div>{{ a &gt; b ? a : b }}</div>",
    "<div><span>前</span>{{ a }}<strong>後</strong></div>",
    "<div>{{ a }}{{ b }}</div>",
    "<div>&lt;{{ 日本 }}&gt;</div>",
    "<input disabled value='雪'/>",
    "<div/>",
    "<div>{{ a.toString() }}</div>",
    "<div>{{ [a,b,日本] }}</div>",
    "<div class='x'/>",
    "<button type='button'>{{ a + b }}</button>",
    "<div>{{ 1 }}</div>",
    "<div id='&lt;&gt;&quot;&amp;&#39;'>{{ a }}</div>",
    "<div id='&amp;amp;'>{{ a }}</div>",
    "<div class=''>{{ a }}</div>",
];
struct Lookup;
impl BindingLookup for Lookup {
    fn lookup(&self, name: &str) -> Option<BindingId> {
        CONTEXT
            .iter()
            .position(|known| *known == name)
            .map(|index| BindingId::new(index as u32))
    }
}
fn tables<'a>(artifact: &Artifact<'a>, lookup: &impl BindingLookup) -> Vec<ResolutionTable<'a>> {
    let mut rows = Vec::new();
    artifact
        .visit_nodes(&mut |_, node| {
            node.for_each_expression(&mut |expression| {
                let ExprRef::Js(expression) = expression else {
                    panic!("retained JS")
                };
                rows.push(resolve_expression(expression, lookup).unwrap());
            });
        })
        .unwrap();
    rows.sort_by_key(|row| (row.expression().span.start, row.expression().span.end));
    rows
}
fn main() {
    let bindings = CONTEXT
        .iter()
        .enumerate()
        .map(|(index, name)| Binding {
            id: BindingId::new(index as u32),
            access: if *name == "Math" {
                Access::Global
            } else {
                Access::Context
            },
        })
        .collect::<Vec<_>>();
    let rows = SOURCES
        .iter()
        .map(|source| capture(source, &Lookup, &bindings))
        .collect::<Vec<_>>();
    let packet = serde_json::json!({ "schema": "vize.native-vue2-template-target.v1", "scope": "Explicit fixture context bindings; template backend only, no whole-SFC or product admission", "rows": rows, "bindingRows": vue2_ir_template_lexical::capture_rows() });
    let bytes = serde_json::to_vec_pretty(&packet).unwrap();
    std::io::stdout().write_all(&bytes).unwrap();
    std::io::stdout().write_all(b"\n").unwrap();
}

fn capture(
    source: &str,
    lookup: &impl BindingLookup,
    bindings: &[Binding<'_>],
) -> serde_json::Value {
    let ids = bindings
        .iter()
        .map(|binding| binding.id)
        .collect::<Vec<_>>();
    let access = Vue2Access::checked(bindings).unwrap();
    let allocator = Allocator::default();
    let start = Instant::now();
    let original = parse_component(&allocator, source)
        .unwrap()
        .into_expression_pool();
    let parsed_ns = start.elapsed().as_nanos();
    let originals = original
        .bindings()
        .iter()
        .filter_map(|binding| binding.admitted())
        .map(|binding| binding.base().expression().unwrap())
        .collect::<Vec<_>>();
    let start = Instant::now();
    let observation = lower_vue2_component(&allocator, original);
    let lowered_ns = start.elapsed().as_nanos();
    let artifact = observation.admitted().unwrap();
    let start = Instant::now();
    let tables = tables(artifact, lookup);
    let bound_ns = start.elapsed().as_nanos();
    let start = Instant::now();
    let analysis = build_dom_decisions(artifact, &ContextOnly::new(&tables, &ids)).unwrap();
    let analyzed_ns = start.elapsed().as_nanos();
    let start = Instant::now();
    let emitted = emit::<Recorded>(
        &analysis,
        ResolvedExpressions::checked(&tables).unwrap(),
        &access,
    )
    .unwrap()
    .finish();
    let emitted_ns = start.elapsed().as_nanos();
    assert_eq!(tables.len(), originals.len());
    for (table, original) in tables.iter().zip(originals) {
        assert!(core::ptr::eq(table.expression().ast, original));
    }
    let mut references = tables.iter().flat_map(|table| table.occurrences()).map(|occurrence| {
            let binding = bindings.iter().find(|binding| binding.id == occurrence.binding).unwrap();
            serde_json::json!({"name": occurrence.name, "role": match binding.access { Access::Local => "local", Access::Global => "global", _ => "context" }})
        }).collect::<Vec<_>>();
    references.sort_by(|left, right| left["name"].as_str().cmp(&right["name"].as_str()));
    references.dedup();
    let document = emitted.into_document();
    serde_json::json!({ "source": source, "code": document.as_str(), "map": document.source_map("template.vue", source), "links": document.links().iter().map(|link| serde_json::json!({ "generated": [link.generated.start,link.generated.end], "authored": [link.authored.start,link.authored.end], "name": link.name })).collect::<Vec<_>>(), "nodes": artifact.node_count(), "originalExpressions": tables.len(), "originalAstIdentitiesPreserved": true, "references": references, "phaseNs": { "parseAndRetain": parsed_ns, "lower": lowered_ns, "bind": bound_ns, "analyze": analyzed_ns, "emitWithLinks": emitted_ns } })
}
