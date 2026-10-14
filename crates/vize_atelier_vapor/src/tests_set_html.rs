//! `v-html` writes through the runtime's `setHtml`, which maps `null` and
//! `undefined` to an empty string and skips HTML it already set, so an effect
//! re-run for another binding keeps the element's children.

use super::{VaporCompilerOptions, compile_vapor};
use vize_carton::{Allocator, String};

fn render_lines(source: &str, davinci_retained_lane: bool) -> std::vec::Vec<String> {
    let allocator = Allocator::new();
    let result = compile_vapor(
        &allocator,
        source,
        VaporCompilerOptions {
            prefix_identifiers: true,
            davinci_retained_lane,
            ..Default::default()
        },
    );
    assert!(
        result.error_messages.is_empty(),
        "{:?}",
        result.error_messages
    );
    result
        .code
        .lines()
        .map(|line| String::from(line.trim()))
        .collect()
}

#[test]
fn v_html_sets_html_through_the_runtime_helper_on_both_lanes() {
    for retained in [false, true] {
        let lines = render_lines(r#"<div v-html="html"></div>"#, retained);
        assert_eq!(
            lines
                .iter()
                .filter(|line| *line == "_renderEffect(() => _setHtml(n0, _ctx.html))")
                .count(),
            1,
            "{lines:#?}"
        );
        let shared = render_lines(r#"<div v-html="html" :id="id"></div>"#, retained);
        assert_eq!(
            shared
                .iter()
                .filter(|line| *line == "_setHtml(n0, _ctx.html)")
                .count(),
            1,
            "{shared:#?}"
        );
        assert_eq!(
            shared
                .iter()
                .filter(|line| line.starts_with("n0.innerHTML"))
                .count(),
            0,
            "{shared:#?}"
        );
    }
}
