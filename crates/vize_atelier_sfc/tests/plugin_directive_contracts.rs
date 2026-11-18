//! `vue.plugins` directive contracts in every template output (#13074).
//!
//! A declared directive's absent modifiers that default to `true` reach the
//! VDOM, SSR and Vapor runtime calls, and a binding that breaks its contract
//! fails the compile on its authored span. An empty table changes nothing.
#![expect(
    clippy::disallowed_macros,
    clippy::expect_used,
    reason = "tests assert by panicking; insta and fixtures use format!"
)]

use vize_atelier_sfc::{SfcCompileOptions, SfcParseOptions, compile_sfc, parse_sfc};
use vize_l0::config::{DirectiveContract, DirectiveContracts, DirectiveModifier};

const SOURCE: &str = r#"<script setup>
const payload = { id: 1 }
</script>

<template>
  <a v-track.strict="payload">go</a>
  <b v-track:click.once.strict.passive>x</b>
  <i v-focus.x>y</i>
</template>"#;

fn tracking() -> DirectiveContracts {
    let modifier = |name: &str, optional, default| DirectiveModifier {
        name: name.into(),
        optional,
        default,
    };
    DirectiveContracts::new(vec![DirectiveContract {
        plugin: "@acme/tracking".into(),
        name: "track".into(),
        modifiers: vec![
            modifier("once", true, false),
            modifier("passive", true, true),
            modifier("strict", false, false),
        ],
    }])
}

fn options(ssr: bool, vapor: bool, contracts: DirectiveContracts) -> SfcCompileOptions {
    let mut options = SfcCompileOptions::default();
    options.template.ssr = ssr;
    options.vapor = vapor;
    options.template.compiler_options = Some(vize_atelier_dom::DomCompilerOptions {
        directive_contracts: contracts,
        ..Default::default()
    });
    options
}

fn compile(source: &str, options: SfcCompileOptions) -> vize_atelier_sfc::SfcCompileResult {
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse SFC");
    compile_sfc(&descriptor, options).expect("compile SFC")
}

/// The runtime directive calls of one output, one per line.
fn directive_calls(code: &str) -> Vec<&str> {
    code.lines()
        .map(str::trim)
        .filter(|line| {
            line.starts_with("[_directive_")
                || line.starts_with("_ssrGetDirectiveProps(")
                || line.contains("_ssrGetDirectiveProps(")
                || line.starts_with("_withVaporDirectives(")
        })
        .collect()
}

#[test]
fn vdom_output_carries_declared_default_modifiers() {
    let result = compile(SOURCE, options(false, false, tracking()));
    assert_eq!(result.errors.len(), 0, "{:?}", result.errors);
    insta::assert_snapshot!(
        "vdom_default_modifiers",
        directive_calls(&result.code).join("\n")
    );
}

#[test]
fn ssr_output_carries_declared_default_modifiers() {
    let result = compile(SOURCE, options(true, false, tracking()));
    assert_eq!(result.errors.len(), 0, "{:?}", result.errors);
    insta::assert_snapshot!(
        "ssr_default_modifiers",
        directive_calls(&result.code).join("\n")
    );
}

#[test]
fn vapor_output_carries_declared_default_modifiers() {
    let result = compile(SOURCE, options(false, true, tracking()));
    assert_eq!(result.errors.len(), 0, "{:?}", result.errors);
    insta::assert_snapshot!(
        "vapor_default_modifiers",
        directive_calls(&result.code).join("\n")
    );
}

#[test]
fn a_binding_that_breaks_its_contract_fails_every_output() {
    let source =
        "<template>\n  <a v-track.bogus.strict=\"e\">go</a>\n  <b v-track>x</b>\n</template>\n";
    let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse SFC");
    for (ssr, vapor) in [(false, false), (true, false), (false, true)] {
        let error = compile_sfc(&descriptor, options(ssr, vapor, tracking()))
            .expect_err("a broken binding fails the template");
        insta::assert_snapshot!(
            format!("contract_errors_ssr_{ssr}_vapor_{vapor}"),
            format!("{:?}\n{}", error.code, error.message)
        );
    }
}

#[test]
fn an_empty_table_compiles_byte_identically() {
    for (ssr, vapor) in [(false, false), (true, false), (false, true)] {
        let without = compile(SOURCE, options(ssr, vapor, DirectiveContracts::EMPTY));
        let unrelated = DirectiveContracts::new(vec![DirectiveContract {
            plugin: "@acme/other".into(),
            name: "other".into(),
            modifiers: vec![DirectiveModifier {
                name: "always".into(),
                optional: true,
                default: true,
            }],
        }]);
        let with_unrelated = compile(SOURCE, options(ssr, vapor, unrelated));
        let default = compile(SOURCE, {
            let mut options = SfcCompileOptions::default();
            options.template.ssr = ssr;
            options.vapor = vapor;
            options
        });
        assert_eq!(without.code, default.code, "ssr={ssr} vapor={vapor}");
        assert_eq!(with_unrelated.code, default.code, "ssr={ssr} vapor={vapor}");
    }
}

#[test]
fn a_file_compiles_only_to_an_output_its_plugins_support() {
    let vdom_only = DirectiveContracts::with_outputs(
        tracking().as_slice().to_vec(),
        vec![("@acme/tracking".into(), vec!["vdom".into()])],
    );
    // A `vapor` script attribute selects Vapor without any request flag.
    let vapor_source = "<script setup vapor>\nconst e = 1\n</script>\n<template><a v-track.strict=\"e\">go</a></template>\n";
    let refusals: Vec<_> = [(vapor_source, false), (SOURCE, true)]
        .into_iter()
        .map(|(source, ssr)| {
            let descriptor = parse_sfc(source, SfcParseOptions::default()).expect("parse SFC");
            // A script-bearing SFC reports template errors in its result.
            let error = match compile_sfc(&descriptor, options(ssr, false, vdom_only.clone())) {
                Err(error) => error,
                Ok(result) => result
                    .errors
                    .into_iter()
                    .next()
                    .expect("an unsupported output fails the template"),
            };
            (
                error.code.map(|code| code.to_string()),
                error.message.to_string(),
            )
        })
        .collect();
    assert_eq!(
        refusals,
        [
            (
                Some("PLUGIN_OUTPUT_UNSUPPORTED".to_owned()),
                "[plugin/host-unsupported] @acme/tracking: vue.directive `track` does not support the vapor output".to_owned(),
            ),
            (
                Some("PLUGIN_OUTPUT_UNSUPPORTED".to_owned()),
                "[plugin/host-unsupported] @acme/tracking: vue.directive `track` does not support the ssr output".to_owned(),
            ),
        ]
    );
    assert_eq!(
        compile(SOURCE, options(false, false, vdom_only))
            .errors
            .len(),
        0
    );
}
