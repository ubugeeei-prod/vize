//! VDOM-backend JSX/TSX parity suite (Part of #1491).

mod common;

use common::{dom_code, snapshot_cases};
use vize_atelier_jsx::JsxLang;

#[test]
fn vdom_parity_matrix_snapshot() {
    let cases = [
        ("intrinsic element", "const A = () => <div/>;"),
        ("component", "const A = () => <Comp/>;"),
        ("fragment", "const A = () => <><a/><b/></>;"),
        (
            "fragment with dynamic child",
            "const A = () => <><h1>a</h1><p>{x}</p></>;",
        ),
        (
            "static attributes",
            "const A = () => <div class=\"a\" id=\"b\"/>;",
        ),
        ("boolean attribute", "const A = () => <input disabled/>;"),
        ("single dynamic bind", "const A = () => <div id={x}/>;"),
        (
            "multiple dynamic binds",
            "const A = () => <div id={a} title={b}/>;",
        ),
        ("spread alone", "const A = () => <div {...attrs}/>;"),
        (
            "spread with static",
            "const A = () => <div class=\"a\" {...attrs}/>;",
        ),
        ("dynamic class", "const A = () => <div class={c}/>;"),
        ("array class", "const A = () => <div class={['a', b]}/>;"),
        ("dynamic style", "const A = () => <div style={s}/>;"),
        (
            "namespaced colon attribute",
            "const A = () => <use xlink:href=\"#id\"/>;",
        ),
        ("key prop", "const A = () => <div key={k}/>;"),
        ("ref prop", "const A = () => <div ref={r}/>;"),
        ("static text", "const A = () => <div>hello</div>;"),
        ("interpolation", "const A = () => <div>{count}</div>;"),
        ("mixed text", "const A = () => <div>Hi {name}!</div>;"),
        (
            "logical and child",
            "const A = () => <ul>{ok && <li/>}</ul>;",
        ),
        (
            "ternary arms",
            "const A = () => <div>{ok ? <a/> : <b/>}</div>;",
        ),
        (
            "map callback",
            "const A = () => <ul>{items.map((i) => <li>{i}</li>)}</ul>;",
        ),
        ("directive v-if", "const A = () => <div v-if={ok}>x</div>;"),
        (
            "non jsx logical and",
            "const A = () => <div>{a && b}</div>;",
        ),
        ("v-model input", "const A = () => <input v-model={val}/>;"),
        (
            "v-model checkbox",
            "const A = () => <input type=\"checkbox\" v-model={checked}/>;",
        ),
        (
            "v-model component",
            "const A = () => <Input v-model={val}/>;",
        ),
        (
            "v-model named argument",
            "const A = () => <Comp v-model:foo={val}/>;",
        ),
        ("v-show", "const A = () => <div v-show={ok}>x</div>;"),
        ("v-html", "const A = () => <div v-html={raw}/>;"),
        ("v-text", "const A = () => <div v-text={msg}/>;"),
        ("custom directive", "const A = () => <div v-foo={bar}/>;"),
        ("plain event", "const A = () => <button onClick={h}/>;"),
        (
            "capture event",
            "const A = () => <button onClickCapture={h}/>;",
        ),
        ("once event", "const A = () => <button onClickOnce={h}/>;"),
        (
            "passive capture event",
            "const A = () => <input onInputPassiveCapture={h}/>;",
        ),
        (
            "object child slots",
            "const A = () => <Comp>{{ header: () => <h1>Hi</h1> }}</Comp>;",
        ),
        (
            "render prop slot",
            "const A = () => <List>{(item) => <li>{item}</li>}</List>;",
        ),
        (
            "scoped named slot",
            "const A = () => <List>{{ item: ({ x }) => <li>{x}</li> }}</List>;",
        ),
        (
            "plain children default slot",
            "const A = () => <Card><h1>Title</h1></Card>;",
        ),
        (
            "v-model modifier array",
            "const A = () => <input v-model={[val, ['trim']]}/>;",
        ),
        (
            "v-model modifier array no modifiers",
            "const A = () => <input v-model={[val]}/>;",
        ),
        (
            "v-model component arg modifiers",
            "const A = () => <Comp v-model={[val, 'foo', ['trim']]}/>;",
        ),
        (
            "v-model underscore lazy",
            "const A = () => <input v-model_lazy={val}/>;",
        ),
        (
            "v-model underscore number lazy",
            "const A = () => <input v-model_number_lazy={val}/>;",
        ),
    ];

    insta::assert_snapshot!(snapshot_cases(&cases, |source| {
        dom_code(source, JsxLang::Jsx)
    }));
}
