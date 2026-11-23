use super::super::*;
use std::{fs, path::Path};

#[test]
#[ignore = "original upstream descriptor locations from the pinned Vue compiler"]
fn original_spans_and_complete_results() {
    let oracle = std::env::var_os("VIZE_ORIGINAL_TEMPLATE_ORACLE").unwrap();
    let cases: Vec<(String, Option<(u32, u32)>, usize)> =
        serde_json::from_slice(&fs::read(oracle).unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let (mut selected, mut partial, mut recovered, mut contributions, mut upstream_errors) =
        (0, 0, 0, 0, 0);
    let mut upstream_recoveries = 0;
    let mut stock_spans = 0;
    for (name, expected, error_count) in &cases {
        let source = fs::read_to_string(root.join(name)).unwrap();
        let arena = Allocator::default();
        let observation = Vue.observe_descriptor(&arena, &source, options());
        let old = observation.admitted();
        let candidate = observation.admitted_template();
        if let Ok(whole) = old {
            assert_eq!(
                candidate
                    .as_ref()
                    .expect("a whole descriptor qualifies")
                    .as_ref()
                    .map(|t| t.block().span()),
                whole.template().map(|t| t.block().span()),
                "{name}"
            );
        }
        let Ok(Some(view)) = candidate else { continue };
        let span = view.block().span();
        if let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&source, Default::default())
            && let Some(template) = descriptor.template
        {
            assert_eq!(
                (span.start as usize, span.end as usize),
                (template.loc.start, template.loc.end),
                "{name}"
            );
            assert_eq!(view.block().source(), template.content.as_ref(), "{name}");
            stock_spans += 1;
        }
        if *expected != Some((span.start, span.end)) {
            // The stock splitter preserves all 92 authored bytes of this historical
            // interrupted slot. Vue 3.5.35 emits three errors and an empty payload.
            // The two independent LSP regression fixtures carry identical bytes.
            const BROKEN_SLOT: &str = include_str!(
                "../../../../tests/_fixtures/differential/lsp/broken-overlay-isolation-10201/Child.broken.vue.txt"
            );
            assert_eq!(
                source, BROKEN_SLOT,
                "unrecognized upstream range difference in {name}"
            );
            assert!(old.is_ok());
            assert_eq!(
                (*expected, span, *error_count),
                (Some((79, 79)), Span::new(79, 171), 3)
            );
            upstream_recoveries += 1;
        }
        let (errors, rows) = compare(&arena, view);
        selected += 1;
        partial += usize::from(old.is_err());
        recovered += usize::from(errors > 0);
        contributions += rows;
        upstream_errors += usize::from(*error_count > 0);
    }
    println!(
        "original template queries files={} selected={selected} partial={partial} syntax_recovered={recovered} upstream_errors={upstream_errors} upstream_recovery_ranges={upstream_recoveries} stock_ranges={stock_spans} contributions={contributions}",
        cases.len()
    );
    assert!(selected > 0 && contributions > 0);
}
