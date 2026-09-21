//! P3-8 witnesses for JSX SSR on the S4 string plan: every case compiles on
//! the plan lane and with the legacy walker pinned, and the output must be
//! byte-identical; admitted cases must actually be emitted from the plan.

use vize_s0::{Allocator, String};

use super::{SsrLane, compile_root_on_lane};
use crate::{JsxLang, JsxOutputMode, lower_source};

/// Cases the plan must own.
const ADMITTED: &[(&str, &str, JsxLang)] = &[
    ("element", "const A = () => <div/>;", JsxLang::Jsx),
    ("text", "const A = () => <p>hello</p>;", JsxLang::Jsx),
    (
        "static attribute",
        "const A = () => <div class=\"box\">x</div>;",
        JsxLang::Jsx,
    ),
    (
        "interpolation",
        "const A = () => <div>{msg}</div>;",
        JsxLang::Jsx,
    ),
    (
        "dynamic attribute",
        "const A = () => <div id={id}>{msg}</div>;",
        JsxLang::Jsx,
    ),
    (
        "member expression",
        "const A = () => <p>{user.name}</p>;",
        JsxLang::Jsx,
    ),
    (
        "logical and",
        "const A = () => <div>{cond && <span>yes</span>}</div>;",
        JsxLang::Jsx,
    ),
    (
        "ternary",
        "const A = () => <div>{ok ? <span>yes</span> : <em>no</em>}</div>;",
        JsxLang::Jsx,
    ),
    (
        "nested ternary",
        "const A = () => <div>{a ? <p>A</p> : b ? <em>B</em> : <span>C</span>}</div>;",
        JsxLang::Jsx,
    ),
    (
        "list",
        "const A = () => <ul>{items.map((item) => <li>{item}</li>)}</ul>;",
        JsxLang::Jsx,
    ),
    (
        "tsx",
        "const A = (): JSX.Element => <span>{label}</span>;",
        JsxLang::Tsx,
    ),
    (
        "component in list",
        "const A = () => <ul>{rows.map((row) => <Item data={row}/>)}</ul>;",
        JsxLang::Jsx,
    ),
    (
        "scoped slot",
        "const A = () => <List>{(item) => <li>{item}</li>}</List>;",
        JsxLang::Jsx,
    ),
    (
        "named slots",
        "const A = () => <Comp>{{ header: () => <h1>H</h1>, default: () => <p>P</p> }}</Comp>;",
        JsxLang::Jsx,
    ),
    (
        "child component",
        "const A = () => <div><Child msg={x}/></div>;",
        JsxLang::Jsx,
    ),
    (
        "slot list",
        "const A = () => <List>{(rows) => rows.map((r) => <li>{r}</li>)}</List>;",
        JsxLang::Jsx,
    ),
    (
        "fragment root",
        "const A = () => <><i/><b/></>;",
        JsxLang::Jsx,
    ),
    (
        "event dropped",
        "const A = () => <button onClick={save} disabled={off}>go</button>;",
        JsxLang::Jsx,
    ),
    (
        "spread",
        "const A = () => <div {...attrs} id=\"a\" />;",
        JsxLang::Jsx,
    ),
    (
        "scoped style",
        "const A = () => { return <div class=\"a\">x<style scoped>{`.a { color: red }`}</style></div>; };",
        JsxLang::Jsx,
    ),
    (
        "v-show",
        "const A = () => <div v-show={ok}>x</div>;",
        JsxLang::Jsx,
    ),
    (
        "v-html",
        "const A = () => <div><p v-html={html} /></div>;",
        JsxLang::Jsx,
    ),
    (
        "v-model input",
        "const A = () => <div><input v-model={text} /></div>;",
        JsxLang::Jsx,
    ),
    (
        "teleport",
        "const A = () => <Teleport to=\"body\"><p>{msg}</p></Teleport>;",
        JsxLang::Jsx,
    ),
    (
        "nested list in ternary",
        "const A = () => <div>{ok ? items.map((i) => <b>{i}</b>) : <i>none</i>}</div>;",
        JsxLang::Jsx,
    ),
    (
        "two roots",
        "const A = () => <p>{a}</p>; const B = () => <ul>{xs.map((x) => <li key={x}>{x}</li>)}</ul>;",
        JsxLang::Jsx,
    ),
];

/// Cases that stay on the walker (JSX custom directives and `<component>`
/// are not projected to S2 yet); parity holds either way.
const OBSERVED: &[(&str, &str, JsxLang)] = &[
    (
        "custom directive",
        "const A = () => <div><p v-focus={x}>t</p></div>;",
        JsxLang::Jsx,
    ),
    (
        "dynamic component",
        "const A = () => <component is={view} />;",
        JsxLang::Jsx,
    ),
];

fn compile(source: &str, lang: JsxLang, lane: SsrLane) -> std::vec::Vec<String> {
    let allocator = Allocator::new();
    let lowered = lower_source(&allocator, allocator.as_oxc(), source, lang);
    let analysis = allocator.alloc_owned(lowered.analysis);
    lowered
        .roots
        .into_iter()
        .map(|root| {
            compile_root_on_lane(
                &allocator,
                root,
                analysis,
                JsxOutputMode::Vdom,
                source,
                lane,
            )
            .code
        })
        .collect()
}

fn plan_owns(source: &str, lang: JsxLang) -> bool {
    let allocator = Allocator::new();
    let lowered = lower_source(&allocator, allocator.as_oxc(), source, lang);
    !lowered.roots.is_empty()
        && lowered.roots.iter().all(|root| {
            root.s2.as_ref().is_ok_and(|s2| {
                vize_atelier_ssr::compile_s2_to_ssr(
                    &allocator,
                    s2.source,
                    &s2.root,
                    &vize_atelier_ssr::SsrCompilerOptions::default(),
                )
                .is_some()
            })
        })
}

#[test]
fn jsx_ssr_plan_lane_matches_the_legacy_walker() {
    for (name, source, lang) in ADMITTED.iter().chain(OBSERVED) {
        assert_eq!(
            compile(source, *lang, SsrLane::Plan),
            compile(source, *lang, SsrLane::Legacy),
            "{name}: {source}"
        );
    }
}

#[test]
fn admitted_jsx_ssr_cases_emit_from_the_plan() {
    for (name, source, lang) in ADMITTED {
        assert!(plan_owns(source, *lang), "{name}: {source}");
    }
}

#[test]
fn observed_jsx_ssr_cases_stay_on_the_walker() {
    for (name, source, lang) in OBSERVED {
        assert!(!plan_owns(source, *lang), "{name}: {source}");
    }
}
