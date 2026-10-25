//! The ownership observer consumes actual accepted Router edits, without changing them.

use super::{FILE, ROUTE, SourceType, apply, output};
use crate::virtual_ts::diagnostic_ownership::{DiagnosticOwnershipCollector, InputOwner};
use std::ops::Range;

fn binding() -> (InputOwner, Range<usize>) {
    let template = "<Child :count=\"items[0].length\" />";
    let allocator = vize_l0::Allocator::new();
    let (root, _) = vize_armature::parse(&allocator, template);
    let mut analyzer = vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
    analyzer.analyze_script_setup("import Child from './Child.vue'; const items: string[] = [];");
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();
    let usages = vize_croquis::facts::component_usage_list(&summary);
    let usage = usages.first().expect("real component usage");
    let prop = usage.props.first().expect("real passed prop");
    let owner = InputOwner::for_prop(usage, prop).expect("context-free component input");
    let start = template
        .find("items[0].length")
        .expect("handwritten authored value");
    let source = start..start + "items[0].length".len();
    (owner, source)
}

#[test]
fn actual_router_insertions_preserve_full_output_and_transform_only_proven_ranges() {
    for newline in ["\n", "\r\n"] {
        for prefix in ["", "/* 日本 🧭 */\n"] {
            let script = format!("{prefix}import {{ useRoute }} from 'vue-router';\nconst before = 0;\nconst route = useRoute();\nconst after = 1;\n").replace('\n', newline);
            let call = script.find("useRoute()").expect("real route call");
            let insertion = call + "useRoute".len();
            let primary = script.find("before = 0").expect("stable primary") + "before = ".len();
            for (range, expected) in [
                (call - 4..call - 1, Some(call - 4..call - 1)),
                (
                    insertion..insertion + 2,
                    Some(insertion + ROUTE.len()..insertion + ROUTE.len() + 2),
                ),
                (insertion - 1..insertion + 1, None),
                (insertion - 2..insertion, Some(insertion - 2..insertion)),
                (
                    insertion + 4..insertion + 8,
                    Some(insertion + ROUTE.len() + 4..insertion + ROUTE.len() + 8),
                ),
            ] {
                let (owner, authored) = binding();
                let collector = DiagnosticOwnershipCollector::default();
                collector
                    .borrow_mut()
                    .record(owner, primary..primary + 1, authored.clone(), false);
                collector
                    .borrow_mut()
                    .record(owner, range, authored.clone(), true);
                let mut baseline = output(&script);
                let mut observed = output(&script);
                assert_eq!(
                    apply(
                        &mut baseline,
                        &script,
                        |at| at,
                        0..script.len(),
                        SourceType::ts(),
                        FILE,
                        None
                    ),
                    apply(
                        &mut observed,
                        &script,
                        |at| at,
                        0..script.len(),
                        SourceType::ts(),
                        FILE,
                        Some(&collector)
                    )
                );
                assert_eq!(observed.code, baseline.code);
                assert_eq!(
                    observed.mapping, baseline.mapping,
                    "all navigation rows, features and links stay exact"
                );
                let echoes = collector
                    .into_inner()
                    .into_echoes()
                    .into_iter()
                    .map(|echo| (echo.generated, echo.authored))
                    .collect::<Vec<_>>();
                assert_eq!(
                    echoes,
                    expected
                        .map(|range| vec![(range, authored)])
                        .unwrap_or_default()
                );
            }
        }
    }
}
