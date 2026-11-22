use super::*;
use std::hint::black_box;

#[test]
#[ignore = "measure the existing joined checker/editor plus independent lint against one style visit"]
fn paired_style_lint_requests() {
    davinci_harness::alloc::mark_installed();
    let base = options();
    let rules = CssLinter::with_all_rules();
    let mut failures = Vec::new();
    let mut measure = |label: &str, source: &str, at| {
        in_document(source, at, |ctx| {
            let run = |joined| {
                let ctx = black_box(ctx);
                if joined {
                    ctx.style_module_products_with_lint(
                        black_box(&base),
                        true,
                        false,
                        black_box(&rules),
                        true,
                    )
                    .unwrap()
                    .unwrap()
                } else {
                    let modules = ctx
                        .style_module_products(black_box(&base), true, false)
                        .unwrap()
                        .unwrap();
                    (modules, original_lint(ctx, black_box(&rules), true))
                }
            };
            let (old, old_lint) = run(false);
            let (new, new_lint) = run(true);
            assert_equivalent(&new, &old, label);
            assert_lint(&new_lint, &old_lint, label);
            if super::super::profile::regression(label, run) {
                failures.push(label.to_owned());
            }
        });
    };
    for (language, body) in STYLES {
        let source = format!("{TEMPLATE}<style module lang='{language}'>{body}</style>");
        measure(
            language,
            &source,
            source.find(CURSOR).unwrap() + CURSOR.len(),
        );
    }
    for (name, styles) in [
        ("template", ""),
        (
            "ordinary",
            "<style>.card { color: red !important; }</style>",
        ),
        (
            "mixed",
            "<style>.before { display: none }</style><style module lang='scss'>.card { &-body { margin-left: 4px !important; } }</style><style>.after { color: blue }</style>",
        ),
    ] {
        let source = format!("{TEMPLATE}{styles}");
        measure(name, &source, source.find(CURSOR).unwrap() + CURSOR.len());
    }
    original_modules(|name, source, offsets| {
        if let Some(&at) = offsets.get(1) {
            measure(name, source, at);
        }
    });
    assert!(failures.is_empty(), "regressions: {failures:?}");
}
