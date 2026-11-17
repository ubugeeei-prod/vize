//! A declaration hoisted to module scope keeps the JSDoc that documents it
//! (#12985): `@deprecated` and documentation stay on the declaration, and the
//! setup body no longer hands the comment to the next statement.

use super::{leading_jsdoc_len, with_leading_jsdoc};
use crate::sfc_typecheck::{SfcTypeCheckOptions, type_check_sfc};

/// The SFC reported in #12985.
const ALIAS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/hoisted-jsdoc-12985/src/Alias.vue.txt"
));
/// typescript-eslint's plain-script `no-deprecated` cases, merged.
const PLAIN: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/hoisted-jsdoc-12985/src/Plain.vue.txt"
));

fn virtual_ts(source: &str) -> vize_carton::String {
    let options = SfcTypeCheckOptions::new("test.vue").with_virtual_ts();
    type_check_sfc(source, &options)
        .virtual_ts
        .unwrap_or_default()
}

/// The generated module scope and setup body of `source`.
fn scopes(source: &str) -> (vize_carton::String, vize_carton::String) {
    let generated = virtual_ts(source);
    let (module, rest) = generated
        .split_once("// ========== Setup Scope ==========")
        .expect("setup scope marker");
    let setup = rest
        .split_once("// User setup code")
        .map_or(rest, |(_, body)| body);
    let setup = setup
        .split_once("// @vize-map")
        .map_or(setup, |(body, _)| body);
    (module.into(), setup.into())
}

/// The line right before the line that reads `line`, both trimmed.
fn line_before<'a>(text: &'a str, line: &str) -> Option<&'a str> {
    let lines = text.lines().map(str::trim).collect::<Vec<_>>();
    let index = lines.iter().position(|candidate| *candidate == line)?;
    lines.get(index.checked_sub(1)?).copied()
}

fn occurrences(text: &str, needle: &str) -> usize {
    text.matches(needle).count()
}

fn span_of(script: &str, needle: &str) -> (u32, u32) {
    let start = script.find(needle).expect("needle") as u32;
    (start, start + needle.len() as u32)
}

#[test]
fn widens_over_whole_line_jsdoc_blocks() {
    let script = "const a = 1;\n/** one */\n  /**\n   * @deprecated two\n   */\ntype A = string;\n";
    let span = span_of(script, "type A = string;");
    let widened = with_leading_jsdoc(script, span);
    assert_eq!(
        script.get(widened.0 as usize..widened.1 as usize),
        Some("/** one */\n  /**\n   * @deprecated two\n   */\ntype A = string;")
    );
}

#[test]
fn leaves_plain_and_shared_line_comments_in_place() {
    for script in [
        "/* block */\ntype A = string;\n",
        "// line\ntype A = string;\n",
        "const a = 1; /** shared */\ntype A = string;\n",
        "/**/\ntype A = string;\n",
    ] {
        let span = span_of(script, "type A = string;");
        assert_eq!(with_leading_jsdoc(script, span), span, "{script:?}");
    }
}

#[test]
fn measures_the_leading_jsdoc_of_a_hoisted_text() {
    let text = "/** a */\n/** b */\ntype A<T> = T;";
    assert_eq!(text.get(leading_jsdoc_len(text)..), Some("type A<T> = T;"));
    assert_eq!(leading_jsdoc_len("type A = string;"), 0);
}

#[test]
fn setup_type_alias_moves_with_its_jsdoc() {
    let (module, setup) = scopes(ALIAS);
    assert_eq!(
        line_before(&module, "type Old = { a: number };"),
        Some("/** @deprecated use Next */"),
        "{module}"
    );
    assert_eq!(occurrences(&setup, "@deprecated"), 0, "{setup}");
    assert_eq!(occurrences(&setup, "const first = 1;"), 1, "{setup}");
}

#[test]
fn plain_script_declarations_move_with_their_jsdoc() {
    let (module, setup) = scopes(PLAIN);
    assert_eq!(
        line_before(&module, "declare const a: A;"),
        Some("/** @deprecated */"),
        "{module}"
    );
    assert_eq!(
        line_before(&module, "interface Foo {}"),
        Some("/** @deprecated */"),
        "{module}"
    );
    assert_eq!(occurrences(&setup, "@deprecated"), 0, "{setup}");
}

#[test]
fn generic_injection_keeps_the_jsdoc_ahead_of_the_parameters() {
    let (module, _) = scopes(
        r#"<script setup lang="ts" generic="T">
/** Rows of T. */
type Rows = T[];
defineProps<{ rows: Rows }>();
</script>
"#,
    );
    let rows = module
        .lines()
        .find(|line| {
            line.trim_start()
                .split_whitespace()
                .take(2)
                .eq(["type", "Rows<T"])
        })
        .map(str::trim);
    assert_eq!(
        rows.and_then(|rows| line_before(&module, rows)),
        Some("/** Rows of T. */"),
        "{module}"
    );
}
