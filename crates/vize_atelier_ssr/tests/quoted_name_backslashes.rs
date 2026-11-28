//! A name written between quotes (a bound attribute, a prop or event key, a
//! slot name) is spelled as a JavaScript string literal on both SSR lanes. A
//! backslash in a name is text: left alone, `"e\f"` would name `e` + form feed
//! and `"onT\u"` would not parse. A double quote in a directive name is
//! reported, as Vue reports it, and no module is written.

use vize_atelier_ssr::differential::compare_ssr_lanes;
use vize_atelier_ssr::{SsrCompilerExperimentalOptions, SsrCompilerOptions};

const CASES: &[(&str, &str)] = &[
    (
        r#"<section><p c\d="1" :e\f="value">text</p></section>"#,
        r#"function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(`<section${_ssrRenderAttrs(_attrs)}><p c\\d="1"${_ssrRenderAttr("e\\f", _ctx.value)}>text</p></section>`)
}
"#,
    ),
    (
        r#"<Card p\q="1" :r\s="value" @t\u="onPick"><template #s\t>slot</template></Card>"#,
        r#"function ssrRender(_ctx, _push, _parent, _attrs) {
  _push(_ssrRenderComponent(_resolveComponent("Card"), _mergeProps({ "p\\q": "1", "r\\s": _ctx.value, "onT\\u": _ctx.onPick }, _attrs), {
    "s\\t": _withCtx((_, _push, _parent, _scopeId) => {
      if (_push) {
        _push(`slot`)
      } else {
        return [_createTextVNode("slot")]
      }
    }),
    _: 1
  }, _parent))
}
"#,
    ),
];

#[test]
fn quoted_names_are_escaped_on_both_lanes() {
    let options = SsrCompilerOptions::default();
    let experimental = SsrCompilerExperimentalOptions::default();
    for (source, expected) in CASES {
        let lanes = compare_ssr_lanes(source, &options, &experimental);
        assert!(
            lanes.selected.errors.is_empty(),
            "{source}: {:?}",
            lanes.selected.errors
        );
        assert!(
            lanes.legacy.errors.is_empty(),
            "{source}: {:?}",
            lanes.legacy.errors
        );
        assert_eq!(
            [
                lanes.selected.result.code.as_str(),
                lanes.legacy.result.code.as_str()
            ],
            [*expected, *expected],
            "{source} ({})",
            lanes.lane
        );
    }
}

const QUOTE_IN_NAME: &str = "Attribute name contains an invalid character; inferred the nearest attribute boundary and continued.";

#[test]
fn a_quote_in_a_directive_name_is_reported_on_both_lanes() {
    let options = SsrCompilerOptions::default();
    let experimental = SsrCompilerExperimentalOptions::default();
    for source in [
        r#"<section><div :a"b="v"></div></section>"#,
        r#"<div v-x:a"b="v"></div>"#,
        r#"<div v-x.a"b="v"></div>"#,
        r#"<Comp @a"b="v" />"#,
    ] {
        let lanes = compare_ssr_lanes(source, &options, &experimental);
        for lane in [&lanes.selected, &lanes.legacy] {
            let messages: Vec<_> = lane.errors.iter().map(ToString::to_string).collect();
            assert_eq!(messages, [QUOTE_IN_NAME], "{source}");
            assert_eq!(
                lane.result.code.as_str(),
                "",
                "{source}: no module is written"
            );
        }
    }
}
