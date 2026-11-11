//! A retained rewrite parse must answer exactly what a fresh parse of the
//! same bytes answers, and nothing for any other bytes.

use oxc_span::SourceType;
use vize_carton::{String, cstr};

use super::super::collect::collect_specifier_spans;
use super::super::{ImportRewriter, RewriteSyntaxMode, SyntaxRewrite};
use crate::PackageResolutionMode;

/// Module literals in every syntactic position the rewriter visits, beside
/// comments, strings and templates that only look like imports.
const SOURCES: &[&str] = &[
    "import Panel from './Panel.vue';\nexport * from './Panel.vue';\nexport { a } from './a';\n",
    "import type { Props } from './Card.vue';\nexport type * from './Types.vue';\nexport * as ns from './Ns.vue';\n",
    "const lazy = () => import('./Lazy.vue');\nconst attrs = import('./Attrs.vue', { with: { 'resolution-mode': 'require' } });\n",
    "type Helper = import(\"vize:virtual-ts\").__VizeHelper;\ntype Ctx = typeof import('./Ctx.vue')['default'];\nexport {};\n",
    "// import x from './Comment.vue'\n/* export * from './Block.vue' */\nconst text = \"import y from './String.vue'\";\nconst tpl = `export * from './Template.vue'`;\nimport real from './Real.vue';\n",
    "import Query from './Query.vue?vue&type=script';\nimport Raw from './Raw.vue?raw';\nexport * from './Star.vue?inline';\n",
    "import x = require('./Required.vue');\nconst y = require('./Call.vue');\ndeclare module './Augmented.vue' { export const z: 1 }\n",
    "import { e } from './Esc\\u0061ped.vue';\nexport * from \"./Double.vue\";\nimport w from './Attr.vue' with { 'resolution-mode': 'import' };\n",
    "export * from './First.vue';\nexport * from './First.vue';\nimport again from './First.vue';\n",
    // A recoverable diagnostic: literals stay, stars are not captured.
    "import a from './A.vue';\nexport * from './B.vue';\nconst mixed = a ?? b || c;\n",
    // Fatal errors after valid imports: the line-prefix recovery's literals.
    "import a from './A.vue';\nexport * from './B.vue';\nlet = ;\n",
    "import a from './A.vue';\nexport * from './B.vue';\nconst x = {\n  import('./C.vue')\n",
    "",
];

fn mirror(path: &str, _mode: PackageResolutionMode) -> Option<String> {
    let (base, query) = path.split_once('?').unwrap_or((path, ""));
    base.ends_with(".vue").then(|| {
        if query.is_empty() {
            cstr!("{base}.ts")
        } else {
            cstr!("{base}.ts?{query}")
        }
    })
}

fn retained(source: &str, source_type: SourceType) -> SyntaxRewrite {
    ImportRewriter::new().rewrite_with_syntax(
        source,
        source_type,
        mirror,
        RewriteSyntaxMode::Retain,
    )
}

fn assert_same_rewrite(source: &str, expected: &SyntaxRewrite, actual: &SyntaxRewrite) {
    assert_eq!(expected.result.code, actual.result.code, "{source}");
    assert_eq!(
        expected.result.source_map, actual.result.source_map,
        "{source}"
    );
    assert_eq!(
        expected
            .result
            .source_map
            .pre_rewrite_text(&expected.result.code),
        actual
            .result
            .source_map
            .pre_rewrite_text(&actual.result.code),
        "{source}"
    );
    assert_eq!(
        cstr!("{:?}", expected.stars),
        cstr!("{:?}", actual.stars),
        "{source}"
    );
}

#[test]
fn a_replayed_parse_rewrites_exactly_as_a_fresh_parse() {
    let rewriter = ImportRewriter::new();
    for source_type in [SourceType::ts(), SourceType::tsx()] {
        for source in SOURCES {
            let fresh = rewriter.rewrite_with_syntax(
                source,
                source_type,
                mirror,
                RewriteSyntaxMode::Reuse(None),
            );
            let (legacy, legacy_stars, legacy_spans) =
                rewriter.rewrite_with_star_capture(source, source_type, mirror, true);
            assert_eq!(fresh.result.code, legacy.code, "{source}");
            assert_eq!(fresh.result.source_map, legacy.source_map, "{source}");
            assert_eq!(
                cstr!("{:?}", fresh.stars),
                cstr!("{legacy_stars:?}"),
                "{source}"
            );

            let first = retained(source, source_type);
            assert_same_rewrite(source, &fresh, &first);
            let syntax = first.syntax.as_ref().expect("a retained parse");
            assert_eq!(syntax.first_occurrences(source), legacy_spans, "{source}");
            let replayed = rewriter.rewrite_with_syntax(
                source,
                source_type,
                mirror,
                RewriteSyntaxMode::Reuse(Some(syntax)),
            );
            assert_same_rewrite(source, &fresh, &replayed);
            assert!(replayed.syntax.is_none());
        }
    }
}

#[test]
fn a_replayed_parse_reports_the_dependency_occurrences_of_a_fresh_parse() {
    let rewriter = ImportRewriter::new();
    for source in SOURCES {
        let rewrite = retained(source, SourceType::ts());
        let syntax = rewrite.syntax.as_ref().unwrap();
        assert_eq!(
            syntax.dependency_occurrences(source),
            rewriter.collect_all_specifier_occurrences(source, SourceType::ts()),
            "{source}"
        );
    }
}

#[test]
fn replaced_literals_name_the_specifiers_of_the_rewritten_text() {
    // Plain, unchanged, sentinel-suffixed and escape-requiring replacements.
    let policy = |path: &str, _mode: PackageResolutionMode| -> Option<String> {
        match path {
            "./Panel.vue" => Some("./Panel.vue.ts".into()),
            "./Same.vue" => Some("./Same.vue".into()),
            "./Alias.vue.ts" => Some(cstr!(
                "./Alias.vue.ts{}",
                crate::batch::AUTHORED_VUE_TS_ALIAS_SENTINEL
            )),
            "./Quote.vue" => Some("./Qu'o\"te\\.vue.ts".into()),
            "./Line.vue" => Some("./Li\nne\u{2028}.vue.ts".into()),
            _ => None,
        }
    };
    let source = "import a from './Panel.vue';\nimport b from './Same.vue';\nimport c from './Alias.vue.ts';\nexport * from './Quote.vue';\nconst d = import(\"./Line.vue\");\nimport e from './Untouched.vue';\nimport f from './Esc\\u0061ped.vue';\n";
    let rewrite = ImportRewriter::new().rewrite_with_syntax(
        source,
        SourceType::ts(),
        policy,
        RewriteSyntaxMode::Retain,
    );
    let syntax = rewrite.syntax.as_ref().unwrap();
    let mut derived = syntax
        .rewritten_specifiers(source, &rewrite.replaced)
        .expect("a clean parse")
        .map(String::from)
        .collect::<Vec<_>>();
    let mut parsed = collect_specifier_spans(&rewrite.result.code, SourceType::ts())
        .into_iter()
        .map(|(_, _, specifier, _)| specifier)
        .collect::<Vec<_>>();
    derived.sort();
    parsed.sort();
    assert_eq!(derived, parsed);
    // Replaced literals take the policy's value; equal and unrewritten ones
    // keep their parser-decoded spelling.
    let mut expected: Vec<String> = vec![
        "./Panel.vue.ts".into(),
        "./Same.vue".into(),
        cstr!(
            "./Alias.vue.ts{}",
            crate::batch::AUTHORED_VUE_TS_ALIAS_SENTINEL
        ),
        "./Qu'o\"te\\.vue.ts".into(),
        "./Li\nne\u{2028}.vue.ts".into(),
        "./Untouched.vue".into(),
        "./Escaped.vue".into(),
    ];
    expected.sort();
    assert_eq!(derived, expected);
}

#[test]
fn rewritten_specifiers_of_every_source_match_a_parse_of_the_rewritten_text() {
    let mut recovered = 0;
    for source in SOURCES {
        let rewrite = retained(source, SourceType::ts());
        let syntax = rewrite.syntax.as_ref().unwrap();
        let Some(derived) = syntax.rewritten_specifiers(source, &rewrite.replaced) else {
            // Only a fatal parse error defers to the rewritten text's own recovery.
            assert!(syntax.recovered, "{source}");
            recovered += 1;
            continue;
        };
        assert!(!syntax.recovered, "{source}");
        let mut derived = derived.map(String::from).collect::<Vec<_>>();
        let mut parsed = collect_specifier_spans(&rewrite.result.code, SourceType::ts())
            .into_iter()
            .map(|(_, _, specifier, _)| specifier)
            .collect::<Vec<_>>();
        derived.sort();
        parsed.sort();
        assert_eq!(derived, parsed, "{source}");
    }
    assert_eq!(recovered, 2, "both fatal sources take the recovery path");
}

#[test]
fn a_retained_parse_answers_only_for_its_own_bytes_and_source_type() {
    let source = SOURCES[0];
    let rewrite = retained(source, SourceType::ts());
    let syntax = rewrite.syntax.as_ref().unwrap();
    assert!(syntax.reusable_for(source, SourceType::ts()));
    assert!(!syntax.reusable_for(source, SourceType::tsx()));
    assert!(!syntax.reusable_for(&source.replace("Panel", "Pane1"), SourceType::ts()));
    assert!(!syntax.reusable_for(&cstr!("{source} "), SourceType::ts()));

    // Other bytes are parsed again, never answered from the retained spans.
    let changed = source.replace("./Panel.vue", "./Other.vue");
    let fresh = ImportRewriter::new().rewrite_with_syntax(
        &changed,
        SourceType::ts(),
        mirror,
        RewriteSyntaxMode::Reuse(None),
    );
    let replayed = ImportRewriter::new().rewrite_with_syntax(
        &changed,
        SourceType::ts(),
        mirror,
        RewriteSyntaxMode::Reuse(Some(syntax)),
    );
    assert_same_rewrite(&changed, &fresh, &replayed);
    assert_eq!(
        replayed.result.code,
        "import Panel from './Other.vue.ts';\nexport * from './Other.vue.ts';\nexport { a } from './a';\n"
    );
}

#[test]
fn a_projection_without_rewrite_candidates_still_retains_the_parse_it_replays() {
    let root = tempfile::tempdir().unwrap();
    let mirror_root = tempfile::tempdir().unwrap();
    let roots = (root.path(), mirror_root.path());
    // No `.vue`, project root, relative or alias spelling: nothing to rewrite.
    let source = "import { ref } from 'vue';\nexport * from 'pkg';\ntype H = import(\"vize:virtual-ts\").H;\n";
    let options = super::super::VirtualProjectRewriteOptions {
        preserve_relative_declarations: false,
        mirrorable_project_files: None,
        alias_rewrite_policy: None,
        module_resolver: None,
    };
    let rewriter = ImportRewriter::new();
    let rewrite_generated = |options, syntax| {
        rewriter.rewrite_generated_for_virtual_project_with_captured_stars(
            source,
            SourceType::ts(),
            roots,
            Some(root.path()),
            options,
            syntax,
        )
    };
    let retained = rewrite_generated(options, RewriteSyntaxMode::Retain);
    let skipped = rewrite_generated(options, RewriteSyntaxMode::Reuse(None));
    // The registration's own result is the unparsed one, stars included.
    assert_same_rewrite(source, &skipped, &retained);
    assert!(retained.stars.is_empty());
    let syntax = retained
        .syntax
        .as_ref()
        .expect("the parse later passes replay");

    // The final pass resolves every module, so it never skips the parse.
    let resolve = |specifier: &str, _mode: PackageResolutionMode| {
        (specifier == "pkg").then(|| "/resolved/pkg.d.ts".to_owned())
    };
    let options = super::super::VirtualProjectRewriteOptions {
        module_resolver: Some(&resolve),
        ..options
    };
    let fresh = rewrite_generated(options, RewriteSyntaxMode::Reuse(None));
    let replayed = rewrite_generated(options, RewriteSyntaxMode::Reuse(Some(syntax)));
    assert_same_rewrite(source, &fresh, &replayed);
    assert_eq!(
        fresh.result.code,
        "import { ref } from 'vue';\nexport * from '/resolved/pkg.d.ts';\ntype H = import(\"vize:virtual-ts\").H;\n"
    );
    assert_eq!(fresh.stars.len(), 1);
    assert_eq!(
        syntax.dependency_occurrences(source),
        rewriter.collect_all_specifier_occurrences(source, SourceType::ts())
    );
}
