#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_methods, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
#![expect(clippy::string_slice, reason = "tests assert by panicking")]

use serde_json::{Value, json};

#[path = "support/lsp_authored_rename.rs"]
mod authored;
#[path = "support/lsp_process.rs"]
mod lsp_process;
#[path = "support/lsp_vue_project.rs"]
mod support;
use authored::{capture, edit, expected, location, observe, project};

macro_rules! source {
    ($name:literal) => {
        include_str!(concat!(
            "../../../tests/_fixtures/differential/lsp/type-alias-navigation/8011/supplemental/",
            $name,
            ".vue.txt"
        ))
    };
}

const ORIGINAL: &str =
    include_str!("../../../tests/_fixtures/differential/lsp/type-alias-navigation/8011/E.vue.txt");
const GUARD: &str = "<script setup lang=\"ts\">\nconst nativeGuard: number = 'wrong';\n</script>\n<template>{{ nativeGuard }}</template>\n";
const REPAIRED_GUARD: &str = "<script setup lang=\"ts\">\nconst nativeGuard: number = 1;\n</script>\n<template>{{ nativeGuard }}</template>\n";

#[derive(Clone, Copy, Debug)]
enum Form {
    Plain,
    Defaults,
    Partial,
}

impl Form {
    fn input(self) -> &'static str {
        match self {
            Self::Plain => ORIGINAL,
            Self::Defaults => source!("E-defaults"),
            Self::Partial => source!("E-partial"),
        }
    }

    fn repaired(self, name: &str) -> &'static str {
        match (self, name) {
            (Self::Plain, "hidden") => source!("E-plain-hidden"),
            (Self::Plain, "tone") => source!("E-plain-tone"),
            (Self::Defaults, "hidden") => source!("E-defaults-hidden"),
            (Self::Defaults, "tone") => source!("E-defaults-tone"),
            (Self::Partial, "hidden") => source!("E-partial-hidden"),
            (Self::Partial, "tone") => source!("E-partial-tone"),
            _ => unreachable!("authored property"),
        }
    }

    fn needles(self, name: &str) -> Vec<&'static str> {
        match (self, name) {
            (_, "hidden") => vec!["hidden?:", "hidden\""],
            (Self::Defaults, "tone") => vec!["tone?:", "tone: \"light\"", "tone\""],
            (_, "tone") => vec!["tone?:", "tone\""],
            _ => unreachable!("authored property"),
        }
    }
}

#[test]
fn original_alias_has_complete_transactions_from_both_property_origins() {
    assert_transactions(Form::Plain);
}

#[test]
fn defaulted_alias_has_complete_transactions_from_member_default_and_template() {
    assert_transactions(Form::Defaults);
}

#[test]
fn partially_destructured_alias_keeps_both_independent_bare_prop_transactions() {
    assert_transactions(Form::Partial);
}

#[test]
fn original_alias_forms_define_each_property_at_its_exact_authored_member() {
    let mut wanted = Vec::new();
    let mut observed = Vec::new();
    for form in [Form::Plain, Form::Defaults, Form::Partial] {
        for newline in ["\n", "\r\n"] {
            let files = [("E.vue", form.input().replace('\n', newline))];
            let (mut fixture, [uri]) = project(&files);
            let guard = prove_native(&mut fixture);
            let initial = fixture.open(&files[0].1);
            let mut queries = Vec::new();
            let mut answers = Vec::new();
            for name in ["hidden", "tone"] {
                let needles = form.needles(name);
                for query in &needles {
                    queries.push(json!({"name":name,"query":query,"definition":
                        location(&uri, &files[0].1, needles[0], name.len())}));
                    answers.push(json!({"name":name,"query":query,"definition":
                        fixture.request("textDocument/definition", &files[0].1, query)}));
                }
            }
            let context = format!("alias definitions {form:?}, newline={newline:?}");
            let expected = json!({"nativeGuard":expected_guard(),"initialDiagnostics":[],"definitions":queries});
            fixture.shutdown();
            let actual =
                json!({"nativeGuard":guard,"initialDiagnostics":initial,"definitions":answers});
            capture(&fixture, &files, &context, &expected, &actual);
            wanted.push(expected);
            observed.push(actual);
        }
    }
    assert_eq!(observed, wanted, "complete authored alias definitions");
}

fn assert_transactions(form: Form) {
    let mut wanted = Vec::new();
    let mut observed = Vec::new();
    for newline in ["\n", "\r\n"] {
        for (name, replacement) in [("hidden", "concealed"), ("tone", "palette")] {
            let files = [("E.vue", form.input().replace('\n', newline))];
            let goldens = [("E.vue", form.repaired(name).replace('\n', newline))];
            let needles = form.needles(name);
            for query in &needles {
                let (mut fixture, [uri]) = project(&files);
                let references = json!(
                    needles
                        .iter()
                        .map(|needle| location(&uri, &files[0].1, needle, name.len()))
                        .collect::<Vec<_>>()
                );
                let rename = json!({"changes":{&uri:needles.iter().map(|needle|
                    edit(&files[0].1, needle, name.len(), replacement)).collect::<Vec<_>>()}});
                // Full expected sources and packets are authored before any
                // provider query; actual returned edits are applied unchanged.
                let expected = json!({"nativeGuard":expected_guard(),
                    "transaction":expected(&goldens, references, rename)});
                let context = format!("alias {form:?}, {name}, {query}, newline={newline:?}");
                let guard = prove_native(&mut fixture);
                let transaction = observe(
                    &mut fixture,
                    &files,
                    &[uri.clone()],
                    (&uri, &files[0].1, query, replacement),
                    &goldens,
                    &context,
                );
                let actual = json!({"nativeGuard":guard,"transaction":transaction});
                capture(&fixture, &files, &context, &expected, &actual);
                wanted.push(expected);
                observed.push(actual);
            }
        }
    }
    assert_eq!(observed, wanted, "all complete {form:?} alias transactions");
}

fn expected_guard() -> Value {
    json!({"source":GUARD,"repairedSource":REPAIRED_GUARD,
        "invalidDiagnostics":[{
            "range":{"start":{"line":1,"character":6},"end":{"line":1,"character":17}},
            "severity":1,"code":2322,"source":"vize/types",
            "message":"Type 'string' is not assignable to type 'number'."
        }],"repairedDiagnostics":[]})
}

fn prove_native(fixture: &mut support::Fixture) -> Value {
    let uri = fixture.write_file("NativeGuard.vue", GUARD);
    let invalid = fixture.open_file(&uri, GUARD);
    fixture.write_file("NativeGuard.vue", REPAIRED_GUARD);
    let repaired = fixture.change_file(&uri, REPAIRED_GUARD, 2);
    json!({"source":GUARD,"repairedSource":REPAIRED_GUARD,
        "invalidDiagnostics":invalid,"repairedDiagnostics":repaired})
}
