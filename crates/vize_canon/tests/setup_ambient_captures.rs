#[path = "support/script_block_project.rs"]
mod project;

#[test]
fn setup_ambient_captures_preserve_authored_types_and_diagnostics() {
    let fixtures: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/setup_ambient_captures.json")).unwrap();
    for fixture in fixtures.as_array().unwrap() {
        for newline in ["\n", "\r\n"] {
            let source = format!(
                "<script setup lang=\"ts\">\n{}\n</script>\n<template>{}</template>\n",
                fixture["script"].as_str().unwrap(),
                fixture["template"].as_str().unwrap_or("{{ label }}")
            )
            .replace('\n', newline);
            let expected: Vec<&str> = fixture["expected"]
                .as_array()
                .unwrap()
                .iter()
                .map(|row| row.as_str().unwrap())
                .collect();
            let consumer = fixture["consumer"]
                .as_str()
                .map(|source| source.replace('\n', newline));
            let mut files = vec![("src/App.vue", source.as_str())];
            if let Some(consumer) = &consumer {
                files.push(("src/consumer.ts", consumer));
            }
            assert_eq!(
                project::check(&files),
                expected,
                "{} with {newline:?}",
                fixture["name"]
            );
        }
    }
}
