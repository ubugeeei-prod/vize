use super::tests::{fix_until_stable, lint};

#[test]
fn moves_leading_comments_with_their_keys() {
    let source = "\
export default defineNuxtConfig({
  // Client-only app
  ssr: false,

  // Registered modules
  modules: [\"@pinia/nuxt\"],
});";
    let expected = "\
export default defineNuxtConfig({
  // Registered modules
  modules: [\"@pinia/nuxt\"],

  // Client-only app
  ssr: false,
});";
    assert_eq!(fix_until_stable(source), expected);
    assert!(lint(&expected).diagnostics.is_empty());
}

#[test]
fn moves_trailing_comments_with_their_keys() {
    let source = "\
export default defineNuxtConfig({
  ssr: false, // client-only app
  modules: [\"@pinia/nuxt\"], // registered modules
});";
    let expected = "\
export default defineNuxtConfig({
  modules: [\"@pinia/nuxt\"], // registered modules
  ssr: false, // client-only app
});";
    assert_eq!(fix_until_stable(source), expected);
    assert!(lint(&expected).diagnostics.is_empty());
}

#[test]
fn keeps_a_leading_comment_with_the_inserted_trailing_comma() {
    let source =
        "export default defineNuxtConfig({\n  ssr: true, // ssr\n  // modules\n  modules: []\n})";
    assert_eq!(
        fix_until_stable(source),
        "export default defineNuxtConfig({\n  // modules\n  modules: [],\n  ssr: true, // ssr\n})"
    );
}

#[test]
fn reports_without_a_fix_when_a_comment_cannot_move() {
    let source = "\
export default defineNuxtConfig({
  ssr: false,

  // orphan

  modules: [],
});";
    let result = lint(source);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0].fix.is_none());
    assert_eq!(fix_until_stable(source), source);
}

#[test]
fn leaves_a_brace_line_comment_in_place() {
    let source = "\
export default defineNuxtConfig({ // keep
  ssr: false,
  modules: [],
});";
    let expected = "\
export default defineNuxtConfig({ // keep
  modules: [],
  ssr: false,
});";
    assert_eq!(fix_until_stable(source), expected);
}

#[test]
fn moves_a_multiline_block_comment_with_its_key() {
    let source = "\
export default defineNuxtConfig({
  /* client
     only */
  ssr: false,
  modules: [],
});";
    let expected = "\
export default defineNuxtConfig({
  modules: [],
  /* client
     only */
  ssr: false,
});";
    assert_eq!(fix_until_stable(source), expected);
}

#[test]
fn reports_without_a_fix_when_a_block_comment_has_a_blank_line() {
    let source = "\
export default defineNuxtConfig({
  /*
   client

   only
  */
  ssr: false,
  modules: [],
});";
    let result = lint(source);
    assert_eq!(result.diagnostics.len(), 1);
    assert!(result.diagnostics[0].fix.is_none());
    assert_eq!(fix_until_stable(source), source);
}

#[test]
fn sorts_a_url_string_without_treating_slashes_as_comments() {
    let source = "export default { ssr: \"https://example.com\", modules: [] }";
    assert_eq!(
        fix_until_stable(source),
        "export default { modules: [], ssr: \"https://example.com\", }"
    );
}
