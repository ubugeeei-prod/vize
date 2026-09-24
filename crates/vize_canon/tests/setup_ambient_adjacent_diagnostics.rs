#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#[path = "support/script_block_project.rs"]
mod project;

#[test]
fn adjacent_ambient_declarations_match_authored_native_diagnostics() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/setup_ambient_adjacent.json")).unwrap();
    for fixture in fixtures.as_array().unwrap() {
        for newline in ["\n", "\r\n"] {
            let source = format!(
                "<script setup lang=\"ts\">\n{}\n</script>\n<template>{}</template>\n",
                fixture["script"].as_str().unwrap(),
                fixture["template"].as_str().unwrap_or("<div />")
            )
            .replace('\n', newline);
            let expected: Vec<&str> = fixture["expected"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| row.as_str().unwrap())
                .collect();
            assert_eq!(
                project::check(&[("src/App.vue", &source)]),
                expected,
                "{} with {newline:?}",
                fixture["name"]
            );
        }
    }
}
