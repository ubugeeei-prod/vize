use super::extract_module_specifier_occurrences;
use vize_canon::PackageResolutionMode::{Contextual, Import, Require};

#[test]
fn preserves_import_and_require_occurrence_modes() {
    let source = r#"
import direct from "direct"
export { named } from 'exported'
const dynamic = import ( "dynamic" )
const commonjs = require('commonjs')
import legacy = require("import-equals")
"#;
    let actual = extract_module_specifier_occurrences(source)
        .into_iter()
        .map(|occurrence| (occurrence.specifier.to_string(), occurrence.mode))
        .collect::<Vec<_>>();

    assert_eq!(
        actual,
        vec![
            ("direct".to_owned(), Contextual),
            ("exported".to_owned(), Contextual),
            ("dynamic".to_owned(), Import),
            ("commonjs".to_owned(), Require),
            ("import-equals".to_owned(), Require),
        ]
    );
}

#[test]
fn explicit_resolution_mode_attributes_override_import_occurrences() {
    let source = r#"
import type { Static } from "static-package" with { "resolution-mode": "require" }
type Dynamic = import("dynamic-package", { with: { "resolution-mode": "require" } })
import type { Native } from "import-package" with { "resolution-mode": "import" }
"#;
    let actual = extract_module_specifier_occurrences(source)
        .into_iter()
        .filter(|occurrence| occurrence.specifier.contains("package"))
        .map(|occurrence| (occurrence.specifier.to_string(), occurrence.mode))
        .collect::<Vec<_>>();

    assert_eq!(
        actual,
        vec![
            ("static-package".to_owned(), Require),
            ("dynamic-package".to_owned(), Require),
            ("import-package".to_owned(), Import),
        ]
    );
}

#[test]
fn comments_inside_import_syntax_do_not_hide_specifiers() {
    let source = r#"
const view = import(
  /* webpackChunkName: "profile" */
  "dynamic-package",
  /* import attributes may be split across build-tool comments */
  {
    with: {
      /* keep package resolution parity with TypeScript */
      "resolution-mode": /* inline */ "require",
    },
  },
)
const commonjs = require(
  /* real-world generated helper comment */
  "commonjs-package"
)
export { named } from /* generated barrel comment */ "reexported-package"
import type { Static } from "static-package" /* bundler hint */ with /* mode */ {
  "resolution-mode": "require"
}
"#;
    let actual = extract_module_specifier_occurrences(source)
        .into_iter()
        .map(|occurrence| (occurrence.specifier.to_string(), occurrence.mode))
        .collect::<Vec<_>>();

    assert_eq!(
        actual,
        vec![
            ("dynamic-package".to_owned(), Require),
            ("commonjs-package".to_owned(), Require),
            ("reexported-package".to_owned(), Contextual),
            ("static-package".to_owned(), Require),
        ]
    );
}

#[test]
fn keywords_must_be_standalone_identifier_tokens() {
    let source = r#"
const imported = "ignored-one"
const requirement = "ignored-two"
import value from "kept"
"#;
    let actual = extract_module_specifier_occurrences(source);

    assert_eq!(actual.len(), 1);
    assert_eq!(actual[0].specifier.as_str(), "kept");
    assert_eq!(actual[0].mode, Contextual);
}

#[test]
fn keywords_inside_strings_and_comments_cannot_consume_later_imports() {
    let source = r#"
const words = "require import from"
// require("ignored-comment")
/* import("ignored-block") */
import("kept-dynamic", { with: { "resolution-mode": "require" } })
import type { Kept } from "kept-static" with { "resolution-mode": "import" }
"#;
    let actual = extract_module_specifier_occurrences(source)
        .into_iter()
        .map(|occurrence| (occurrence.specifier.to_string(), occurrence.mode))
        .collect::<Vec<_>>();

    assert_eq!(
        actual,
        vec![
            ("kept-dynamic".to_owned(), Require),
            ("kept-static".to_owned(), Import),
        ]
    );
}
