use crate::croquis::{
    SfcCroquisOptions, analyze_sfc_descriptor_with_context, analyze_sfc_descriptor_with_occurrences,
};
use crate::{SfcParseOptions, parse_sfc};
use vize_croquis::facts::{
    Bindings, CroquisFacts, Demand, FactConsumer, FactGroup, UnusedBindings,
};

struct Reader;
impl FactConsumer for Reader {
    const NAME: &'static str = "test/sibling-script-unused";
    const DEMAND: Demand = Demand::NONE.with(Bindings::ID).with(UnusedBindings::ID);
}

const SETUP: &str = "import { helper } from './helper'; const unused = 0;";

fn check(plain: &str, setup: &str, expected: &[&str]) {
    for newline in ["\n", "\r\n"] {
        for setup_first in [false, true] {
            let plain = format!("<script lang=\"ts\">{newline}{plain}{newline}</script>");
            let setup = format!("<script setup lang=\"ts\">{newline}{setup}{newline}</script>");
            let source = if setup_first {
                format!("{setup}{newline}{plain}")
            } else {
                format!("{plain}{newline}{setup}")
            };
            let descriptor = parse_sfc(&source, SfcParseOptions::default()).unwrap();
            let options = SfcCroquisOptions::lint_demand().with_unused_bindings();
            let ordinary = analyze_sfc_descriptor_with_context(&descriptor, None, options);
            assert_eq!(
                ordinary
                    .croquis
                    .unused_bindings
                    .iter()
                    .map(|name| name.as_str())
                    .collect::<Vec<_>>(),
                expected,
                "{source}"
            );
            let (captured, _) = analyze_sfc_descriptor_with_occurrences(&descriptor, None, options);
            assert_eq!(
                ordinary.croquis.to_vir(),
                captured.croquis.to_vir(),
                "{source}"
            );
            assert_eq!(
                serde_json::to_value(ordinary.croquis.semantic_snapshot()).unwrap(),
                serde_json::to_value(captured.croquis.semantic_snapshot()).unwrap()
            );
            let mut facts = CroquisFacts::new(&ordinary.croquis);
            let view = facts.prepare::<Reader>();
            let independent = vize_croquis::facts::spec::unused_bindings::evaluate(
                &descriptor,
                view.get::<Bindings>().unwrap(),
                &[],
                &ordinary.croquis,
            )
            .unwrap();
            assert_eq!(
                &independent,
                view.get::<UnusedBindings>().unwrap(),
                "all names and declaration ranges: {source}"
            );
        }
    }
}

#[test]
fn module_reads_use_original_lexical_import_identity() {
    for plain in [
        "export function describe(value: string) { return helper(value); }",
        "export const describe = () => helper('x');",
        "export const object = { helper };",
        "export const object = { [helper]: true };",
        "export type Helper = typeof helper;",
        "export type Helper = helper;",
        "export default { methods: { describe() { return helper('x'); } } };",
    ] {
        check(plain, SETUP, &["unused"]);
    }
    check(
        "export const describe = () => work('x');",
        "import { helper as work } from './helper'; const unused = 0;",
        &["unused"],
    );
    check(
        "export const describe = () => helper('x');",
        "import helper from './helper'; const unused = 0;",
        &["unused"],
    );
    check(
        "export const describe = () => helper.run();",
        "import * as helper from './helper'; const unused = 0;",
        &["unused"],
    );
}

#[test]
fn comments_properties_writes_and_shadows_do_not_read_the_setup_import() {
    for plain in [
        "export const text = 'helper'; // helper('x')",
        "export const object = { helper: true };",
        "export const object = { helper() { return 1; } };",
        "export const describe = () => object.helper();",
        "helper = replacement;",
        "export function describe(helper: string) { return helper; }",
        "export function describe() { const helper = () => 1; return helper(); }",
        "export function describe() { { const helper = 1; return helper; } }",
        "export type Helper<helper> = { value: helper };",
    ] {
        check(plain, SETUP, &["helper", "unused"]);
    }
    // Setup-local values never become module-level bindings.
    check(
        "export function describe() { return helper(); }",
        "const helper = () => 1; const unused = 0;",
        &["helper", "unused"],
    );
    check(
        "export type T = Helper;",
        "import type { Helper } from './helper'; const unused = 0;",
        &["unused"],
    );
    check(
        "export const describe = () => helper();",
        "import { helper as work } from './helper'; const unused = 0;",
        &["unused", "work"],
    );
}

#[test]
fn unknown_normal_blocks_do_not_prove_absence_and_default_demand_is_unchanged() {
    check(
        "export function describe() { return helper +; }",
        SETUP,
        &[],
    );
    check("eval('helper()');", SETUP, &[]);
    for block in [
        "<script src=\"./normal.ts\" />",
        "<script lang=\"coffee\">helper()</script>",
    ] {
        let source = format!("{block}<script setup>{SETUP}</script>");
        let descriptor = parse_sfc(&source, SfcParseOptions::default()).unwrap();
        let analysis = analyze_sfc_descriptor_with_context(
            &descriptor,
            None,
            SfcCroquisOptions::lint_demand().with_unused_bindings(),
        );
        assert!(analysis.croquis.unused_bindings.is_empty());
    }
    let source = format!(
        "<script>export const describe = () => helper()</script><script setup>{SETUP}</script>"
    );
    let descriptor = parse_sfc(&source, SfcParseOptions::default()).unwrap();
    let ordinary =
        analyze_sfc_descriptor_with_context(&descriptor, None, SfcCroquisOptions::lint_demand());
    assert!(ordinary.croquis.unused_bindings.is_empty());
}

#[test]
fn inline_javascript_and_typescript_share_the_same_import_join() {
    for language in ["", " lang=\"js\"", " lang=\"ts\""] {
        let source = format!(
            "<script{language}>export const describe = () => helper();</script><script setup{language}>{SETUP}</script>"
        );
        let descriptor = parse_sfc(&source, SfcParseOptions::default()).unwrap();
        let analysis = analyze_sfc_descriptor_with_context(
            &descriptor,
            None,
            SfcCroquisOptions::lint_demand().with_unused_bindings(),
        );
        assert_eq!(
            analysis
                .croquis
                .unused_bindings
                .iter()
                .map(|name| name.as_str())
                .collect::<Vec<_>>(),
            ["unused"]
        );
    }
}
