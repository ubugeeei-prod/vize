//! Scope discipline of literal-const hoisting (#3944).

use super::compile_script_setup_inline;
use crate::compile_script::TemplateParts;

fn compile(source: &str) -> vize_carton::String {
    compile_script_setup_inline(
        source,
        "ConstScope",
        false,
        true,
        false,
        TemplateParts {
            imports: "",
            hoisted: "",
            render_fn: "",
            render_fn_name: "",
            preamble: "",
            render_body: "null",
            render_is_block: false,
        },
        None,
        &[],
        "",
        None,
    )
    .map(|result| result.code)
    .unwrap()
}

#[test]
fn a_function_local_shadow_of_a_hoisted_const_stays_in_its_function() {
    // The #3944 reproducer: the top-level literal `const max = 7` hoists to
    // module scope, but the line-matched scan also ripped the function-local
    // `const max = …` out of `hover()` — a duplicate module-scope declaration
    // referencing setup bindings that do not exist there.
    let source = r#"
import { ref } from 'vue'

const fetched = ref({ width: 5 })

function hover() {
    const max = fetched.value.width
    return max
}

const max = 7
"#;
    let output = compile(source);
    let component = output.find("export default").expect("component wrapper");

    // Exactly one `const max` before the component: the top-level literal.
    let hoisted_region = &output[..component];
    assert_eq!(
        hoisted_region.matches("const max").count(),
        1,
        "only the top-level literal hoists:\n{output}"
    );
    assert!(
        hoisted_region.contains("const max = 7"),
        "the hoisted one is the literal:\n{output}"
    );
    assert!(
        !hoisted_region.contains("fetched.value"),
        "no setup-scope reference may leak to module scope:\n{output}"
    );

    // The local declaration is still inside the function body.
    let body_region = &output[component..];
    assert!(
        body_region.contains("const max = fetched.value.width"),
        "the function keeps its local declaration:\n{output}"
    );
}

#[test]
fn a_setup_that_only_parses_with_recovery_hoists_nothing() {
    // oxc reports a syntax error either by setting `panicked` or by returning a
    // recovered program with diagnostics attached. In the recovered case the
    // statements it salvaged around the error are not a trustworthy basis for
    // moving a declaration out of setup scope, so nothing hoists.
    let source = r#"
const label = 'plain'
function () {}
"#;
    let output = compile(source);
    let component = output.find("export default").expect("component wrapper");
    assert!(
        !output[..component].contains("const label"),
        "a setup with a syntax error hoists nothing:\n{output}"
    );
}

#[test]
fn non_literal_and_multi_declarator_consts_stay_in_setup() {
    let source = r#"
const label = 'plain'
const computed_like = label + '!'
const a = 1, b = 2
"#;
    let output = compile(source);
    let component = output.find("export default").expect("component wrapper");
    let hoisted_region = &output[..component];
    assert!(
        hoisted_region.contains("const label") && hoisted_region.contains("plain"),
        "the string literal hoists (quotes may normalize):\n{output}"
    );
    assert!(
        !hoisted_region.contains("computed_like"),
        "computed initializers stay in setup:\n{output}"
    );
    assert!(
        !hoisted_region.contains("const a"),
        "multi-declarator statements stay in setup:\n{output}"
    );

    // Not hoisting is only half the contract: the declarations must still be
    // emitted inside setup rather than dropped.
    let body_region = &output[component..];
    assert!(
        body_region.contains("computed_like"),
        "computed initializers remain in setup:\n{output}"
    );
    assert!(
        body_region.contains("const a"),
        "multi-declarator statements remain in setup:\n{output}"
    );
}
