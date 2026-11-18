//! #13314: a v-pre subtree renders as plain markup on both SSR lanes.

#![cfg(feature = "legacy-differential")]

use vize_atelier_ssr::differential::{SsrLaneComparison, compare_ssr_lanes};
use vize_atelier_ssr::{SsrCompilerExperimentalOptions, SsrCompilerOptions};

fn both_lanes(source: &str) -> SsrLaneComparison {
    let pair = compare_ssr_lanes(
        source,
        &SsrCompilerOptions::default(),
        &SsrCompilerExperimentalOptions::default(),
    );
    assert!(
        pair.selected.errors.is_empty(),
        "{source}: {:?}",
        pair.selected.errors
    );
    assert!(
        pair.legacy.errors.is_empty(),
        "{source}: {:?}",
        pair.legacy.errors
    );
    assert_eq!(
        pair.selected.result.code, pair.legacy.result.code,
        "{source}"
    );
    assert_eq!(
        pair.selected.result.preamble, pair.legacy.result.preamble,
        "{source}"
    );
    pair
}

fn pushes(code: &str) -> Vec<&str> {
    code.lines()
        .filter(|line| line.trim_start().starts_with("_push("))
        .collect()
}

/// Each line is `@vue/compiler-sfc` 3.5.35 `compileTemplate({ ssr: true })` output.
#[test]
fn a_v_pre_subtree_renders_plain_markup_on_both_lanes() {
    for (source, lane, expected) in [
        (
            "<div v-pre><div><Child>{{ x }}</Child></div></div>",
            "s4",
            "  _push(`<div${_ssrRenderAttrs(_attrs)}><div><Child>{{ x }}</Child></div></div>`)",
        ),
        (
            "<Child v-pre>{{ x }}</Child>",
            "s4",
            "  _push(`<Child${_ssrRenderAttrs(_attrs)}>{{ x }}</Child>`)",
        ),
        (
            r#"<div v-pre><component is="x"/></div>"#,
            // The selected lane still refuses a literal `<component>`.
            "legacy.element",
            r#"  _push(`<div${_ssrRenderAttrs(_attrs)}><component is="x"></component></div>`)"#,
        ),
        (
            "<div v-pre><my-comp/></div>",
            "s4",
            "  _push(`<div${_ssrRenderAttrs(_attrs)}><my-comp></my-comp></div>`)",
        ),
        (
            "<div v-pre><slot/></div>",
            "s4",
            "  _push(`<div${_ssrRenderAttrs(_attrs)}><slot></slot></div>`)",
        ),
        (
            r#"<div v-pre><template v-if="a">x</template></div>"#,
            "s4",
            r#"  _push(`<div${_ssrRenderAttrs(_attrs)}><template v-if="a">x</template></div>`)"#,
        ),
    ] {
        let pair = both_lanes(source);
        assert_eq!(pair.lane, lane, "{source}");
        assert_eq!(pushes(&pair.selected.result.code), [expected], "{source}");
    }
}

/// HTML tree construction reads the type outside v-pre, so a literal component
/// stays where it was authored, and both lanes agree without a recovery.
#[test]
fn a_literal_component_keeps_its_authored_place_on_both_lanes() {
    for (source, expected) in [
        (
            "<table v-pre><tr><Row></Row></tr></table>",
            "  _push(`<table${_ssrRenderAttrs(_attrs)}><tbody><tr><Row></Row></tr></tbody></table>`)",
        ),
        (
            "<p v-pre><Child><div></div></Child></p>",
            "  _push(`<p${_ssrRenderAttrs(_attrs)}><Child><div></div></Child></p>`)",
        ),
    ] {
        let pair = both_lanes(source);
        assert_eq!(pushes(&pair.selected.result.code), [expected], "{source}");
    }
}
