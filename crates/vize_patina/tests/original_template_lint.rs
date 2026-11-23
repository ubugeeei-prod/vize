//! Compare the complete published warning with the existing rule on the same source.
use std::{fs, path::Path};
use vize_atelier_sfc::{SfcDescriptor, SfcParseOptions, parse_sfc};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
    pass::NoObserver,
};
use vize_l1::{
    container::{Vue, vue::DescriptorOptions},
    markup::SurfaceParseOptions,
};
use vize_l1_to_l2::pass::template::OriginalTemplate;
use vize_patina::{
    LintContext, LintDiagnostic, Linter, Rule, RuleRegistry, rules::facts::MaxTemplateComplexity,
};

#[global_allocator]
static ALLOCATOR: davinci_harness::alloc::CountingAllocator<std::alloc::System> =
    davinci_harness::alloc::CountingAllocator::system();

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

fn published<'a>(
    arena: &'a Allocator,
    stock: &'a SfcDescriptor<'a>,
    source: &'a str,
    shared: Option<&OriginalTemplate<'a>>,
) -> Vec<LintDiagnostic> {
    let mut context = LintContext::new(arena, source, "Original.vue");
    context.set_sfc_descriptor(stock);
    if let Some(template) = shared {
        if let Some(warning) = MaxTemplateComplexity::original_diagnostic(template) {
            context.report(warning);
        }
    } else {
        MaxTemplateComplexity.run_on_sfc(&mut context);
    }
    context.into_diagnostics()
}

fn wire(diagnostics: &[LintDiagnostic]) -> serde_json::Value {
    serde_json::Value::Array(diagnostics.iter().map(|d| serde_json::json!({
        "rule": d.rule_name,
        "severity": d.severity,
        "message": d.message.as_str(),
        "start": d.start, "end": d.end,
        "help": d.help.as_deref(),
        "labels": d.labels.iter().map(|label| (label.message.as_str(), label.start, label.end)).collect::<Vec<_>>(),
        "fix": d.fix,
    })).collect())
}

fn source(width: usize, condition: &str) -> String {
    let body = format!(r#"<p v-if="{condition}">雪</p>"#).repeat(width);
    format!(
        "<!--🧭-->\r\n<script src='outside.ts'></script>\r\n<template lang='html'>{body}</template>"
    )
}

#[test]
fn the_shared_warning_matches_the_registered_rule_and_points_into_the_original() {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(MaxTemplateComplexity));
    let linter = Linter::with_registry(registry);
    for (source, expected) in [
        (source(1, "ok"), 0),
        (source(20, "ready &amp;&amp; open"), 1),
        (source(28, "value ? a : b"), 1),
        (
            source(20, "ok").replace("</template>", "<p :x=>😀</p></template>"),
            1,
        ),
    ] {
        let arena = Allocator::default();
        let owner = Vue.observe_descriptor(&arena, &source, options());
        assert!(owner.admitted().is_err());
        let selected = owner.admitted_template().unwrap().unwrap();
        let template = OriginalTemplate::analyze(&arena, selected, &mut NoObserver).unwrap();
        assert_eq!(template.open_tag().slice(&source), "<template lang='html'>");
        let stock = parse_sfc(&source, SfcParseOptions::default()).unwrap();
        let result = published(&arena, &stock, &source, Some(&template));
        assert_eq!(result.len(), expected);
        assert_eq!(
            wire(&result),
            wire(&published(&arena, &stock, &source, None))
        );
        let (warnings, parse): (Vec<_>, Vec<_>) = linter
            .lint_sfc(&source, "Original.vue")
            .diagnostics
            .into_iter()
            .partition(|d| d.rule_name == "vue/max-template-complexity");
        assert_eq!(wire(&result), wire(&warnings));
        assert_eq!(parse.len(), template.syntax_errors().len());
        if source.contains(":x=>") {
            let [error] = parse.as_slice() else {
                panic!("one independent parser diagnostic");
            };
            assert_eq!(error.rule_name, "parser/template");
            assert_eq!(error.severity, vize_patina::Severity::Error);
            assert_eq!(
                error.message.as_str(),
                "Attribute `:x` is missing a value after `=`; continuing without the value."
            );
            let delimiter = source.find(":x=>").unwrap() as u32 + 3;
            assert_eq!((error.start, error.end), (delimiter, delimiter + 1));
            assert_eq!(&source[error.start as usize..error.end as usize], ">");
        }
        if let Some(warning) = result.first() {
            assert_eq!(warning.labels.len(), 5);
            assert_eq!(
                &source[warning.start as usize..warning.end as usize],
                "<template lang='html'>"
            );
        }
    }
}

#[test]
#[ignore = "read the tracked or external original Vue source manifest"]
fn complete_rule_differential() {
    let manifest = std::env::var_os("VIZE_ORIGINAL_TEMPLATE_ORACLE").unwrap();
    let paths: Vec<(String, Option<(u32, u32)>, usize)> =
        serde_json::from_slice(&fs::read(manifest).unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut checked = 0;
    let mut warnings = 0;
    let mut labels = 0;
    let mut partial = 0;
    let mut refused_by_stock = 0;
    for (path, _, _) in &paths {
        let text = fs::read_to_string(root.join(path)).unwrap();
        let arena = Allocator::default();
        let owner = Vue.observe_descriptor(&arena, &text, options());
        let Ok(Some(block)) = owner.admitted_template() else {
            continue;
        };
        let template = OriginalTemplate::analyze(&arena, block, &mut NoObserver).unwrap();
        let stock = match parse_sfc(&text, SfcParseOptions::default()) {
            Ok(stock) => stock,
            Err(error) => {
                // Two tracked editor snapshots have the identical interrupted
                // nested slot. Stock stops before invoking the rule. The kept
                // original template is below the warning thresholds.
                const INTERRUPTED: &str = include_str!(
                    "../../../tests/_fixtures/differential/lsp/broken-overlay-isolation-10201/Child.broken.vue.txt"
                );
                assert_eq!(text, INTERRUPTED, "{path}");
                assert_eq!(error.code.as_deref(), Some("MALFORMED_BLOCK"));
                let location = error.loc.unwrap();
                assert_eq!((location.start, location.end), (69, 183));
                assert_eq!(template.block().span(), vize_l0::Span::new(79, 171));
                assert!(MaxTemplateComplexity::original_diagnostic(&template).is_none());
                refused_by_stock += 1;
                continue;
            }
        };
        let before = published(&arena, &stock, &text, None);
        let after = published(&arena, &stock, &text, Some(&template));
        assert_eq!(wire(&after), wire(&before), "{path}");
        checked += 1;
        partial += usize::from(owner.admitted().is_err());
        warnings += after.len();
        labels += after.iter().map(|d| d.labels.len()).sum::<usize>();
    }
    println!(
        "complexity lint files={} checked={checked} partial={partial} known_stock_refusals={refused_by_stock} warnings={warnings} labels={labels}",
        paths.len()
    );
    assert!(checked > 0 && warnings > 0);
}

mod original_template_lint {
    mod profile;
}
