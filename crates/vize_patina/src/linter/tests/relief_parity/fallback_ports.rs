//! Rules moved off the JSX lowering fallback (#6848) report identically
//! through the markup facade and their Relief hooks.

use crate::rule::Rule;
use crate::rules::a11y::{LandmarkRoles, RoleImgHasName};
use crate::rules::html::IdDuplication;
use crate::rules::vapor::NoInlineTemplate;
use crate::rules::vue::{
    A11yImgAlt, NoChildContent, NoLoneTemplate, NoTemplateKey, NoVForTemplateKey,
    NoVForTemplateKeyOnChild,
};

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

fn id_family() -> Vec<Box<dyn Rule>> {
    vec![Box::new(IdDuplication)]
}

const ID_TEMPLATES: &[&str] = &[
    r#"<div id="a">A</div><div id="a">B</div><div id="a">C</div><div :id="a">D</div><div id>E</div>"#,
    r#"<div v-if="x"><i id="r"></i></div><div v-else-if="y"><i id="r"></i></div><div v-else><i id="r"></i><b id="r"></b></div><p id="r"></p>"#,
    "<p id=\"g\" v-if=\"x\">a</p>\n<!-- gap -->\n<p id=\"g\" v-else>b</p>\n<p id=\"g\">c</p>",
    r#"<template v-if="x"><i id="t"></i></template><template v-else><i id="t"></i></template><Comp id="c" /><Comp id="c" />"#,
    r#"<ul><li v-for="item in items" id="row">{{ item }}</li></ul><div id="row"></div><div v-pre id="row"></div>"#,
    r#"<div id="a &amp; b"></div><div id="a &amp; b"></div><div id=" a"></div><div id="a"></div>"#,
];

const ID_JSX: &[&str] = &[
    r#"const A = () => <><div id="a" /><div id="a" /><div id={a} /><div id="a" /></>;"#,
    r#"const A = () => <div>{ok ? <i id="r" /> : <i id="r" />}<b id="r" />{ok && <s id="s" />}<s id="s" /></div>;"#,
    r#"const A = () => <div>{a ? (b ? <i id="n" /> : <i id="n" />) : <i id="n" />}<Comp id="c" /><Comp id="c" /></div>;"#,
    r#"const A = () => <ul>{items.map((item) => <li id="row" key={item}>{item}</li>)}<li id="row" /></ul>;"#,
    r#"function A() { const one = <p id="x" />; return <section id="x">{one}<p id="x" /></section>; }"#,
];

#[test]
fn ported_id_duplication_matches_its_relief_hooks() {
    super::assert_parity(id_family, ID_TEMPLATES, ID_JSX);
}

fn vue_family() -> Vec<Box<dyn Rule>> {
    vec![
        Box::new(NoTemplateKey),
        Box::new(NoVForTemplateKey),
        Box::new(NoVForTemplateKeyOnChild),
        Box::new(NoLoneTemplate),
        Box::new(NoChildContent),
        Box::new(NoInlineTemplate),
    ]
}

const VUE_TEMPLATES: &[&str] = &[
    r#"<template key="a"><p>x</p></template><template :key="b"><p>y</p></template><template v-bind:key="c" v-if="ok"><p>z</p></template>"#,
    r#"<template v-for="item in items" :key="item.id"><li>{{ item }}</li></template><template v-for="item in items" key="k"><li /></template>"#,
    r#"<template v-for="item in items"><li :key="item.id">{{ item }}</li><li v-if="item.ok" :key="item.id" /><li v-bind:key="item.id" /></template>"#,
    r#"<template v-for="item in items" :key="item"><li :key="item" /></template><template v-for=""><li :key="x" /></template>"#,
    r#"<template><p>lone</p></template><template v-if="a">a</template><template v-else>b</template><Comp><template #header>h</template><template v-slot:default>d</template></Comp>"#,
    r#"<template slot="old">s</template><template slot-scope="p">p</template><template :slot="name">n</template><template scope="q">q</template><Template>c</Template>"#,
    r#"<div v-html="html">child</div><div v-text="text"><!-- c --></div><div v-html="html">  </div><p v-text="t">{{ x }}</p><p v-html="h"><b v-if="x" /></p>"#,
    r#"<ul><li v-html="a">one<li>two</ul><div v-html="raw"></div><MyComp inline-template><p>x</p></MyComp><div INLINE-TEMPLATE></div>"#,
];

const VUE_JSX: &[&str] = &[
    r#"const A = () => <><div v-html={html}>child</div><div v-text={text}>{value}</div><div v-html={raw} /></>;"#,
    r#"const A = () => <div v-html={html}>{ok && <b />}{items.map((i) => <i key={i} />)}</div>;"#,
    r#"const A = () => <><template key="a"><p /></template><MyComp inline-template /><div inline-template="x" /></>;"#,
];

#[test]
fn ported_vue_rules_match_their_relief_hooks() {
    super::assert_parity(vue_family, VUE_TEMPLATES, VUE_JSX);
}
