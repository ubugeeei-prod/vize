use serde_json::Value;

/// Advance an unmapped archived Result by its explicitly reviewed #10891 edit.
/// Keep measured archive files and every other public Result field immutable.
#[track_caller]
pub fn advance(mut reference: Value, before: &str, after: &str) -> Value {
    let result = &mut reference["Ok"];
    assert_eq!(
        result["map"],
        Value::Null,
        "mapped archives require an explicit coordinate receipt"
    );
    let code = result["code"].as_str().expect("archived complete module");
    assert_eq!(
        code.matches(before).count(),
        1,
        "the reviewed setup signature must occur exactly once"
    );
    result["code"] = Value::String(code.replacen(before, after, 1));
    reference
}
