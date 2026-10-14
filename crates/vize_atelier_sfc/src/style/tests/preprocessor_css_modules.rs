//! `<style module lang="scss|less|...">` compile regressions.

use crate::compile_sfc;
use crate::types::SfcCompileOptions;
use crate::{SfcParseOptions, parse_sfc};

const CARD: &str = include_str!(
    "../../../../../tests/_fixtures/differential/compiler/css-module-preprocessor-syntax/Card.vue.txt"
);

fn compile(lang: &str) -> crate::types::SfcCompileResult {
    let source = CARD.replace("lang=\"scss\"", &vize_carton::cstr!("lang=\"{lang}\""));
    let descriptor = parse_sfc(&source, SfcParseOptions::default()).expect("parse");
    let mut opts = SfcCompileOptions {
        scope_id: Some("test".into()),
        ..Default::default()
    };
    opts.script.id = Some("src/Card.vue".into());
    compile_sfc(&descriptor, opts).expect("compile")
}

#[test]
fn preprocessor_css_module_syntax_is_not_a_css_warning() {
    for lang in ["scss", "sass", "less", "stylus", "styl"] {
        let result = compile(lang);
        assert!(result.errors.is_empty(), "{lang}: {:?}", result.errors);
        assert!(result.warnings.is_empty(), "{lang}: {:?}", result.warnings);
        let css = result.css.unwrap_or_default();
        assert!(
            css.contains("$card-padding: var(--spacing--sm);"),
            "{lang}: {css}"
        );
    }
}

#[test]
fn plain_css_module_parse_failure_still_warns() {
    let result = compile("css");
    assert!(
        result
            .warnings
            .iter()
            .any(|warning| warning.message.starts_with("CSS parse error")),
        "{:?}",
        result.warnings
    );
}
