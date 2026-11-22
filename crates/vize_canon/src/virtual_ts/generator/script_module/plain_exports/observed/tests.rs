#![expect(
    clippy::unwrap_used,
    reason = "the comparison requires real original inputs and outputs"
)]
use super::super::super::{
    collect_named_value_export_starts, collect_normal_script_named_value_exports,
};
use super::*;
use crate::virtual_ts::{VizeSemanticLinkKind, generate_virtual_ts_with_offsets};
use davinci_harness::alloc::{self, CountingAllocator};
use std::alloc::System;
use vize_atelier_sfc::{
    SfcDescriptor,
    croquis::{SfcCroquisOptions, analyze_sfc_descriptor},
    parse_sfc,
};
use vize_carton::{Allocator, SourceRoot};
use vize_croquis::Croquis;

#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::system();

mod corpus;
mod profile;

fn frame<'a>(descriptor: &'a SfcDescriptor<'_>) -> SourceBlock<'a> {
    let script = descriptor.script.as_ref().unwrap();
    let root = SourceRoot::new(descriptor.source.as_ref()).unwrap();
    let text = &descriptor.source[script.loc.start..script.loc.end];
    root.block(text, script.loc.start.try_into().unwrap())
        .unwrap()
}

fn parses<'a>(
    arena: &'a Allocator,
    frame: SourceBlock<'a>,
) -> (AuthoredProgram<'a>, Option<AuthoredProgram<'a>>) {
    let ts = AuthoredProgram::parse(arena, frame, SourceType::ts().with_module(true));
    let retry = (ts.parsed().panicked || !ts.parsed().diagnostics.is_empty())
        .then(|| AuthoredProgram::parse(arena, frame, SourceType::tsx().with_module(true)));
    (ts, retry)
}

fn previous(block: SourceBlock<'_>) -> PlainScriptBridge<'_> {
    let script = block.source();
    let mut exports = collect_normal_script_named_value_exports(Some(script), false, true);
    let hoist = NamespaceHoistPlan::collect(Some(script), false, true);
    hoist.reconcile_exports(&mut exports);
    PlainScriptBridge {
        block,
        exports,
        hoist,
        keywords: collect_named_value_export_starts(script),
    }
}

fn compare_plans(actual: &PlainScriptBridge<'_>, old: &PlainScriptBridge<'_>, summary: &Croquis) {
    assert_eq!(actual.exports, old.exports);
    assert_eq!(actual.keywords, old.keywords);
    assert_eq!(
        actual.hoist.spans(),
        old.hoist.spans(),
        "{}",
        actual.block.source()
    );
    assert_eq!(actual.masked_script(), old.masked_script());
    let mut old_fields = vec![CompactString::new("previous")];
    let mut new_fields = old_fields.clone();
    old.append_setup_fields(&mut old_fields);
    actual.append_setup_fields(&mut new_fields);
    assert_eq!(new_fields, old_fields);
    let source = actual.block.source();
    let prefix = format!("/* previous projection */{source}\n");
    let start = prefix.len() - source.len() - 1;
    let initial = VizeMapping::new(
        start..start + source.len(),
        actual.block.start() as usize..actual.block.end() as usize,
    );
    let mut old_rows = vec![initial.clone()];
    let mut rows = vec![initial];
    assert_eq!(
        actual.project(summary, CompactString::from(prefix.as_str()), &mut rows),
        old.project(summary, CompactString::from(prefix), &mut old_rows)
    );
    assert_eq!(rows, old_rows);
}

/// Compare against the *published* document's text, SFC spans and navigation
/// links. Its original preceding generated ranges are the source of each edge.
fn public_footer(bridge: &PlainScriptBridge<'_>, summary: &Croquis) -> (usize, usize, usize) {
    let actual = generate_virtual_ts_with_offsets(
        summary,
        Some(bridge.block.source()),
        None,
        bridge.block.start(),
        0,
        &Default::default(),
    );
    let marker = "// Invoke setup to verify types\n";
    let start = actual.code.rfind(marker).unwrap() + marker.len();
    let mut original: Vec<_> = actual
        .mapping
        .spans()
        .iter()
        .filter(|r| r.gen_range.end <= start)
        .cloned()
        .collect();
    let prior = original.len();
    let result = bridge.project(
        summary,
        CompactString::new(&actual.code[..start]),
        &mut original,
    );
    let end = result.code.len();
    assert_eq!(
        actual.code.get(..end),
        Some(result.code.as_str()),
        "footer mismatch"
    );
    if !result.ambient.is_empty() {
        let prefix = &actual.code[..start];
        let begin = prefix
            .find("// Setup-scope bindings a hoisted namespace body reads\n")
            .unwrap();
        let end = begin + prefix[begin..].find("\n\n").unwrap() + 2;
        assert_eq!(
            &prefix[begin..end],
            result.ambient.as_str(),
            "published ambient section"
        );
    }
    let public_rows: Vec<_> = actual
        .mapping
        .spans()
        .iter()
        .filter(|r| start <= r.gen_range.start && r.gen_range.end <= end)
        .collect();
    assert!(
        original[prior..].iter().eq(public_rows),
        "published export mappings"
    );
    let public_links = actual
        .mapping
        .semantic_links()
        .iter()
        .filter(|link| link.kind == VizeSemanticLinkKind::VuePlainScriptExport);
    assert!(
        result.links.iter().eq(public_links),
        "published export navigation"
    );
    (
        end - start + result.ambient.len(),
        original.len() - prior,
        result.links.len(),
    )
}

#[test]
fn merged_export_names_shadowed_utility_types_and_document_ranges() {
    let source = r#"<!--📦-->
<script lang="ts">
export type ReturnType = number;
export const \u0061 = 1, { b: [value, , ...rest] } = bag;
export const {} = empty;
export namespace NS { export const result = value; }
/** public */
export class Box<T> { value!: T }
export enum Mode { A }
export enum Mode { B = 2 }
export function f(v: string): string;
export function f(v: string) { return v }
f.detail = 1;
export { forwarded } from 'other';
</script>"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let block = frame(&descriptor);
    let arena = Allocator::new();
    let (ts, tsx) = parses(&arena, block);
    let observed = PlainScriptBridge::new(&descriptor, &ts, tsx.as_ref()).unwrap();
    let summary = analyze_sfc_descriptor(&descriptor, None, SfcCroquisOptions::full());
    compare_plans(&observed, &previous(block), &summary);
    let mut expected_text = block.source().to_owned();
    for keyword in ["const", "enum", "function", "class"] {
        expected_text = expected_text.replace(
            &format!("\nexport {keyword}"),
            &format!("\n       {keyword}"),
        );
    }
    assert_eq!(observed.masked_script(), expected_text);
    let keywords: Vec<_> = observed
        .export_keywords()
        .map(|span| span.slice(source))
        .collect();
    assert_eq!(keywords, ["export"; 7]);
    let ranges: Vec<_> = observed
        .relocated_declarations()
        .map(|span| span.slice(source))
        .collect();
    assert_eq!(
        ranges,
        [
            "export namespace NS { export const result = value; }",
            "/** public */\nexport class Box<T> { value!: T }",
            "export enum Mode { A }",
            "export enum Mode { B = 2 }",
        ]
    );
    let (_, rows, links) = public_footer(&observed, &summary);
    assert!(rows >= 2 && links > 0);
}

#[test]
fn checker_grammar_retry_and_descriptor_identity_are_required() {
    let source = "<script lang='tsx'>export const View = () => <Panel />;</script>";
    let descriptor = parse_sfc(source, Default::default()).unwrap();
    let block = frame(&descriptor);
    let arena = Allocator::new();
    let (ts, retry) = parses(&arena, block);
    let retry = retry.unwrap();
    assert!(matches!(
        PlainScriptBridge::new(&descriptor, &ts, None),
        Err(PlainScriptBridgeError::RetryRequired)
    ));
    assert!(matches!(
        PlainScriptBridge::new(&descriptor, &retry, None),
        Err(PlainScriptBridgeError::Grammar)
    ));
    let owned = descriptor.clone().into_owned();
    assert!(matches!(
        PlainScriptBridge::new(&owned, &ts, Some(&retry)),
        Err(PlainScriptBridgeError::Source)
    ));
    let (_, foreign) = parses(&arena, frame(&owned));
    assert!(matches!(
        PlainScriptBridge::new(&descriptor, &ts, foreign.as_ref()),
        Err(PlainScriptBridgeError::Source)
    ));
    let observed = PlainScriptBridge::new(&descriptor, &ts, Some(&retry)).unwrap();
    let summary = analyze_sfc_descriptor(&descriptor, None, SfcCroquisOptions::full());
    compare_plans(&observed, &previous(block), &summary);
    public_footer(&observed, &summary);
}

#[test]
fn selected_plain_block_and_parser_diagnostics_bound_the_query() {
    for (source, expected) in [
        (
            "<script>export const bad=</script>",
            PlainScriptBridgeError::Syntax(DeclarationSyntaxError::Panicked),
        ),
        (
            "<script src='./other'/>",
            PlainScriptBridgeError::ExternalScript,
        ),
        (
            "<script>const x=1</script><script setup>const y=1</script>",
            PlainScriptBridgeError::Selection,
        ),
        (
            "<script lang='coffee'>foo</script>",
            PlainScriptBridgeError::Grammar,
        ),
    ] {
        let descriptor = parse_sfc(source, Default::default()).unwrap();
        let arena = Allocator::new();
        let (ts, retry) = parses(&arena, frame(&descriptor));
        assert_eq!(
            PlainScriptBridge::new(&descriptor, &ts, retry.as_ref()).unwrap_err(),
            expected
        );
    }
}

#[test]
fn owned_projection_outlives_both_original_arenas() {
    let source = "<script lang='tsx'>export const identity = <T>(v: T) => v;</script>";
    let descriptor = parse_sfc(source, Default::default()).unwrap().into_owned();
    let block = frame(&descriptor);
    let plan = {
        let enabled = Allocator::new();
        let editor = Allocator::new();
        let ts = AuthoredProgram::parse(&enabled, block, SourceType::ts().with_module(true));
        let tsx = AuthoredProgram::parse(&editor, block, SourceType::tsx().with_module(true));
        PlainScriptBridge::new(&descriptor, &ts, Some(&tsx)).unwrap()
    };
    assert_eq!(
        plan.masked_script(),
        "       const identity = <T>(v: T) => v;"
    );
    let summary = analyze_sfc_descriptor(&descriptor, None, SfcCroquisOptions::full());
    compare_plans(&plan, &previous(block), &summary);
    assert_eq!(public_footer(&plan, &summary).2, 1);
}
