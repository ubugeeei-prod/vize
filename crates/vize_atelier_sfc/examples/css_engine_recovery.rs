//! Executed with the actual release profile: Rust test harnesses always unwind.

use vize_atelier_sfc::{CssCompileOptions, compile_css, parse_css_ast};

const _: () = assert!(
    cfg!(panic = "unwind"),
    "native CSS recovery requires unwinding"
);

fn main() {
    let parse_error = "CSS parse error: the CSS engine hit an internal defect (upstream lightningcss panic; see vize issue #3295)";
    let compile_error = "CSS compile error: the CSS engine hit an internal defect (upstream lightningcss panic; see vize issue #3295)";
    let artifact = include_str!("../tests/fixtures/css-engine/6190.css");
    assert_eq!(artifact.len(), 112);
    for source in [
        artifact,
        "@keyframes x { cos(8) {} }",
        "a{border-image-slice:abs(-50%)}",
        "a{text-size-adjust:calc(5)}",
        "@property --x{syntax:\"<percentage>\";inherits:false;initial-value:calc(5);}",
    ] {
        let result = parse_css_ast(source, &CssCompileOptions::default());
        assert!(result.ast.is_none(), "{source:?}");
        assert_eq!(result.errors, [parse_error], "{source:?}");
        let result = compile_css(source, &CssCompileOptions::default());
        assert_eq!(result.code, source);
        assert_eq!(result.errors, [compile_error], "{source:?}");
        // The process must remain usable after a rejected input.
        let repaired = parse_css_ast(
            "@keyframes x { 50% { opacity: 0.5 } }",
            &CssCompileOptions::default(),
        );
        assert!(repaired.ast.is_some());
        assert!(repaired.errors.is_empty());
    }
    println!("CSS release recovery: five crash cases and subsequent repairs passed");
}
