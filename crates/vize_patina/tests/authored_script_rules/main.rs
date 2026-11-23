//! Compare the complete configured rule result with the original standalone API.
#![expect(
    clippy::unwrap_used,
    reason = "fixture ranges and original responses are asserted"
)]

use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_l0::{Allocator, SourceBlock, SourceRoot};
use vize_l1::embed::authored_program::AuthoredProgram;
use vize_patina::rules::script::{
    NoReactiveDestructure, ScriptLintRefusal, ScriptLintResult, ScriptLinter,
};

mod profile;

fn check(linter: &ScriptLinter, block: SourceBlock<'_>) -> ScriptLintResult {
    let expected = linter.lint(block.source(), block.start() as usize);
    let arena = Allocator::default();
    let grammar = linter
        .authored_source_type()
        .unwrap_or_else(oxc_span::SourceType::mjs);
    let syntax = AuthoredProgram::parse(&arena, block, grammar);
    let actual = linter.lint_authored(syntax).unwrap();
    assert_same_result(&actual, &expected, block.source());
    actual
}

fn assert_same_result(actual: &ScriptLintResult, expected: &ScriptLintResult, source: &str) {
    assert_eq!(
        vize_l0::cstr!("{actual:?}"),
        vize_l0::cstr!("{expected:?}"),
        "{source}"
    );
}

fn scripts(source: &str, mut read: impl FnMut(SourceBlock<'_>)) -> bool {
    let Ok(sfc) = parse_sfc(source, SfcParseOptions::default()) else {
        return false;
    };
    let root = SourceRoot::new(source).unwrap();
    for block in [sfc.script.as_ref(), sfc.script_setup.as_ref()]
        .into_iter()
        .flatten()
    {
        let text = &source[block.loc.start..block.loc.end];
        assert_eq!(text, block.content.as_ref());
        read(root.block(text, block.loc.start as u32).unwrap());
    }
    true
}

#[test]
fn configured_rules_preserve_diagnostic_order_fixes_recovery_and_frame() {
    let mut bytes = ScriptLinter::new();
    bytes.add_rule(Box::new(NoReactiveDestructure));
    assert!(bytes.authored_source_type().is_none());
    assert!(ScriptLinter::new().authored_source_type().is_none());
    let all = ScriptLinter::with_all_rules();
    let vapor = ScriptLinter::with_vapor_rules();
    let clean = "import { nextTick } from 'vue'; nextTick(); const store = reactive({ x: 1 }); const { x } = store;";
    let broken = "const broken: = 1; const store = reactive({ x: 1 }); const { x } = store;";
    let source =
        format!("<!-- Ω🌱 -->\r\n<script>{clean}</script>\r\n<script setup>{broken}</script>");
    let mut results = Vec::new();
    assert!(scripts(&source, |block| {
        let byte_only = check(&bytes, block);
        let result = check(&all, block);
        let vapor = check(&vapor, block);
        assert!(!byte_only.diagnostics.is_empty());
        for diagnostic in &result.diagnostics {
            assert!(diagnostic.start >= block.start());
            assert!(diagnostic.end <= block.span().end);
        }
        results.push((
            result.diagnostics.len(),
            byte_only.diagnostics.len(),
            vapor.diagnostics.len(),
        ));
    }));
    assert!(results[0].0 > results[0].1 && results[0].2 > 0);
    assert_eq!(results[1].0, results[1].1);
    assert_eq!(results[1].2, 0);
}

#[test]
fn a_distinct_lint_grammar_is_optional_only_for_a_known_empty_result() {
    let vapor = ScriptLinter::with_vapor_rules();
    let mut mixed = ScriptLinter::with_vapor_rules();
    mixed.add_rule(Box::new(NoReactiveDestructure));
    let all = ScriptLinter::with_all_rules();
    for (source, empty) in [
        ("import x from './x'; const id = x;", true),
        ("const broken: = ;", true),
        ("export default defineComponent({})", false),
        ("import 'vue'; nextTick()", false),
        ("getCurrentInstance()", false),
        (r"ctx['$ne\x78tTick']()", false),
        ("const obj = {}; export default defineComponent(obj)", false),
    ] {
        let block = SourceRoot::new(source).unwrap().whole_block();
        let direct = vapor.authored_rules().empty_result_for(block);
        assert_eq!(direct.is_some(), empty, "{source}");
        if let Some(result) = direct {
            assert_same_result(&result, &vapor.lint(source, 0), source);
        }
        assert!(mixed.authored_rules().empty_result_for(block).is_none());
        assert!(all.authored_rules().empty_result_for(block).is_none());
    }
}

#[test]
fn ast_rules_require_the_original_standalone_grammar() {
    let lint = ScriptLinter::with_vapor_rules();
    let block = SourceRoot::new("nextTick()").unwrap().whole_block();
    let arena = Allocator::default();
    let syntax = AuthoredProgram::parse(&arena, block, oxc_span::SourceType::mjs());
    assert_eq!(
        lint.lint_authored(syntax).unwrap_err(),
        ScriptLintRefusal::DifferentGrammar
    );
}

fn for_originals(mut read: impl FnMut(usize, &str, &str)) -> usize {
    let inputs = std::fs::read(std::env::var_os("VIZE_SCRIPT_QUERY_INPUTS").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut invalid = 0;
    for (i, path) in inputs
        .split(|b| *b == 0)
        .filter(|p| !p.is_empty())
        .enumerate()
    {
        let name = std::str::from_utf8(path).unwrap();
        let bytes = std::fs::read(root.join(name)).unwrap();
        match std::str::from_utf8(&bytes) {
            Ok(source) => read(i, name, source),
            Err(_) => invalid += 1,
        }
    }
    invalid
}

#[test]
#[ignore = "original Vue source manifest"]
fn original_configured_rule_packets_match() {
    let all = ScriptLinter::with_all_rules();
    let vapor = ScriptLinter::with_vapor_rules();
    let mut empty_vapor_blocks = 0;
    let (mut files, mut descriptors, mut blocks, mut findings, mut fixes) = (0, 0, 0, 0, 0);
    let invalid = for_originals(|_, _, source| {
        files += 1;
        descriptors += usize::from(scripts(source, |block| {
            blocks += 1;
            if let Some(result) = vapor.authored_rules().empty_result_for(block) {
                let original = vapor.lint(block.source(), block.start() as usize);
                assert_same_result(&result, &original, block.source());
                empty_vapor_blocks += 1;
            }
            let result = check(&all, block);
            findings += result.diagnostics.len();
            fixes += result
                .diagnostics
                .iter()
                .filter(|d| d.fix.is_some())
                .count();
        }));
    });
    assert!(blocks > 0);
    eprintln!("authored_empty_vapor_blocks={empty_vapor_blocks}");
    eprintln!(
        "authored_script_lint files={files} descriptors={descriptors} blocks={blocks} diagnostics={findings} fixes={fixes} invalid={invalid}"
    );
}
