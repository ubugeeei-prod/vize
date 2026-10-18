//! Real parser/transform coverage of both SSR emitters' empty-value consumers.

use crate::compile::{SsrLane, compile_ssr_on_lane};
use crate::{SsrCompilerExperimentalOptions, SsrCompilerOptions};
use vize_atelier_core::options::{CustomElementMatcher, TemplateSyntaxMode};
use vize_l0::{Allocator, String, cstr};

fn compile(source: &str, lane: SsrLane, is_ts: bool) -> String {
    let allocator = Allocator::new();
    let (_, errors, result) = compile_ssr_on_lane(
        &allocator,
        source,
        SsrCompilerOptions {
            is_ts,
            ..Default::default()
        },
        TemplateSyntaxMode::Standard,
        CustomElementMatcher::default(),
        SsrCompilerExperimentalOptions::default(),
        true,
        lane,
    );
    assert!(errors.is_empty(), "{source}: {errors:?}");
    assert_eq!(result.map, None);
    cstr!("{}\n{}", result.preamble, result.code)
}

#[test]
fn both_ssr_lanes_keep_empty_component_handlers_as_whole_no_value_output() {
    for lane in [SsrLane::Selected, SsrLane::LegacyOnly] {
        for is_ts in [false, true] {
            for event in ["@click", "@update:focused", "@click.stop"] {
                let baseline = compile(&cstr!("<Child {event} />"), lane, is_ts);
                assert!(baseline.contains(": () => {}"), "{baseline}");
                for value in ["\"\"", "''", "\" \t \""] {
                    let source = cstr!("<Child {event}={value} />");
                    assert_eq!(compile(&source, lane, is_ts), baseline, "{source}");
                }
                let active = compile(&cstr!("<Child {event}=\"save()\" />"), lane, is_ts);
                assert_ne!(active, baseline);
                assert!(active.contains("save()"), "{active}");
            }
        }
    }
}

#[test]
fn both_ssr_lanes_keep_custom_directive_value_padding_and_static_values() {
    for lane in [SsrLane::Selected, SsrLane::LegacyOnly] {
        for (directive, expected) in [
            (
                "v-masonry",
                "_ssrGetDirectiveProps(_ctx, _resolveDirective(\"masonry\"))",
            ),
            (
                "v-masonry:foo",
                "_ssrGetDirectiveProps(_ctx, _resolveDirective(\"masonry\"), void 0, \"foo\")",
            ),
            (
                "v-masonry.bar",
                "_ssrGetDirectiveProps(_ctx, _resolveDirective(\"masonry\"), void 0, void 0, { bar: true })",
            ),
            (
                "v-masonry:foo.bar",
                "_ssrGetDirectiveProps(_ctx, _resolveDirective(\"masonry\"), void 0, \"foo\", { bar: true })",
            ),
        ] {
            let baseline = compile(&cstr!("<div {directive} />"), lane, false);
            assert!(baseline.contains(expected), "{baseline}");
            for value in ["\"\"", "''", "\" \t \""] {
                let source = cstr!("<div {directive}={value} />");
                assert_eq!(compile(&source, lane, false), baseline, "{source}");
            }
            let explicit_string = compile(&cstr!("<div {directive}=\"''\" />"), lane, false);
            assert_ne!(explicit_string, baseline);
            assert!(
                explicit_string.contains("\"masonry\"), ''"),
                "{explicit_string}"
            );
        }
    }
}

#[test]
fn both_ssr_lanes_keep_empty_slot_parameters_as_the_whole_no_value_function() {
    for lane in [SsrLane::Selected, SsrLane::LegacyOnly] {
        for is_ts in [false, true] {
            for slot in ["#container", "v-slot:default"] {
                let baseline = compile(
                    &cstr!("<Child><template {slot}>body</template></Child>"),
                    lane,
                    is_ts,
                );
                assert!(
                    baseline.contains("_withCtx((_, _push, _parent, _scopeId) =>"),
                    "{baseline}"
                );
                for value in ["\"\"", "''", "\" \t \""] {
                    let source = cstr!("<Child><template {slot}={value}>body</template></Child>");
                    assert_eq!(compile(&source, lane, is_ts), baseline, "{source}");
                }
                let active = compile(
                    &cstr!(
                        "<Child><template {slot}=\"{{ item }}\">{{{{ item }}}}</template></Child>"
                    ),
                    lane,
                    is_ts,
                );
                assert_ne!(active, baseline);
                assert!(
                    active.contains("_withCtx(({ item }, _push, _parent, _scopeId) =>"),
                    "{active}"
                );
            }
        }
    }
}
