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
];

fn sources() -> Vec<(vize_l0::String, JsxLang)> {
    let mut sources = Vec::new();
    for corpus in [VAPOR_CORPUS, BABEL_CORPUS] {
        let corpus: Value = serde_json::from_str(corpus).expect("corpus JSON");
        for case in corpus["cases"].as_array().expect("cases") {
            let lang = if case["lang"] == "tsx" {
                JsxLang::Tsx
            } else {
                JsxLang::Jsx
            };
            sources.push((case["source"].as_str().expect("source").into(), lang));
        }
    }
    sources.extend(SHAPES.iter().map(|source| ((*source).into(), JsxLang::Jsx)));
    sources
}

#[test]
fn level_lowering_matches_the_relief_projection() {
    let (mut roots, mut native) = (0, 0);
    for (source, lang) in sources() {
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
            let Ok(level) = root.l2.as_ref() else {
                continue;
            };
            if !level.native {
                continue;
            }
            native += 1;
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
    println!("level-lowered {native} of {roots} roots");
    assert!(native >= SHAPES.len(), "{native} of {roots}");
}
