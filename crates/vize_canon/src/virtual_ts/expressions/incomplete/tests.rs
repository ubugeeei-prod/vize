use super::isolate_incomplete_expression;

#[test]
fn trailing_member_dot_keeps_a_completion_position() {
    for (source, emitted, mapped) in [
        ("foo.", "foo. x", "foo."),
        ("foo.bar.", "foo.bar. x", "foo.bar."),
        ("foo?.", "foo?. x", "foo?."),
        ("getApi().", "getApi(). x", "getApi()."),
        ("(foo.)", "(foo. x)", "foo."),
        ("((foo.))", "((foo. x))", "foo."),
    ] {
        let isolated = isolate_incomplete_expression(source);
        assert_eq!(isolated.as_str(), emitted, "{source}");
        assert_eq!(isolated.mapped_str(), mapped, "{source}");
    }
}

#[test]
fn valid_numeric_literal_and_non_null_assertion_stay() {
    assert_eq!(isolate_incomplete_expression("1.").as_str(), "1.");
    assert_eq!(isolate_incomplete_expression("foo!").as_str(), "foo!");
    assert_eq!(isolate_incomplete_expression("foo.bar").as_str(), "foo.bar");
    assert_eq!(isolate_incomplete_expression("(1.)").as_str(), "(1.)");
    assert_eq!(isolate_incomplete_expression("foo()").as_str(), "foo()");
}

#[test]
fn unclosed_call_drops_the_incomplete_argument() {
    assert_eq!(isolate_incomplete_expression("foo(bar").as_str(), "foo");
    assert_eq!(isolate_incomplete_expression("foo(").as_str(), "foo");
    assert_eq!(isolate_incomplete_expression("foo..").as_str(), "foo");
}
