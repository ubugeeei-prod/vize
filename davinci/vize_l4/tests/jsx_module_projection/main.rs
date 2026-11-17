//! JSX/TSX modules projected from the L2 definitions of their markup roots.

mod checker;
mod control_flow;
mod support;

use vize_l0::{Allocator, Span};
use vize_l2::component::{Exec, RenderScope};
use vize_l2::expr::ExprRef;
use vize_l2::op::Op;
use vize_l4::targets::ts::{
    JsxModuleRefusal, LinkMapping, LinkRole, MappingError, TsDocument, project_jsx_module,
};

type Link<'d> = (&'d str, &'d str, LinkRole, LinkMapping);

/// Every link as its generated text, authored text and fact.
fn links<'d>(projected: &'d TsDocument, source: &'d str) -> Vec<Link<'d>> {
    let text = projected.document().as_str();
    let slice = |text: &'d str, span: Span| text.get(span.start as usize..span.end as usize);
    projected
        .document()
        .links()
        .iter()
        .zip(projected.link_facts())
        .map(|(link, fact)| {
            (
                slice(text, link.generated).unwrap_or("<generated>"),
                slice(source, link.authored).unwrap_or("<authored>"),
                fact.role,
                fact.mapping,
            )
        })
        .collect()
}

fn generated(projected: &TsDocument, text: &str) -> Span {
    let start = projected.document().as_str().find(text).unwrap();
    let start = u32::try_from(start).unwrap();
    Span::new(start, start + u32::try_from(text.len()).unwrap())
}

const MODULE: &str = "/*😀*/ const 日本語: string = 'é';\nexport const View = (props: { n: number; onTap(): void }) => <div title={日本語} onClickCapture={props.onTap} {...props}><span>{props.n}</span>text</div>;\nconst other = <><i/>{'x'}</>; // tail";

#[test]
fn roots_are_replaced_in_place_and_script_stays_verbatim() {
    use LinkMapping::{Anchor, Verbatim};
    use LinkRole::{Attribute, Event, Expression, Script};
    let arena = Allocator::default();
    let module = support::lower(&arena, MODULE, true).unwrap();
    let projected = project_jsx_module(&module.program, &module.roots).unwrap();
    let view = "([__vizeNativeElementProp<\"div\", \"title\">((日本語)), __vizeNativeElementProp<\"div\", \"onClick\">((props.onTap)), ({ ...(props) }), (props.n)] as unknown as import(\"vue\").VNode)";
    let other = "([] as unknown as import(\"vue\").VNode)";
    let head = "/*😀*/ const 日本語: string = 'é';\nexport const View = (props: { n: number; onTap(): void }) => ";
    assert_eq!(
        projected.document().as_str(),
        format!("{head}{view};\nconst other = {other}; // tail\n;\nexport {{}};\n")
    );
    assert_eq!(
        links(&projected, MODULE),
        [
            (head, head, Script, Verbatim),
            (
                view,
                "<div title={日本語} onClickCapture={props.onTap} {...props}><span>{props.n}</span>text</div>",
                Expression,
                Anchor,
            ),
            (
                "__vizeNativeElementProp<\"div\", \"title\">((日本語))",
                "title={日本語}",
                Attribute,
                Anchor,
            ),
            ("日本語", "日本語", Expression, Verbatim),
            (
                "__vizeNativeElementProp<\"div\", \"onClick\">((props.onTap))",
                "onClickCapture={props.onTap}",
                Event,
                Anchor,
            ),
            ("props.onTap", "props.onTap", Expression, Verbatim),
            ("({ ...(props) })", "{...props}", Attribute, Anchor),
            ("props", "props", Expression, Verbatim),
            ("props.n", "props.n", Expression, Verbatim),
            (";\nconst other = ", ";\nconst other = ", Script, Verbatim),
            (other, "<><i/>{'x'}</>", Expression, Anchor),
            ("; // tail", "; // tail", Script, Verbatim),
        ]
    );
    assert_eq!(projected.directives(), []);
}

#[test]
fn ranges_map_through_the_innermost_link_and_never_clamp() {
    let arena = Allocator::default();
    let module = support::lower(&arena, MODULE, true).unwrap();
    let projected = project_jsx_module(&module.program, &module.roots).unwrap();
    let authored = |text: &str| {
        let start = u32::try_from(MODULE.rfind(text).unwrap()).unwrap();
        Span::new(start, start + u32::try_from(text.len()).unwrap())
    };
    let read = generated(&projected, "props.onTap)");
    let member = Span::new(read.start + 6, read.start + 11);
    assert_eq!(
        projected.map_span(member),
        Ok(Span::new(
            authored("props.onTap").start + 6,
            authored("props.onTap").start + 11
        ))
    );
    // TypeScript reports a root's value inside its parentheses.
    let value = generated(&projected, "[] as unknown as import(\"vue\").VNode");
    assert_eq!(projected.map_span(value), Ok(authored("<><i/>{'x'}</>")));
    let check = generated(&projected, "__vizeNativeElementProp<\"div\", \"title\">");
    assert_eq!(projected.map_span(check), Ok(authored("title={日本語}")));
    let spread = generated(&projected, "...(props)");
    assert_eq!(projected.map_span(spread), Ok(authored("{...props}")));
    let end = Span::new(read.start + 11, read.start + 11);
    assert_eq!(
        projected.map_span(end),
        Ok(Span::new(
            authored("props.onTap").end,
            authored("props.onTap").end
        ))
    );
    assert_eq!(
        projected.map_span(Span::new(read.start + 6, read.end)),
        Err(MappingError::CrossesBoundary)
    );
    let length = u32::try_from(projected.document().as_str().len()).unwrap();
    assert_eq!(
        projected.map_span(Span::new(length - 3, length - 1)),
        Err(MappingError::GeneratedOnly)
    );
    assert_eq!(
        projected.map_span(Span::new(4, 3)),
        Err(MappingError::InvalidRange)
    );
    assert_eq!(projected.map_utf16(2, 2), Ok(Span::new(2, 6)));
    assert_eq!(
        projected.map_utf16(3, 0),
        Err(MappingError::InvalidUtf16Boundary)
    );
    assert_eq!(
        projected.map_utf16(u32::MAX, 1),
        Err(MappingError::InvalidUtf16Boundary)
    );
}

#[test]
fn unprojected_families_and_mismatched_roots_refuse_without_a_document() {
    for (source, refusal) in [
        (
            "const v = <Foo title={1} />;",
            JsxModuleRefusal::UnpublishedHelper("ui.component"),
        ),
        (
            "const C = () => null; const v = <div><C /></div>;",
            JsxModuleRefusal::UnpublishedHelper("ui.component"),
        ),
        (
            "declare const ok: boolean; const v = <div>{ok ? <Foo /> : <b />}</div>;",
            JsxModuleRefusal::UnpublishedHelper("ui.component"),
        ),
        (
            "declare const xs: string[]; const v = <ul>{xs.map((x) => <Item title={x} />)}</ul>;",
            JsxModuleRefusal::UnpublishedHelper("ui.component"),
        ),
        // Markup outside a region would be copied as script.
        (
            "const v = <ul>{[<a />].map((x) => <li />)}</ul>;",
            JsxModuleRefusal::NestedMarkup,
        ),
        (
            "const v = <p>{(<a />) ? <b /> : <i />}</p>;",
            JsxModuleRefusal::NestedMarkup,
        ),
    ] {
        let arena = Allocator::default();
        let module = support::lower(&arena, source, true).unwrap();
        assert_eq!(
            project_jsx_module(&module.program, &module.roots).map(|_| ()),
            Err(refusal),
            "{source}"
        );
    }
    let copy = String::from(MODULE);
    let arena = Allocator::default();
    let mut module = support::lower(&arena, MODULE, true).unwrap();
    let foreign = support::lower(&arena, &copy, true).unwrap();
    assert_eq!(
        project_jsx_module(&module.program, &[]).map(|_| ()),
        Err(JsxModuleRefusal::Roots)
    );
    assert_eq!(
        project_jsx_module(&module.program, module.roots.get(..1).unwrap()).map(|_| ()),
        Err(JsxModuleRefusal::Roots)
    );
    assert_eq!(
        project_jsx_module(&module.program, &foreign.roots).map(|_| ()),
        Err(JsxModuleRefusal::ForeignSource)
    );
    module.roots.swap(0, 1);
    assert_eq!(
        project_jsx_module(&module.program, &module.roots).map(|_| ()),
        Err(JsxModuleRefusal::Roots)
    );
}

/// The sole child value of the sole element root.
fn child_value<'m, 'a>(module: &'m mut support::Module<'a>) -> &'m mut ExprRef<'a> {
    let Some(Op::Element(element)) = module.roots[0].body.root.ops.first_mut() else {
        panic!("element root")
    };
    let Some(Op::JsxChildValue(child)) = element.children.ops.first_mut() else {
        panic!("child value")
    };
    &mut child.expression
}

#[test]
fn root_facts_and_expression_custody_are_checked() {
    let source = "declare const x: number; const v = <div>{x}</div>;";
    let arena = Allocator::default();
    let mut module = support::lower(&arena, source, true).unwrap();
    module.roots[0].exec = Exec::RunOnce;
    assert_eq!(
        project_jsx_module(&module.program, &module.roots).map(|_| ()),
        Err(JsxModuleRefusal::RunOnce)
    );
    module.roots[0].exec = Exec::ReRender;
    module.roots[0].scope = RenderScope::Instance;
    assert_eq!(
        project_jsx_module(&module.program, &module.roots).map(|_| ()),
        Err(JsxModuleRefusal::InstanceScope)
    );
    module.roots[0].scope = RenderScope::Lexical;
    let root = module.roots[0].span;
    // A verbatim expression never carries markup bytes.
    let markup = source.get(root.start as usize..root.end as usize).unwrap();
    *child_value(&mut module) = ExprRef::parse_js_in(&arena, markup, root);
    assert_eq!(
        project_jsx_module(&module.program, &module.roots).map(|_| ()),
        Err(JsxModuleRefusal::NestedMarkup)
    );
    // An expression's text must be the source at its own span.
    let at = u32::try_from(source.find("{x}").unwrap() + 1).unwrap();
    *child_value(&mut module) = ExprRef::parse_js_in(&arena, "y", Span::new(at, at + 1));
    assert_eq!(
        project_jsx_module(&module.program, &module.roots).map(|_| ()),
        Err(JsxModuleRefusal::Expression)
    );
    *child_value(&mut module) = ExprRef::parse_js_in(&arena, "x", Span::new(at, at + 1));
    let projected = project_jsx_module(&module.program, &module.roots).unwrap();
    assert_eq!(
        projected.document().as_str(),
        "declare const x: number; const v = ([(x)] as unknown as import(\"vue\").VNode);\n;\nexport {};\n"
    );
}
