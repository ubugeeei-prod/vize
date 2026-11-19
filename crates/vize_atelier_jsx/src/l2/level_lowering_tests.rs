//! The shared L1→L2 JSX lowering against the Relief projection it replaces.
#![expect(
    clippy::disallowed_macros,
    reason = "the witness compares Debug renderings"
)]

use serde_json::Value;
use vize_atelier_core::lane::ScriptTables;
use vize_l0::Allocator;

use super::try_lower_root;
use crate::lower::BabelLoweringOptions;
use crate::vdom::{VdomCompatOptions, VdomCompileOptions, compile_root_to_vdom};
use crate::{JsxCompatMode, JsxLang, JsxOutputMode, LoweredRoot, lower_source_with_compat};

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
    "const A = () => <p>{busy ? \"Loading\" : <i class=\"idle\" />}</p>;",
    "const A = () => <p><i />{busy ? label : (b) ? <b /> : null}</p>;",
    "const A = () => <p>{a ? (x, y) : <i />}{a ? `t${n}` : <b />}</p>;",
    "const A = () => <ul>{rows.map((r) => r.ok ? r.label : <li>{r.id}</li>)}</ul>;",
    "import Card from './c';\nconst A = () => <Card>{n ? 'none' : <i />}</Card>;",
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
    "import Card from './c';\nconst A = () => <Card>\n  {(item) => <li>{item}</li>}\n</Card>;",
    "const A = () => <List>{function ({ id }) { return <i id={id} />; }}</List>;",
    "const A = () => <Card>{{ header: () => <h1>T</h1>, 'footer': (p) => <><i>{p}</i><b /></>, body() { return ok ? <i /> : <b />; } }}</Card>;",
    "const A = () => <Card>{{}}</Card>;",
    "const A = () => <List>{(rows) => rows.map((r) => <li key={r.id}>{r.name}</li>)}</List>;",
    "const A = () => <Outer>{() => <Inner>{(x) => <b>{x}</b>}</Inner>}</Outer>;",
    "const A = () => <Card v-slots={slots} title=\"t\" x={1}>text</Card>;",
    "const A = () => <Card vSlots={(own.slots)}>{(p) => <i>{p}</i>}</Card>;",
    "const A = () => <Card title=\"t\" v-slots={{ header: () => <h1>T</h1>, footer: (p) => <i>{p}</i> }}><b /></Card>;",
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

/// One root through the VDOM backend.
fn vdom_code<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    lang: JsxLang,
    root: LoweredRoot<'a>,
    tables: &ScriptTables,
) -> (vize_l0::String, vize_l0::String) {
    let mut diagnostics = Vec::new();
    let component = compile_root_to_vdom(
        allocator,
        root,
        tables,
        lang.is_typescript(),
        &VdomCompileOptions::default(),
        VdomCompatOptions::default(),
        &mut diagnostics,
        source,
    );
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    (component.preamble, component.code)
}

/// The module as the VDOM compile lowers it.
fn lower_vdom_lane<'a>(
    allocator: &'a Allocator,
    source: &str,
    lang: JsxLang,
) -> (&'a str, crate::LowerOutput<'a>) {
    let source: &'a str = allocator.alloc_str(source);
    let (lowered, _) = lower_source_with_compat(
        allocator,
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
    (source, lowered)
}

/// Every root the level lowering admits emits what it emitted when the Relief
/// projection was its L2 (or, where the projection refuses, the Relief route).
#[test]
fn level_lowered_roots_emit_what_the_projection_emitted() {
    for (source, lang, _) in sources() {
        let (level, relief) = (Allocator::new(), Allocator::new());
        let (level_source, level_lowered) = lower_vdom_lane(&level, &source, lang);
        let (relief_source, mut relief_lowered) = lower_vdom_lane(&relief, &source, lang);
        let relief: &Allocator = &relief;
        if level_lowered.has_errors() {
            continue;
        }
        let level_tables: &ScriptTables =
            level.alloc_owned(ScriptTables::from_croquis(&level_lowered.analysis));
        let relief_tables: &ScriptTables =
            relief.alloc_owned(ScriptTables::from_croquis(&relief_lowered.analysis));
        for root in &mut relief_lowered.roots {
            root.l2 = try_lower_root(relief, relief_source, &root.root);
        }
        for (level_root, relief_root) in level_lowered.roots.into_iter().zip(relief_lowered.roots) {
            if !level_root.l2.as_ref().is_ok_and(|l2| l2.native) {
                continue;
            }
            assert_eq!(
                vdom_code(&level, level_source, lang, level_root, level_tables),
                vdom_code(relief, relief_source, lang, relief_root, relief_tables),
                "{source}"
            );
        }
    }
}
