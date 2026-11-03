use oxc_allocator::Allocator;
use oxc_codegen::Codegen;
use oxc_parser::Parser;
use oxc_span::SourceType;
use serde_json::Value;

/// Advance an unmapped archived Result of a TypeScript source by its reviewed
/// per-piece erasure: the module is no longer reprinted whole, so its bytes
/// change while its JavaScript keeps every archived token. Every other public
/// Result field stays immutable.
#[track_caller]
pub fn advance(mut reference: Value, actual: &Value) -> Value {
    let result = &mut reference["Ok"];
    assert_eq!(
        result["map"],
        Value::Null,
        "mapped archives require an explicit coordinate receipt"
    );
    let archived = result["code"].as_str().expect("archived complete module");
    let erased = actual["Ok"]["code"].as_str().expect("complete module");
    assert_eq!(
        reprint(erased),
        reprint(archived),
        "the erased module must keep the archived JavaScript tokens"
    );
    result["code"] = Value::from(erased);
    reference
}

#[track_caller]
fn reprint(code: &str) -> std::string::String {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, code, SourceType::mjs()).parse();
    assert_eq!(parsed.diagnostics.len(), 0, "{code}");
    Codegen::new().build(&parsed.program).code
}
