use super::super::{DotNotation, Eqeqeq, NoUselessConcat};
use super::reported;
use crate::rule::Rule;

const EXPRESSION: &str = "row['id'] + '-' + 'x' + (row.kind == 1)";

fn texts(rule: Box<dyn Rule>, source: &str) -> Vec<String> {
    reported(rule, source)
        .into_iter()
        .map(str::to_owned)
        .collect()
}

fn same(rule: impl Fn() -> Box<dyn Rule>, left: &str, right: &str) {
    let found = texts(rule(), left);
    assert!(!found.is_empty(), "{left}");
    assert_eq!(found, texts(rule(), right));
}

#[test]
fn template_wrapper_key_matches_an_ordinary_binding() {
    let template_for =
        format!(r#"<template v-for="row in rows" :key="{EXPRESSION}"><span /></template>"#);
    let item_for = format!(r#"<li v-for="row in rows" :key="{EXPRESSION}" />"#);
    let child =
        format!(r#"<template v-for="row in rows"><span :title="{EXPRESSION}" /></template>"#);
    let template_if = format!(r#"<template v-if="ready" :key="{EXPRESSION}"><span /></template>"#);

    same(eqeqeq, &template_for, &item_for);
    same(eqeqeq, &template_if, &item_for);
    same(dot_notation, &template_for, &child);
    same(no_useless_concat, &template_for, &item_for);

    let static_key = r#"<template v-for="row in rows" key="row.id == 1"><span /></template>"#;
    assert!(texts(eqeqeq(), static_key).is_empty());
}

fn eqeqeq() -> Box<dyn Rule> {
    Box::new(Eqeqeq)
}

fn dot_notation() -> Box<dyn Rule> {
    Box::new(DotNotation)
}

fn no_useless_concat() -> Box<dyn Rule> {
    Box::new(NoUselessConcat)
}
