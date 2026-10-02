//! Non-rendering lint pragmas must not change production HTML or hydration
//! boundaries. The corpus also compiles this source through the SFC adapter.

use super::{json, render_cases};
use serde_json::Value;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};
use vize_atelier_ssr::{SsrCompilerOptions, compile_ssr_with_options};
use vize_l0::Allocator;

#[test]
fn lint_pragmas_keep_vue_branch_and_loop_layout() {
    let descriptor = parse_sfc(
        include_str!("../fixtures/lint-pragma-layout.vue"),
        SfcParseOptions::default(),
    )
    .expect("lint pragma fixture");
    let template = &descriptor.template.expect("template").content;
    let compile = || {
        let allocator = Allocator::new();
        let (_, errors, result) =
            compile_ssr_with_options(&allocator, template, SsrCompilerOptions::default());
        assert!(errors.is_empty(), "{errors:?}");
        format!("{}{}", result.preamble, result.code)
    };
    let code = compile();
    #[cfg(feature = "legacy-differential")]
    assert_eq!(
        code,
        vize_atelier_ssr::differential::with_legacy_lane(compile),
        "native and compatibility output"
    );
    for (show, branch) in [
        (false, "<p><em>message</em></p>"),
        (true, "<strong>Title</strong>"),
    ] {
        let cases = format!(
            "{{\"check\":true,\"cases\":[{{\"name\":\"lint-pragma-{show}\",\"template\":{},\"vize\":{},\"data\":{{\"show\":{show},\"html\":\"<em>message</em>\",\"heading\":\"Title\",\"items\":[\"one\",\"two\"]}}}}]}}",
            json(template),
            json(&code),
        );
        let rendered = render_cases(&cases);
        let result: Value = serde_json::from_str(rendered.trim()).expect("rendered HTML");
        assert_eq!(
            result["vize"]["html"].as_str().expect("HTML"),
            format!("<main>{branch}<ul><!--[--><li>one</li><li>two</li><!--]--></ul></main>"),
        );
    }
}
