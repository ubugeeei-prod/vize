#![no_main]

// JS/TS expression parser fuzz target.
//
// Drives the same OXC expression parsing path used by template expression
// transforms and import-usage checks. Invalid expressions are expected and
// reported as parser errors; panics are not.
use libfuzzer_sys::fuzz_target;
use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };
    if exceeds_expression_nesting_limit(source, 256) {
        return;
    }

    let allocator = Allocator::default();
    let parser = Parser::new(
        &allocator,
        source,
        SourceType::default().with_module(true).with_typescript(true),
    );
    let _ = parser.parse_expression();
});

fn exceeds_expression_nesting_limit(source: &str, limit: usize) -> bool {
    let mut depth = 0usize;
    for byte in source.bytes() {
        match byte {
            b'(' | b'[' | b'{' => {
                depth += 1;
                if depth > limit {
                    return true;
                }
            }
            b')' | b']' | b'}' => depth = depth.saturating_sub(1),
            _ => {}
        }
    }
    false
}
