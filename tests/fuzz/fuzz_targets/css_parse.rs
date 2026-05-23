#![no_main]

// CSS parser fuzz target.
//
// Exercises the vize_atelier_sfc Lightning CSS integration through the public
// serialized-AST API. Syntax errors should be returned in CssAstResult; panics
// are always a bug in the integration layer or upstream parser boundary.
use libfuzzer_sys::fuzz_target;
use vize_atelier_sfc::{CssCompileOptions, parse_css_ast};

fuzz_target!(|data: &[u8]| {
    let Ok(source) = std::str::from_utf8(data) else {
        return;
    };

    let _ = parse_css_ast(source, &CssCompileOptions::default());
});
