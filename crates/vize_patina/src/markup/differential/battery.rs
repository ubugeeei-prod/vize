//! The committed differential battery and its pinned census.
//!
//! Four planes, each chosen for a different kind of coverage:
//!
//! - [`TEMPLATES`]: hand-written templates that exercise every normalization
//!   the S1→S2 lowering applies and the facade must read back — `v-if`
//!   chains with gaps, `v-for` on elements and `<template>` carriers, merged
//!   text runs, implicit table owners, slot outlets, carrier keys, `v-pre`,
//!   every binding family and shorthand, and ill-formed spellings.
//! - The rule-fixture plane ([`super::rule_fixtures`]): every template the
//!   rule unit tests lint — what rule authors chose to exercise.
//! - The P2-15 construct matrix (`tests/fixtures/davinci-matrix/*.vue`):
//!   element kind × directive, generated, so no construct is missed by taste.
//! - [`JSX`]: JSX/TSX modules whose lowered roots the P2-16 projection admits
//!   (and some it refuses, counted).

use super::{JsxComparison, TemplateComparison, compare_jsx, compare_template};
use std::path::Path;
use vize_atelier_jsx::JsxLang;
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};

/// Hand-written Vue templates, `(name, source)`.
pub const TEMPLATES: &[(&str, &str)] = &[
    (
        "elements",
        r#"<div id="app" class="a b"><span title="t">hi</span><img src="/x.png" /></div>"#,
    ),
    (
        "components",
        r#"<MyComp :value="x" @change="onChange"><Child v-bind="attrs" /></MyComp><component :is="view" />"#,
    ),
    (
        "if-chain",
        r#"<div><p v-if="a">A</p><p v-else-if="b">B</p><p v-else>C</p></div>"#,
    ),
    (
        "if-chain-gaps",
        "<div>\n  <p v-if=\"a\">A</p>\n  <!-- between -->\n  <p v-else-if=\"b\">B</p>\n  <p v-else>C</p>\n  <span>after</span>\n</div>",
    ),
    (
        "if-chain-kept-gap",
        r#"<div><b v-if="a">A</b> <i v-else>B</i></div>"#,
    ),
    (
        "if-orphans",
        r#"<div><p v-else>orphan</p><p v-else-if="x">orphan</p><p v-if="y" v-else>both</p></div>"#,
    ),
    (
        "if-blank",
        r#"<div><p v-if>none</p><p v-if="">blank</p><p v-else="value">else-with-value</p></div>"#,
    ),
    (
        "for-element",
        r#"<ul><li v-for="(item, index) in items" :key="item.id">{{ index }}: {{ item.name }}</li></ul>"#,
    ),
    (
        "for-template",
        r#"<ul><template v-for="item in items" :key="item.id"><li>{{ item }}</li><li>sep</li></template></ul>"#,
    ),
    (
        "for-template-static-key",
        r#"<ul><template v-for="item in items" key="k"><li>{{ item }}</li></template></ul>"#,
    ),
    (
        "for-blank",
        r#"<ul><li v-for="">blank</li><li v-for>none</li><li v-for="a in b in c">malformed</li></ul>"#,
    ),
    (
        "if-for",
        r#"<ul><li v-if="show" v-for="x in xs" :key="x">{{ x }}</li><template v-if="t" v-for="y of ys"><b>{{ y }}</b></template></ul>"#,
    ),
    (
        "template-if",
        r#"<div><template v-if="ok"><h1>T</h1><p>P</p></template><template v-else><p>no</p></template></div>"#,
    ),
    (
        "template-if-key",
        r#"<div><template v-if="ok" key="a"><p>A</p></template><p v-else key="b">B</p></div>"#,
    ),
    (
        "carrier-keys",
        r#"<div><p v-if="a" key="one">1</p><p v-else-if="b" :key="two">2</p><p v-else key>3</p></div>"#,
    ),
    (
        "slots",
        r#"<MyComp><template #header="{ title }"><h1>{{ title }}</h1></template><template v-slot:default>body</template><template v-if="x" #extra>e</template></MyComp>"#,
    ),
    (
        "slot-outlets",
        r#"<div><slot /><slot name="header" :title="t" @ping="p">fallback</slot><slot :name="dyn" /><slot :name /></div>"#,
    ),
    (
        "text-runs",
        r#"<p>Hello {{ name }}, you have {{ count }} messages &amp; more</p>"#,
    ),
    (
        "text-comments",
        "<p>a<!-- c -->b <!-- d --> c{{ x }}<!-- e -->{{ y }}</p>",
    ),
    (
        "whitespace",
        "<div>\n\n  <span> a   b </span>\n\n  <pre>  keep\n   this  </pre>\n  text   with   spaces\n</div>",
    ),
    (
        "entities",
        r#"<p title="a &amp; b">x &lt; y &nbsp; z &#169;</p>"#,
    ),
    (
        "tables",
        r#"<table><tr><td>1</td></tr><tr><td v-for="c in cols">{{ c }}</td></tr></table>"#,
    ),
    (
        "table-groups",
        r#"<table><thead><th>h</th></thead><tbody><td>d</td></tbody></table>"#,
    ),
    (
        "v-pre",
        r#"<div v-pre :x="1" @y="z">{{ raw }}<span v-if="no">raw</span></div>"#,
    ),
    (
        "bindings",
        r#"<input :value="v" @input.trim.lazy="onInput" v-model.number="n" .prop="p" :[dynamic]="d" @[evt].stop="h" :same-name />"#,
    ),
    (
        "models",
        r#"<div><input v-model="a" /><select v-model="b"><option>1</option></select><Comp v-model:title.trim="t" /><input v-model /></div>"#,
    ),
    (
        "dom-directives",
        r#"<div v-show="s" v-html="h" v-text="t" v-cloak v-once v-memo="[m]" v-custom:arg.mod="c"></div>"#,
    ),
    (
        "ill-formed",
        r#"<div v-once="x" v-memo v-show v-html v-text></div>"#,
    ),
    (
        "events",
        r#"<button @click="go" @click.prevent @keyup.enter="submit" v-on="handlers" v-on:custom-event="x">Go</button>"#,
    ),
    (
        "svg",
        r#"<svg viewBox="0 0 10 10"><path d="M0 0" /><foreignObject><div>html</div></foreignObject></svg>"#,
    ),
    ("math", r#"<math><mi>x</mi><mo>+</mo></math>"#),
    (
        "nested",
        r#"<div><section v-if="a"><ul><li v-for="i in list" :key="i"><a :href="i.url">{{ i.label }}</a></li></ul></section><footer v-else>none</footer></div>"#,
    ),
    (
        "a11y",
        r##"<div><img src="a.png"><a href="#">x</a><h1></h1><iframe src="/f"></iframe><input type="text" placeholder="name"><button>ok</button></div>"##,
    ),
    (
        "builtins",
        r#"<Transition name="fade"><KeepAlive><component :is="c" /></KeepAlive></Transition><Teleport to="body"><p>t</p></Teleport>"#,
    ),
    (
        "unclosed",
        r#"<div><p>unclosed<span>deep</div><p v-if="x">tail"#,
    ),
    ("stray", r#"</div><p>ok</p></span>text"#),
    ("interp-only", r#"{{ a }}{{ b }}"#),
];

/// JSX/TSX modules, `(name, lang, source)`.
pub const JSX: &[(&str, JsxLang, &str)] = &[
    (
        "elements",
        JsxLang::Jsx,
        r#"const A = () => <div id="app" className="a"><img src="/x.png" alt="x" /><span>{label}</span></div>;"#,
    ),
    (
        "events",
        JsxLang::Jsx,
        r#"const B = () => <button type="button" onClick={() => go()} onKeyup={up}>Go</button>;"#,
    ),
    (
        "conditional",
        JsxLang::Jsx,
        r#"const C = () => <div>{ok && <p>yes</p>}{ok ? <b>a</b> : <i>b</i>}</div>;"#,
    ),
    (
        "list",
        JsxLang::Jsx,
        r#"const D = () => <ul>{items.map((item) => <li key={item.id}>{item.name}</li>)}</ul>;"#,
    ),
    (
        "list-no-key",
        JsxLang::Jsx,
        r#"const E = () => <ul>{items.map((item) => <li>{item}</li>)}</ul>;"#,
    ),
    (
        "components",
        JsxLang::Tsx,
        r#"const F = () => <Comp value={x} onChange={f}><Child {...rest} /></Comp>;"#,
    ),
    (
        "model",
        JsxLang::Tsx,
        r#"const G = () => <div><input v-model={text} /><Comp v-model={value} /></div>;"#,
    ),
    (
        "directives",
        JsxLang::Jsx,
        r#"const H = () => <div v-show={visible} v-custom={c}>x</div>;"#,
    ),
    (
        "fragment",
        JsxLang::Jsx,
        r#"const I = () => <><h1>t</h1><p>body {text}</p></>;"#,
    ),
    (
        "nested-roots",
        JsxLang::Jsx,
        r#"function J() { const a = <em>a</em>; return <section>{a}<Comp render={<h2>r</h2>} /></section>; }"#,
    ),
    (
        "slots",
        JsxLang::Tsx,
        r#"const K = () => <Comp v-slots={{ header: () => <h1>h</h1> }}>body</Comp>;"#,
    ),
    (
        "text",
        JsxLang::Jsx,
        "const L = () => <p>\n  Hello {name},\n  welcome   back\n</p>;",
    ),
];

/// What the battery covered; pinned exactly in both lanes.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BatteryCensus {
    /// Hand-written templates compared.
    pub templates: usize,
    /// Rule-fixture templates compared (see [`super::rule_fixtures`]).
    pub rule_fixtures: usize,
    /// Construct-matrix templates compared.
    pub matrix: usize,
    /// Trace lines compared over the template planes.
    pub template_lines: usize,
    /// Templates the lint parse restructured (browser tree construction), so
    /// the two projections present different documents by design.
    pub restructured: usize,
    /// JSX modules, roots, refused roots, and trace lines.
    pub jsx: JsxComparison,
}

/// The committed census. Re-pinned deliberately, in both lanes at once.
pub const PINNED_BATTERY_CENSUS: BatteryCensus = BatteryCensus {
    templates: 37,
    rule_fixtures: 851,
    matrix: 90,
    template_lines: 6103,
    restructured: 0,
    jsx: JsxComparison {
        roots: 13,
        refused: 1,
        lines: 110,
    },
};

fn matrix_dir() -> std::path::PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/davinci-matrix")
}

fn tally(census: &mut BatteryCensus, comparison: TemplateComparison) {
    match comparison {
        TemplateComparison::Compared(lines) => census.template_lines += lines,
        TemplateComparison::Restructured => census.restructured += 1,
    }
}

/// Run the whole battery, panicking with the exact divergence on the first
/// disagreement.
pub fn run_battery() -> BatteryCensus {
    let mut census = BatteryCensus::default();
    for (name, source) in TEMPLATES {
        tally(&mut census, expect_same(name, compare_template(source)));
        census.templates += 1;
    }
    for (name, source) in super::rule_fixtures::rule_fixture_templates() {
        tally(&mut census, expect_same(&name, compare_template(&source)));
        census.rule_fixtures += 1;
    }
    let mut matrix: std::vec::Vec<_> = std::fs::read_dir(matrix_dir())
        .expect("the P2-15 construct matrix is committed")
        .map(|entry| entry.expect("matrix entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "vue"))
        .collect();
    matrix.sort();
    for path in &matrix {
        let source = std::fs::read_to_string(path).expect("matrix fixture is readable");
        let descriptor =
            parse_sfc(&source, SfcParseOptions::default()).expect("matrix fixture parses");
        let template = descriptor.template.expect("matrix fixture has a template");
        let name = vize_s0::cstr!("{}", path.display());
        tally(
            &mut census,
            expect_same(&name, compare_template(&template.content)),
        );
        census.matrix += 1;
    }
    for (name, lang, source) in JSX {
        let comparison = expect_same(name, compare_jsx(source, *lang));
        census.jsx.roots += comparison.roots;
        census.jsx.refused += comparison.refused;
        census.jsx.lines += comparison.lines;
    }
    census
}

fn expect_same<T>(name: &str, result: Result<T, super::Divergence>) -> T {
    match result {
        Ok(value) => value,
        Err(divergence) => panic!(
            "{name}: markup facade diverged at trace line {}\n  relief: {:?}\n  s2:     {:?}",
            divergence.line, divergence.relief, divergence.s2
        ),
    }
}
