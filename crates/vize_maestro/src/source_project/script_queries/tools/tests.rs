use super::*;
use crate::source_project::{SnapshotRefusal, SourceSnapshotCache};
use tower_lsp::lsp_types::Url;
use vize_glyph::{EndOfLine, FormatOptions, VueVersion};
use vize_l0::Allocator;
use vize_patina::rules::script::NoReactiveDestructure;

// The original formatter, standalone rules and the already-composed editor and
// checker products are all called against the same resident document.
fn separate(
    snapshot: &SourceSnapshot,
    state: &ServerState,
    linter: &ScriptLinter,
    formatter: &GlyphFormatter<'_>,
) -> ScriptToolProducts {
    let scripts = snapshot
        .script_query_products_with_diagnostics(state)
        .unwrap()
        .map(|(editor, syntax_diagnostics)| {
            let desc = state
                .sfc_descriptor(snapshot.uri(), snapshot.source())
                .unwrap();
            let lint = |block: &vize_atelier_sfc::SfcScriptBlock<'_>| ScriptBlockLint {
                span: Span::new(block.loc.start as u32, block.loc.end as u32),
                result: linter.lint(&block.content, block.loc.start),
            };
            ScriptToolAnalysis {
                editor,
                syntax_diagnostics,
                script_lint: desc.script.as_ref().map(lint),
                setup_lint: desc.script_setup.as_ref().map(lint),
            }
        });
    let format = formatter.format(snapshot.source());
    snapshot.check_current(&state.documents).unwrap();
    ScriptToolProducts { format, scripts }
}

fn same(actual: &ScriptToolProducts, expected: &ScriptToolProducts, name: &str) {
    match (&actual.format, &expected.format) {
        (Ok(a), Ok(b)) => {
            assert_eq!(a.changed, b.changed, "{name}");
            assert_eq!(a.code.as_bytes(), b.code.as_bytes(), "{name}");
        }
        (Err(a), Err(b)) => {
            assert_eq!(
                std::mem::discriminant(a),
                std::mem::discriminant(b),
                "{name}"
            );
            assert_eq!(a.to_string(), b.to_string(), "{name}");
        }
        _ => panic!("formatter response differed on {name}"),
    }
    match (&actual.scripts, &expected.scripts) {
        (Some(a), Some(b)) => {
            assert_eq!(a.editor, b.editor, "{name}");
            assert_eq!(
                serde_json::to_vec(&a.syntax_diagnostics).unwrap(),
                serde_json::to_vec(&b.syntax_diagnostics).unwrap(),
                "{name}"
            );
            assert_eq!(
                format!("{:?}", a.script_lint),
                format!("{:?}", b.script_lint),
                "{name}"
            );
            assert_eq!(
                format!("{:?}", a.setup_lint),
                format!("{:?}", b.setup_lint),
                "{name}"
            );
        }
        (None, None) => {}
        _ => panic!("script descriptor acceptance differed on {name}"),
    }
}

#[test]
fn recovery_roles_and_configured_formatting_match_the_original_tools() {
    let state = ServerState::new();
    let uri = Url::parse("file:///editor/Composed.vue").unwrap();
    let cache = SourceSnapshotCache::default();
    let mut bytes = ScriptLinter::new();
    bytes.add_rule(Box::new(NoReactiveDestructure));
    let all = ScriptLinter::with_all_rules();
    let empty = ScriptLinter::new();
    let options = FormatOptions {
        single_quote: true,
        semi: false,
        end_of_line: EndOfLine::Auto,
        ..FormatOptions::default()
    };
    let allocator = Allocator::default();
    let sort = vize_glyph::ImportSortOptions::default();
    let formatter = GlyphFormatter::new_with_vue_version(&options, &allocator, VueVersion::V3)
        .with_sort_imports(Some(&sort))
        .with_template_whitespace_preserved(true);
    let mut diagnostics = 0;
    let mut absent = 0;
    let mut changed = 0;
    for source in [
        "\u{feff}<!-- 雪🌱 -->\r\n<script lang='ts'>import z from './z';import a from './a';export {z,a};</script>\r\n<script setup lang='ts'>import {getCurrentInstance} from 'vue';getCurrentInstance();const store=reactive({ x:1 });const {x}=store;</script>\r\n<template><div>  a  b  </div></template>",
        " \r\n<template lang='HTML'>\r\n<p>  雪🌱   a </p>\r\n</template>  \r\n",
        "  <template></template>  ",
        "<template><div :value='('>{{ ( }}</div></template>",
        "<!-- prologue --><template>  a <i/> b </template><!-- after -->",
        "<style scoped>p{color:red}</style><template>  a  </template>",
        "<script setup>const broken: = 1; const thing = reactive({x: 1});const {x}=thing;</script>",
        "<script lang='jsx'>export default <span/>;</script><script setup lang='tsx'>type Props = {value: string};const node=<div/>;</script>",
        "<script>const typed: string = 'x';</script><template><div/></template>",
        r#"<script>import { n\u0065xtTick as later, g\u0065tCurrentInstance as current } from 'vue';later();current();ctx['$ne\x78tTick']()</script>"#,
        "<script src='./entry.js'/><style>.a{color:red}</style>",
        "<script setup lang='tsx' generic='T'>import('./a');</script>",
        "<script>let first</script><script>let second</script>",
        "<script setup>const value='🌱';",
        "<template><span>🌱</span></template>",
        "",
    ] {
        state
            .documents
            .open(uri.clone(), source.into(), 1, "vue".into());
        let snapshot = cache.capture(&state.documents, &uri).unwrap();
        for lint in [&all, &bytes, &empty] {
            let actual = snapshot
                .script_tool_products(&state, lint, &formatter)
                .unwrap();
            same(
                &actual,
                &separate(&snapshot, &state, lint, &formatter),
                source,
            );
            if let Some(products) = actual.scripts {
                diagnostics += products.syntax_diagnostics.len()
                    + products
                        .script_lint
                        .into_iter()
                        .chain(products.setup_lint)
                        .map(|lint| lint.result.diagnostics.len())
                        .sum::<usize>();
            } else {
                absent += 1;
            }
            changed += usize::from(actual.format.is_ok_and(|r| r.changed));
        }
    }
    assert!(diagnostics > 0 && absent > 0 && changed > 0);
}

#[test]
fn missing_superseded_and_foreign_documents_keep_their_host_refusal() {
    let state = ServerState::new();
    let uri = Url::parse("file:///editor/Current.vue").unwrap();
    let cache = SourceSnapshotCache::default();
    let options = FormatOptions::default();
    let allocator = Allocator::default();
    let formatter = GlyphFormatter::new(&options, &allocator);
    let lint = ScriptLinter::with_vapor_rules();
    for (language, expected) in [
        ("typescript", Some(ScriptQueryRefusal::Language)),
        ("vue", None),
    ] {
        state
            .documents
            .open(uri.clone(), "<script/>".into(), 1, language.into());
        let snapshot = cache.capture(&state.documents, &uri).unwrap();
        assert_eq!(
            snapshot
                .script_tool_products(&state, &lint, &formatter)
                .err(),
            expected
        );
        state
            .documents
            .open(uri.clone(), "<template/>".into(), 1, language.into());
        assert_eq!(
            snapshot
                .script_tool_products(&state, &lint, &formatter)
                .unwrap_err(),
            ScriptQueryRefusal::Host(SnapshotRefusal::Superseded)
        );
        state.documents.close(&uri);
        assert_eq!(
            snapshot
                .script_tool_products(&state, &lint, &formatter)
                .unwrap_err(),
            ScriptQueryRefusal::Host(SnapshotRefusal::MissingDocument)
        );
    }
}

fn for_originals(mut read: impl FnMut(usize, &str, &str)) -> usize {
    let paths = std::fs::read(std::env::var_os("VIZE_SCRIPT_QUERY_INPUTS").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut invalid = 0;
    for (i, entry) in paths
        .split(|b| *b == 0)
        .filter(|s| !s.is_empty())
        .enumerate()
    {
        let name = std::str::from_utf8(entry).unwrap();
        let bytes = std::fs::read(root.join(name)).unwrap();
        match std::str::from_utf8(&bytes) {
            Ok(source) => read(i, name, source),
            Err(_) => invalid += 1,
        }
    }
    invalid
}

#[test]
#[ignore = "compare the four tools on original Vue files"]
fn complete_public_products_match_for_each_original_revision() {
    let state = ServerState::new();
    let uri = Url::parse("file:///editor/ToolCorpus.vue").unwrap();
    let cache = SourceSnapshotCache::default();
    let linter = ScriptLinter::with_all_rules();
    let options = FormatOptions::default();
    let (
        mut files,
        mut accepted,
        mut blocks,
        mut findings,
        mut syntax,
        mut bytes,
        mut format_errors,
    ) = (0, 0, 0, 0, 0, 0, 0);
    let invalid = for_originals(|_, name, source| {
        state
            .documents
            .open(uri.clone(), source.into(), 1, "vue".into());
        let snapshot = cache.capture(&state.documents, &uri).unwrap();
        let arena = Allocator::default();
        let formatter = GlyphFormatter::new(&options, &arena);
        let actual = snapshot
            .script_tool_products(&state, &linter, &formatter)
            .unwrap();
        same(
            &actual,
            &separate(&snapshot, &state, &linter, &formatter),
            name,
        );
        files += 1;
        if let Some(s) = actual.scripts {
            accepted += 1;
            syntax += s.syntax_diagnostics.len();
            for b in s.script_lint.into_iter().chain(s.setup_lint) {
                blocks += 1;
                findings += b.result.diagnostics.len();
            }
        }
        match actual.format {
            Ok(formatted) => bytes += formatted.code.len(),
            Err(_) => format_errors += 1,
        }
    });
    assert!(files > 0 && accepted > 0 && blocks > 0 && bytes > 0);
    eprintln!(
        "script_tools files={files} accepted={accepted} blocks={blocks} lint={findings} syntax={syntax} formatted_bytes={bytes} format_errors={format_errors} invalid={invalid}"
    );
}

#[cfg(feature = "micro-profiling")]
mod profile;

#[test]
fn a_semantic_lint_rule_receives_its_exact_filename_profile_after_borrowers() {
    use oxc_ast::ast::Program;
    use oxc_span::FileExtension;
    use vize_patina::rules::script::{ScriptLintResult, ScriptRule, ScriptRuleMeta};

    struct CheckFilename(std::sync::Arc<std::sync::atomic::AtomicUsize>);
    impl ScriptRule for CheckFilename {
        fn meta(&self) -> &'static ScriptRuleMeta {
            NoReactiveDestructure.meta()
        }
        fn uses_ast(&self) -> bool {
            true
        }
        fn check_program<'a>(
            &self,
            program: &'a Program<'a>,
            _: &str,
            _: usize,
            _: &mut ScriptLintResult,
        ) {
            assert_eq!(program.source_type.extension(), Some(FileExtension::Ts));
            self.0.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }
    }
    let counter = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let mut lint = ScriptLinter::new();
    lint.add_rule(Box::new(CheckFilename(counter.clone())));
    let state = ServerState::new();
    let uri = Url::parse("file:///editor/Filename.vue").unwrap();
    let options = FormatOptions::default();
    let arena = Allocator::default();
    let formatter = GlyphFormatter::new(&options, &arena);
    let src = "<script>const x = <string>value;</script><script setup lang='ts'>export const f = <T>(x:T)=>x;</script>";
    state
        .documents
        .open(uri.clone(), src.into(), 1, "vue".into());
    let snapshot = SourceSnapshotCache::default()
        .capture(&state.documents, &uri)
        .unwrap();
    let result = snapshot
        .script_tool_products(&state, &lint, &formatter)
        .unwrap();
    same(
        &result,
        &separate(&snapshot, &state, &lint, &formatter),
        src,
    );
    assert_eq!(counter.load(std::sync::atomic::Ordering::Relaxed), 4);
    let plain = ScriptDiagnosticProfile::for_language(Some("ts"));
    for path in ["x.mts", "x.cts", "x.d.ts", "x.tsx", "x.js"] {
        let offered = oxc_span::SourceType::from_path(path).unwrap();
        assert_eq!(
            plain.sharing_plain_typescript(Some(offered)),
            plain,
            "{path}"
        );
    }
}

#[test]
fn a_revision_replaced_during_lint_cannot_publish_any_tool_product() {
    use std::sync::Arc;
    use vize_patina::rules::script::{ScriptLintResult, ScriptRule, ScriptRuleMeta};

    struct Replace(Arc<ServerState>, Url);
    impl ScriptRule for Replace {
        fn meta(&self) -> &'static ScriptRuleMeta {
            NoReactiveDestructure.meta()
        }
        fn check(&self, _: &str, _: usize, _: &mut ScriptLintResult) {
            self.0
                .documents
                .open(self.1.clone(), "<template/>".into(), 1, "vue".into());
        }
    }
    let state = Arc::new(ServerState::new());
    let uri = Url::parse("file:///editor/Replaced.vue").unwrap();
    state.documents.open(
        uri.clone(),
        "<script>let original</script>".into(),
        1,
        "vue".into(),
    );
    let snapshot = SourceSnapshotCache::default()
        .capture(&state.documents, &uri)
        .unwrap();
    let mut lint = ScriptLinter::new();
    lint.add_rule(Box::new(Replace(Arc::clone(&state), uri)));
    let options = FormatOptions::default();
    let arena = Allocator::default();
    let formatter = GlyphFormatter::new(&options, &arena);
    assert_eq!(
        snapshot
            .script_tool_products(&state, &lint, &formatter)
            .unwrap_err(),
        ScriptQueryRefusal::Host(SnapshotRefusal::Superseded)
    );
}
