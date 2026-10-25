//! The shared evaluator consumes original ASTs and preserves its refusal frontier.
#![expect(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    reason = "independent fixture assertions"
)]

use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::Allocator;

#[test]
fn original_literal_ast_has_canonical_object_and_css_semantics() {
    for (source, expected) in [
        ("{fontWeight:'700'}", Some("font-weight:700;")),
        (
            "{color:'red',fontWeight:'700',color:'blue'}",
            Some("color:blue;font-weight:700;"),
        ),
        (
            "{fontWeight:'700','font-weight':'400'}",
            Some("font-weight:700;font-weight:400;"),
        ),
        (
            "{'--Brand':'a&b<>','font-Weight':'700'}",
            Some("--Brand:a&b<>;font-weight:700;"),
        ),
        (
            "{font\\u0057eight:'7\\x30\\u0030'}",
            Some("font-weight:700;"),
        ),
        ("{content:'\\\"日😀&<>\\\"'}", None),
        ("{color:'red!important'}", Some("color:red!important;")),
        ("{color:'red !important'}", Some("color:red !important;")),
        ("{color:'red!important '}", None),
        ("{color:'red !IMPORTANT'}", None),
        ("{color:'red ! important'}", None),
        ("{color:'red!important!important'}", None),
        ("{color:'red\\u00a0!important'}", None),
        ("{color:'red\\ufeff!important'}", None),
        ("{color:'red\\u2003!important'}", None),
        ("{color:'!important'}", None),
        ("{color:'  !important'}", None),
        ("{'--Brand':'blue!important'}", None),
        ("{'--Brand':'blue!important '}", None),
        ("{'--Brand':'!important'}", None),
        ("{'--Brand':'  !important'}", None),
        ("{fontWeight:'700',color:'red !IMPORTANT'}", None),
        ("{color:'var(--missing',fontWeight:'700'}", None),
        (r#"{fontFamily:'"broken',fontWeight:'700'}"#, None),
        (r#"{color:'red\\',fontWeight:'700'}"#, None),
        ("{color:'var(--valid)'}", None),
        ("{cssFloat:'left'}", None),
        ("{cssText:'color:red'}", None),
        ("{Color:'red'}", None),
        ("{WebkitTransform:'none'}", None),
        ("{webkitTransform:'none'}", None),
        ("{msTransform:'none'}", None),
        ("{setProperty:'red'}", None),
        ("{toString:'red'}", None),
        ("{parentRule:'red'}", None),
        ("{}", None),
        ("{color:''}", None),
        ("{color:0}", None),
        ("{color:null}", None),
        ("{color:undefined}", None),
        ("{color:true}", None),
        ("{color:shade}", None),
        ("{color:read()}", None),
        ("{color:`red`}", None),
        ("{...other,color:'red'}", None),
        ("{['color']:'red'}", None),
        ("{color}", None),
        ("{get color(){return 'red'}}", None),
        ("{color(){return 'red'}}", None),
        ("{__proto__:'red'}", None),
        ("{'__proto__':'red'}", None),
        ("{constructor:'red'}", None),
        ("{prototype:'red'}", None),
        ("{0:'red'}", None),
        ("{'1':'red'}", None),
        ("{color:'red;display:none'}", None),
        ("{color:'/*x*/red'}", None),
        ("{content:'\\u0000'}", None),
        ("{content:'\\ud800'}", None),
        ("{'--':'red'}", None),
        ("{'bad:key':'red'}", None),
    ] {
        let allocator = Allocator::new();
        let expression = Parser::new(allocator.as_oxc(), source, SourceType::tsx())
            .parse_expression()
            .unwrap();
        assert_eq!(
            vize_relief::literal_object_style(&expression).as_deref(),
            expected,
            "{source}"
        );
    }
}
