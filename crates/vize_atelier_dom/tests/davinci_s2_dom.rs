//! P2-11 installment 2 witness: static native HTML elements (with
//! static attributes), old DOM lane vs S2 emit, compared **byte-for-byte**
//! including helper usage.
//!
//! `vize_atelier_dom` is published; the Davinci crates are not. The
//! comparator therefore rides stripped-on-publish dev-deps (the same
//! carve-out P2-9 used). The shipped `compile_template` path is
//! unchanged. `VIZE_DAVINCI_DOM=legacy` disarms the dual-run; the
//! pinned comparison count makes a silent disarm a loud failure.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

use vize_atelier_dom::compile_template;
use vize_carton::Allocator;
use vize_ricalco::{DOM_LANE_FLAG, emit_dom_source};

const BATTERY: &[(&str, &str)] = &[
    ("empty_div", "<div></div>"),
    ("div_with_text", "<div>hello</div>"),
    ("nested_elements", "<div><span>hello</span></div>"),
    ("paragraph", "<p>hi</p>"),
    ("sibling_spans", "<div><span>a</span><span>b</span></div>"),
    ("class_attr", r#"<div class="x"></div>"#),
    (
        "id_and_class",
        r#"<div id="app" class="container">static</div>"#,
    ),
    ("data_attr", r#"<div data-id="1"></div>"#),
    ("boolean_attr", "<div disabled></div>"),
    ("nested_class", r#"<div><span class="x">hello</span></div>"#),
];

fn shipped(src: &str) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_template(&allocator, src);
    assert!(errors.is_empty(), "shipped lane errors: {errors:?}");
    format!("{}\n{}", result.preamble, result.code)
}

#[test]
fn s2_static_elements_match_the_shipped_dom_lane_byte_for_byte() {
    let mut compared = 0u64;
    let mut skipped_legacy_flag = 0u64;
    if std::env::var(DOM_LANE_FLAG).is_ok_and(|value| value == "legacy") {
        skipped_legacy_flag += 1;
    } else {
        let allocator = Allocator::new();
        for (name, src) in BATTERY {
            let old = shipped(src);
            let new = emit_dom_source(&allocator, src)
                .unwrap_or_else(|error| panic!("{name}: S2 emit refused: {error:?}"))
                .assembled();
            assert_eq!(
                old.as_str(),
                new.as_str(),
                "{name}: S2 DOM emit diverged from the shipped lane"
            );
            compared += 1;
        }
    }
    assert_eq!(
        (compared, skipped_legacy_flag),
        (BATTERY.len() as u64, 0),
        "a cfg or {DOM_LANE_FLAG}=legacy regression disarmed the dual-run"
    );
}

#[test]
fn the_dom_lane_flag_has_its_recorded_name() {
    assert_eq!(DOM_LANE_FLAG, "VIZE_DAVINCI_DOM");
}
