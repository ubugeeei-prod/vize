//! `ui.if` / `ui.for` from `&&`, `?:` and `.map()` keep their authored script.

use super::{links, support};
use vize_l0::{Allocator, Span};
use vize_l2::op::Op;
use vize_l4::targets::ts::{JsxModuleRefusal, LinkMapping, LinkRole, project_jsx_module};

const VNODE: &str = " as unknown as import(\"vue\").VNode)";

fn projected(source: &str) -> String {
    let arena = Allocator::default();
    let module = support::lower(&arena, source, true).unwrap();
    project_jsx_module(&module.program, &module.roots)
        .unwrap()
        .document()
        .as_str()
        .to_owned()
}

#[test]
fn conditionals_and_maps_keep_authored_script_around_projected_regions() {
    for (source, root) in [
        (
            "declare const ok: boolean; declare const n: number; const v = <div>{ok ? <a title={n} /> : <b />}</div>;",
            "([ok ? [__vizeNativeElementProp<\"a\", \"title\">((n))] : []]",
        ),
        (
            "const shown = true; const v = <p>{/* c */ shown && <i />}</p>;",
            "([/* c */ shown && []]",
        ),
        (
            "declare const a: boolean, b: boolean; const v = <>{a ? <i /> : (b ? <b /> : <s />)}<hr /></>;",
            "([a ? [] : (b ? [] : [])]",
        ),
        (
            "declare const xs: string[]; const v = <ul>{xs.map((x, i) => { return (<li title={x} key={i} />); })}</ul>;",
            "([xs.map((x, i) => { return ([__vizeNativeElementProp<\"li\", \"title\">((x)), __vizeNativeElementProp<\"li\", \"key\">((i))]); })]",
        ),
        (
            "declare const ok: boolean; declare const xs: string[]; const v = <div>{ok ? xs.map((x) => <li title={x} />) : <p />}</div>;",
            "([ok ? xs.map((x) => [__vizeNativeElementProp<\"li\", \"title\">((x))]) : []]",
        ),
    ] {
        let head = source
            .get(..source.find("const v = <").unwrap() + 10)
            .unwrap();
        assert_eq!(
            projected(source),
            format!("{head}{root}{VNODE};\n;\nexport {{}};\n"),
            "{source}"
        );
    }
}

#[test]
fn conditions_are_expression_links_and_operators_are_script_runs() {
    use LinkMapping::{Anchor, Verbatim};
    use LinkRole::{Attribute, Expression, Script};
    let source = "declare const ok: boolean; declare const n: number; const v = <div>{ok ? <a title={n} /> : <b />}</div>;";
    let arena = Allocator::default();
    let module = support::lower(&arena, source, true).unwrap();
    let document = project_jsx_module(&module.program, &module.roots).unwrap();
    let root = "([ok ? [__vizeNativeElementProp<\"a\", \"title\">((n))] : []] as unknown as import(\"vue\").VNode)";
    let head = "declare const ok: boolean; declare const n: number; const v = ";
    assert_eq!(
        links(&document, source),
        [
            (head, head, Script, Verbatim),
            (
                root,
                "<div>{ok ? <a title={n} /> : <b />}</div>",
                Expression,
                Anchor
            ),
            (
                "ok ? [__vizeNativeElementProp<\"a\", \"title\">((n))] : []",
                "{ok ? <a title={n} /> : <b />}",
                Expression,
                Anchor,
            ),
            ("ok", "ok", Expression, Verbatim),
            (" ? ", " ? ", Script, Verbatim),
            (
                "[__vizeNativeElementProp<\"a\", \"title\">((n))]",
                "<a title={n} />",
                Expression,
                Anchor,
            ),
            (
                "__vizeNativeElementProp<\"a\", \"title\">((n))",
                "title={n}",
                Attribute,
                Anchor,
            ),
            ("n", "n", Expression, Verbatim),
            (" : ", " : ", Script, Verbatim),
            ("[]", "<b />", Expression, Anchor),
            (";", ";", Script, Verbatim),
        ]
    );
}

#[test]
fn a_child_conditional_must_span_its_container() {
    let source = "declare const ok: boolean; const v = <div>{ok ? <a /> : <b />}</div>;";
    let arena = Allocator::default();
    let mut module = support::lower(&arena, source, true).unwrap();
    let Some(Op::Element(element)) = module.roots[0].body.root.ops.first_mut() else {
        panic!("element root")
    };
    let Some(Op::If(conditional)) = element.children.ops.first_mut() else {
        panic!("child conditional")
    };
    conditional.span = Span::new(conditional.span.start + 1, conditional.span.end);
    assert_eq!(
        project_jsx_module(&module.program, &module.roots).map(|_| ()),
        Err(JsxModuleRefusal::Span)
    );
}
