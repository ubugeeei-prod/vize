//! Execute complete finite repaired-original contracts with the actual release profile.
//! The entire former panic-only example remains frozen in the vendor-exit fixture.

use serde_json::{Value, json};
use vize_atelier_sfc::{CssCompileOptions, compile_css, parse_css_ast, print_css_ast};

const _: () = assert!(
    cfg!(panic = "unwind"),
    "native CSS recovery requires unwinding"
);
const AUTHORITY: &str = include_str!(
    "../../../tests/_fixtures/differential/css-engine-vendor-exit/finite-successors.json"
);

fn main() {
    let authority: Value = serde_json::from_str(AUTHORITY).unwrap();
    assert_eq!(authority["additionalAtelier"].as_array().unwrap().len(), 6);
    let artifact = include_str!("../tests/fixtures/css-engine/6190.css");
    assert_eq!(artifact.len(), 112);
    let options = CssCompileOptions::default();
    for (id, source) in [
        ("fuzz-recovery", artifact),
        ("release-keyframes", "@keyframes x { cos(8) {} }"),
        ("release-border-image", "a{border-image-slice:abs(-50%)}"),
        ("release-number", "a{text-size-adjust:calc(5)}"),
        (
            "release-property",
            r#"@property --x{syntax:"<percentage>";inherits:false;initial-value:calc(5);}"#,
        ),
    ] {
        let mut rows = authority["additionalAtelier"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|row| row["input"]["id"] == id);
        let expected = rows.next().expect("one finite original release input");
        assert!(rows.next().is_none());
        assert_eq!(
            expected["input"]["source"], source,
            "whole unchanged original input"
        );
        let parsed = parse_css_ast(source, &options);
        let printed = parsed.ast.clone().map(|ast| print_css_ast(ast, &options));
        assert_eq!(
            json!({"parsed": parsed, "compiled": compile_css(source, &options),
            "printed": printed}),
            expected["wholeExpected"]
        );
        assert_eq!(
            json!(parse_css_ast(
                "@keyframes x { 50% { opacity: 0.5 } }",
                &options
            )),
            authority["subsequentHealthy"],
            "whole subsequent healthy AST"
        );
    }
    println!(
        "CSS release finite contract: five original inputs and subsequent healthy ASTs passed"
    );
}
