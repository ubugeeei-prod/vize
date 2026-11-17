//! Rules moved off the JSX lowering fallback (#6848) report identically
//! through the markup facade and their Relief hooks.

use crate::rule::Rule;
use crate::rules::a11y::{LandmarkRoles, RoleImgHasName};
use crate::rules::vue::A11yImgAlt;

fn a11y_family() -> Vec<Box<dyn Rule>> {
    vec![Box::new(RoleImgHasName), Box::new(A11yImgAlt)]
}

/// Not in `sfc::facade::RULES`: templates and SFCs keep its Relief document,
/// so only JSX/TSX compares two projections.
fn landmark_family() -> Vec<Box<dyn Rule>> {
    vec![Box::new(LandmarkRoles)]
}

const A11Y_TEMPLATES: &[&str] = &[
    r#"<main>one</main><main>two</main><main>three</main>"#,
    r#"<main v-if="a">A</main><main v-else-if="b">B</main><main v-else>C</main><main>D</main>"#,
    "<nav v-if=\"a\">A</nav>\n<!-- gap -->\n<nav v-else>B</nav>\n<nav>C</nav>",
    r#"<template v-if="a"><nav>A</nav><nav>B</nav></template><nav v-else>C</nav>"#,
    r#"<nav aria-label="Same">x</nav><nav :aria-label="'Same'">y</nav><nav :aria-label="label">z</nav><nav title=" ">w</nav>"#,
    r#"<div role="main">x</div><main>y</main><header role="banner">h</header><footer>f</footer><footer>g</footer>"#,
    r#"<section aria-label="One">a</section><section aria-label="One">b</section><section>c</section><Nav>d</Nav><Nav>e</Nav>"#,
    r#"<ul><li v-for="item in items"><nav>{{ item }}</nav></li></ul><nav>outer</nav>"#,
    r#"<div v-if="a"><aside>x</aside></div><div v-else><aside>y</aside><aside>z</aside></div>"#,
    r#"<form role="search">a</form><search>b</search><form>c</form><form>d</form><div role="none">e</div>"#,
    r#"<span role="img"></span><span role="img" aria-label="Status"></span><img src="a" alt="" role="img"><img :src="s" role="img" :alt="label">"#,
    r#"<i role="img presentation" title=" "></i><b :role="'img'"></b><b :role="role"></b><svg role="img"><title>t</title></svg>"#,
    r#"<img src="a"><img src="a" alt><img :alt="x"><img v-bind:alt="x"><img :[alt]="x"><Img src="a">"#,
];

const A11Y_JSX: &[&str] = &[
    r#"const A = () => <><main>one</main><main>two</main></>;"#,
    r#"const A = () => <>{ok ? <main>a</main> : <main>b</main>}<main>c</main></>;"#,
    r#"const A = () => <div>{ok && <nav>a</nav>}<nav>b</nav>{a ? (b ? <nav>c</nav> : <nav>d</nav>) : null}</div>;"#,
    r#"const A = () => <div>{items.map((item) => <nav key={item}>{item}</nav>)}<nav aria-label="Same" /><nav aria-label={"Same"} /></div>;"#,
    r#"const A = () => <><span role="img" /><img src="a" alt="" role="img" /><img src="a" /><img alt={x} /><Img src="a" /></>;"#,
    r#"function A() { const side = <aside>x</aside>; return <section aria-label="S">{side}<aside>y</aside><footer /><footer /></section>; }"#,
];

#[test]
fn ported_a11y_rules_match_their_relief_hooks() {
    super::assert_parity(a11y_family, A11Y_TEMPLATES, A11Y_JSX);
}

#[test]
fn ported_landmark_roles_match_its_relief_document_on_jsx() {
    super::assert_parity(landmark_family, &[], A11Y_JSX);
}
