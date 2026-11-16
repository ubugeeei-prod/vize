//! The shared L1→L2 JSX lowering against the Relief projection it replaces.
#![expect(
    clippy::disallowed_macros,
    reason = "the witness compares Debug renderings"
)]

use serde_json::Value;
use vize_l0::Allocator;

use super::try_lower_root;
use crate::lower::BabelLoweringOptions;
use crate::{JsxCompatMode, JsxLang, JsxOutputMode, lower_source_with_compat};

const VAPOR_CORPUS: &str = include_str!("../../tests/vue_jsx_vapor_compat/fixtures/corpus.json");
const BABEL_CORPUS: &str = include_str!("../../tests/babel_compat/fixtures/corpus.json");

/// Admitted shapes the oracle corpora exercise only inside larger roots.
const SHAPES: &[&str] = &[
    "const A = () => <div class=\"card\" />;",
    "const A = () => <p>Hello</p>;",
    "const A = () => <div>{count}</div>;",
    "const A = () => <div>{(a, b)}</div>;",
    "const A = () => <div>{x as number}</div>;",
    "const A = () => <><i/><b/></>;",
    "const A = () => <button disabled={off} />;",
    "const A = () => <div id=\"a\" class={c} />;",
    "const A = () => <div class=\"a\" class={c} />;",
    "const A = () => <div {...attrs} id={id} {...more} />;",
    "const A = () => <input onInputPassiveCapture={h} onCapture={c} onClick={f} />;",
    "const A = () => <svg viewBox=\"0 0 1 1\" xlink:href={u}><path d={d} /></svg>;",
    "const A = () => <p>\n  first\n  second\n  <b>{' '}</b>\n</p>;",
    "const A = () => <p>{}{/* c */}</p>;",
    "const A = () => <component is={view} />;",
    "const A = () => <component is=\"div\" />;",
    "import Child from './c';\nconst A = () => <Child msg={m}>text<b /></Child>;",
    "const A = () => <Unbound msg={m} />;",
    "const A = () => <ui.Button kind=\"x\" />;",
    "const A = () => <Fragment><i /></Fragment>;",
    "const A = () => <template><i /></template>;",
    "const A = () => <div><><i /><>{'x'}</></></div>;",
    "function A() { const Local = () => null; return <Local />; }",
    "const on = a > 1;\nconst A = () => <div>{on && <i />}{!x && <b>{y}</b>}</div>;",
    "const A = () => <p>{a ? <i /> : (b) ? <b /> : <s />}</p>;",
    "const A = () => <p>{a ? (b ? <i /> : <b />) : <s />}</p>;",
    "const A = () => <ul>{xs.map((x, i) => <li key={i}>{x}</li>)}</ul>;",
    "const A = () => <ul>{xs.map(function ({ id }) { return <li id={id} />; })}</ul>;",
    "const A = () => <ul>{rows.map(() => ok ? <i /> : <b />)}</ul>;",
    "const A = () => <ul>{rows.map((r) => r.items.map((c) => <li>{c}</li>))}</ul>;",
    "import Card from './c';\nconst A = () => <Card>{a ? <i /> : <b />}</Card>;",
    "const A = () => <Free>{xs.map((x) => <i>{x}</i>)}text</Free>;",
    "const A = () => <input v-model={val} />;",
    "const A = () => <input type=\"checkbox\" value=\"a\" v-model_lazy_trim={c} />;",
    "const A = () => <select v-model={[d, ['number']]}><option value=\"a\">a</option></select>;",
    "const A = () => <textarea v-model={state.text} />;",
    "import B from './b';\nconst A = () => <B v-model:title={t} v-model={[u, 'label', ['m']]} v-models={[[x], [y, 'y', ['n']]]} />;",
    "const A = () => <Card v-show={on}><i v-html={h} /><b v-text={t} v-focus /></Card>;",
    "const A = () => <p><a v-mark:left={v} /><a v-mark={[v, 'top', ['a', 'b']]} /><a v-mark={[v, f]} /></p>;",
    "const A = () => <component is={view} v-show={on} v-focus />;",
];

/// TSX spellings of the structural shapes.
const TSX_SHAPES: &[&str] = &[
    "const A = () => <ul>{(xs as Item[]).map((x: Item, i: number) => <li key={i}>{x.id}</li>)}</ul>;",
    "const A = () => <p>{(a satisfies boolean) ? <i /> : <b />}</p>;",
    "const A = () => <input v-model={(state as S).value!} />;",
];

/// Every case, and whether each of its roots must take the level lowering.
fn sources() -> Vec<(vize_l0::String, JsxLang, bool)> {
    let mut sources = Vec::new();
    for corpus in [VAPOR_CORPUS, BABEL_CORPUS] {
        let corpus: Value = serde_json::from_str(corpus).expect("corpus JSON");
        for case in corpus["cases"].as_array().expect("cases") {
            let lang = if case["lang"] == "tsx" {
                JsxLang::Tsx
            } else {
                JsxLang::Jsx
            };
            sources.push((case["source"].as_str().expect("source").into(), lang, false));
        }
    }
    sources.extend(
        SHAPES
            .iter()
            .map(|source| ((*source).into(), JsxLang::Jsx, true)),
    );
    sources.extend(
        TSX_SHAPES
            .iter()
            .map(|source| ((*source).into(), JsxLang::Tsx, true)),
    );
    sources
}

#[test]
fn level_lowering_matches_the_relief_projection() {
    let (mut roots, mut native, mut corpus) = (0, 0, (0, 0));
    for (source, lang, shape) in sources() {
        let allocator = Allocator::new();
        let source = allocator.alloc_str(&source);
        let (lowered, _) = lower_source_with_compat(
            &allocator,
            allocator.as_oxc(),
            source,
            lang,
            JsxCompatMode::Native,
            JsxOutputMode::Vdom,
            BabelLoweringOptions {
                vdom_lane: true,
                ..Default::default()
            },
        );
        for root in &lowered.roots {
            roots += 1;
            corpus.1 += usize::from(!shape);
            let level = root.l2.as_ref().ok().filter(|level| level.native);
            assert!(level.is_some() || !shape, "not level-lowered: {source}");
            let Some(level) = level else {
                continue;
            };
            native += 1;
            corpus.0 += usize::from(!shape);
            let projected = try_lower_root(&allocator, source, &root.root)
                .expect("the projection admits every root the level lowering admits");
            assert_eq!(
                format!(
                    "{:?}",
                    (&level.root, level.op_count, &level.scopes, level.features)
                ),
                format!(
                    "{:?}",
                    (
                        &projected.root,
                        projected.op_count,
                        &projected.scopes,
                        projected.features
                    )
                ),
                "{source}"
            );
        }
    }
    println!(
        "level-lowered {native} of {roots} roots, {} of {} oracle corpus roots",
        corpus.0, corpus.1
    );
}
