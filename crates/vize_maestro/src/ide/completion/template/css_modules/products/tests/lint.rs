//! Compare every checker option, completion item and Patina rule result for a single open source.
use super::*;
use vize_patina::{
    LintDiagnostic,
    rules::css::{CssLintResult, CssLinter},
};

fn original_lint(ctx: &IdeContext<'_>, rules: &CssLinter, respect_comments: bool) -> CssLintResult {
    let mut result = CssLintResult::default();
    for style in &ctx.descriptor().unwrap().styles {
        let Ok(language) = vize_l1::container::lang::style_lang(style.lang.as_deref()) else {
            continue;
        };
        if style.src.is_some() || !rules.reads(language) || style.content.trim().is_empty() {
            continue;
        }
        let block = rules.lint_block(
            style.content.as_ref(),
            style.loc.start,
            language,
            respect_comments,
        );
        result.diagnostics.extend(block.diagnostics);
        result.error_count += block.error_count;
        result.warning_count += block.warning_count;
    }
    result
}

fn diagnostic(row: &LintDiagnostic) -> serde_json::Value {
    let LintDiagnostic {
        rule_name,
        severity,
        message,
        start,
        end,
        help,
        labels,
        fix,
    } = row;
    let labels: Vec<_> = labels
        .iter()
        .map(|label| (&label.message, label.start, label.end))
        .collect();
    serde_json::json!([rule_name, severity, message, start, end, help, labels, fix])
}

fn assert_lint(a: &CssLintResult, b: &CssLintResult, name: &str) {
    assert_eq!(
        (a.error_count, a.warning_count, a.diagnostics.len()),
        (b.error_count, b.warning_count, b.diagnostics.len()),
        "{name}"
    );
    for (actual, expected) in a.diagnostics.iter().zip(&b.diagnostics) {
        assert_eq!(diagnostic(actual), diagnostic(expected), "{name}");
    }
}

fn compare_lint(
    ctx: &IdeContext<'_>,
    base: &VirtualTsOptions,
    rules: &CssLinter,
    comments: bool,
    name: &str,
) -> CssLintResult {
    let (modules, lint) = ctx
        .style_module_products_with_lint(base, true, false, rules, comments)
        .unwrap()
        .unwrap();
    assert_equivalent(&modules, &original(ctx, base, true, false), name);
    assert_lint(&lint, &original_lint(ctx, rules, comments), name);
    lint
}

#[test]
fn mixed_blocks_preserve_rule_order_absolute_spans_and_inline_disables() {
    let base = options();
    let rules = CssLinter::with_all_rules();
    let source = concat!(
        "<template>😀{{ $style.card }}</template>\r\n",
        "<style>.before { color: red !important; }</style>\r\n",
        "<style module>\r\n/* vize-disable-next-line css/no-important */\r\n",
        ".card { color: red !important; }\r\n#unsuppressed { margin-left: 4px !important; }\r\n</style>",
        "<style module='other' lang='scss'>.a { &-b { color: blue !important; } }</style>",
        "<style lang='less'>.end { color: green !important; }</style>",
        "<style module lang='unknown'>.opaque { color: yellow !important; }</style>",
        "<style src='./external.css'></style><style module>   </style>"
    );
    in_document(source, source.find(CURSOR).unwrap() + CURSOR.len(), |ctx| {
        let on = compare_lint(ctx, &base, &rules, true, "disable on");
        let off = compare_lint(ctx, &base, &rules, false, "disable off");
        let starts = |result: &CssLintResult| {
            result
                .diagnostics
                .iter()
                .filter(|d| d.rule_name == "css/no-important")
                .map(|d| {
                    assert_eq!(&source[d.start as usize..d.end as usize], "!important");
                    d.start as usize
                })
                .collect::<Vec<_>>()
        };
        let flags: Vec<_> = source.match_indices("!important").map(|(i, _)| i).collect();
        assert_eq!(starts(&off), flags[..5]);
        assert_eq!(starts(&on), [flags[0], flags[2], flags[3], flags[4]]);
        assert!(
            compare_lint(ctx, &base, &CssLinter::new(), true, "empty registry")
                .diagnostics
                .is_empty()
        );
    });
    for (lang, body) in STYLES {
        let source = format!(
            "{TEMPLATE}<style module lang='{lang}'>{body}</style><style lang='{lang}'>{body}</style>"
        );
        in_document(
            &source,
            source.find(CURSOR).unwrap() + CURSOR.len(),
            |ctx| {
                compare_lint(ctx, &base, &rules, true, lang);
            },
        );
    }
    for body in [
        ".incomplete {",
        "/* unterminated",
        ".unbalanced }",
        ".\\31 abc { display: none }",
    ] {
        let source = format!("{TEMPLATE}<style module>{body}</style>");
        in_document(&source, 0, |ctx| {
            compare_lint(ctx, &base, &rules, true, body);
        });
    }
}

#[test]
fn combined_products_require_the_current_revision_and_original_style_ranges() {
    let source = format!("{TEMPLATE}<style>.first{{}}</style><style module>.second{{}}</style>");
    in_document(&source, 0, |ctx| {
        let mut descriptor = (**ctx.descriptor().unwrap()).clone();
        let plain = descriptor.styles.first_mut().unwrap();
        plain.content = ".other{}".into();
        let base = options();
        assert!(matches!(
            base.observe_file_styles(&descriptor, true, false, |_, _| {}),
            Err(SourceFrameError::BlockNotRootSlice)
        ));
        assert!(
            base.observe_style_modules(&descriptor, true, false, |_, _, _| {})
                .is_ok()
        );
        let read =
            || ctx.style_module_products_with_lint(&base, true, false, &CssLinter::new(), true);
        ctx.state.documents.close(ctx.uri);
        assert!(matches!(read(), Err(StyleModuleRefusal::Closed)));
        ctx.state
            .documents
            .open(ctx.uri.clone(), source.clone(), 1, "vue".into());
        assert!(matches!(read(), Err(StyleModuleRefusal::Superseded)));
    });
}

#[test]
#[ignore = "original file-wide checker, member and complete CSS lint results"]
fn original_style_lint_products_match() {
    let base = options();
    let rules = CssLinter::with_all_rules();
    let paths = std::fs::read(std::env::var_os("VIZE_STYLE_MODULE_INPUTS").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut files, mut invalid, mut parsed, mut styles, mut modules, mut diagnostics) =
        (0, 0, 0, 0, 0, 0);
    for path in paths.split(|&b| b == 0).filter(|p| !p.is_empty()) {
        let name = std::str::from_utf8(path).unwrap();
        let bytes = std::fs::read(root.join(name)).unwrap();
        files += 1;
        let Ok(source) = std::str::from_utf8(&bytes) else {
            invalid += 1;
            continue;
        };
        let at = source.find(CURSOR).map_or(0, |pos| pos + CURSOR.len());
        in_document(source, at, |ctx| {
            let Some(descriptor) = ctx.descriptor() else {
                assert!(
                    ctx.style_module_products_with_lint(&base, true, false, &rules, true)
                        .unwrap()
                        .is_none(),
                    "{name}"
                );
                return;
            };
            parsed += 1;
            styles += descriptor.styles.len();
            modules += descriptor
                .styles
                .iter()
                .filter(|s| s.module.is_some())
                .count();
            diagnostics += compare_lint(ctx, &base, &rules, true, name)
                .diagnostics
                .len();
        });
    }
    assert!(parsed > 0 && styles > 0 && diagnostics > 0);
    eprintln!(
        "original_style_lint files={files} invalid={invalid} parsed={parsed} styles={styles} modules={modules} diagnostics={diagnostics}"
    );
}

#[cfg(feature = "micro-profiling")]
mod profile;
