use core::fmt::Write;

use oxc_ast::ast::{Expression, Statement};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_l0::Allocator;
use vize_l0::dialect::JsxSemantics;
use vize_l2::component::{Exec, RenderScope};
use vize_l2::op::{BindingOp, DynamicName, Op};

use super::{
    BABEL, JsxComponent, JsxDialect, JsxRefusal, JsxRoot, VIZE, VUE_JSX_VAPOR, clean_text,
    lower_component,
};

/// Lower the initializer of `const A = <root>` with Vize semantics.
#[expect(
    clippy::panic_in_result_fn,
    reason = "the helper asserts a clean parse and returns the refusal under test"
)]
fn lower(source: &str, exec: Exec) -> Result<vize_l0::String, JsxRefusal> {
    let allocator = Allocator::new();
    let parsed = Parser::new(allocator.as_oxc(), source, SourceType::jsx()).parse();
    assert!(parsed.diagnostics.is_empty(), "{:?}", parsed.diagnostics);
    let scoping = SemanticBuilder::new()
        .build(&parsed.program)
        .semantic
        .into_scoping();
    let root = parsed
        .program
        .body
        .iter()
        .find_map(|statement| match statement {
            Statement::VariableDeclaration(declaration) => {
                match declaration.declarations[0].init.as_ref()? {
                    Expression::JSXElement(element) => Some(JsxRoot::Element(element)),
                    Expression::JSXFragment(fragment) => Some(JsxRoot::Fragment(fragment)),
                    _ => None,
                }
            }
            _ => None,
        })
        .expect("a JSX root");
    let def = lower_component(
        &allocator,
        allocator.alloc_str(source),
        Some(&scoping),
        &VIZE,
        JsxComponent {
            name: Some("A"),
            exec,
        },
        root,
    )?;
    assert_eq!(def.scope, RenderScope::Lexical);
    let mut out = vize_l0::String::default();
    write!(out, "ops={} ", def.op_count).unwrap();
    for op in &def.body.root.ops {
        describe(op, &mut out);
    }
    Ok(out)
}

fn describe(op: &Op<'_>, out: &mut vize_l0::String) {
    let (bindings, children) = match op {
        Op::Element(element) => {
            write!(out, "<{}", element.tag).unwrap();
            (&element.bindings, &element.children)
        }
        Op::Component(component) => {
            write!(out, "<C:{}", component.name).unwrap();
            (&component.bindings, &component.children)
        }
        Op::Text(text) => {
            write!(out, "{:?} ", text.content).unwrap();
            return;
        }
        Op::JsxChildValue(child) => {
            write!(out, "{{{}}} ", child.expression.source()).unwrap();
            return;
        }
        other => panic!("unexpected {}", other.mnemonic()),
    };
    for binding in bindings {
        match binding {
            BindingOp::Bind(bind) => match bind.name {
                Some(DynamicName::Static(name)) => write!(out, " :{name}").unwrap(),
                None => out.push_str(" :..."),
                Some(DynamicName::Dynamic(_)) => out.push_str(" :[]"),
            },
            BindingOp::On(on) => write!(out, " @{:?}{:?}", on.name, on.modifiers).unwrap(),
            other => panic!("unexpected {}", other.mnemonic()),
        }
    }
    out.push_str("> ");
    for child in &children.ops {
        describe(child, out);
    }
}

#[test]
fn files_select_their_dialect_once() {
    assert_eq!(JsxDialect::for_semantics(JsxSemantics::Vize), &VIZE);
    assert_eq!(JsxDialect::for_semantics(JsxSemantics::Babel), &BABEL);
    assert_eq!(
        JsxDialect::for_semantics(JsxSemantics::VueJsxVapor),
        &VUE_JSX_VAPOR
    );
}

#[test]
fn elements_attributes_and_sole_child_values() {
    assert_eq!(
        lower(
            "const A = <div id=\"a\" class={c} {...rest} onClickCapture={h}>{count}</div>;",
            Exec::ReRender
        )
        .as_deref(),
        Ok("ops=5 <div :class :... @Some(Static(\"click\"))Vec([\"capture\"])> {count} ")
    );
}

#[test]
fn component_tags_resolve_through_lexical_bindings() {
    assert_eq!(
        lower(
            "import Bound from './b';\nconst A = <><Bound /><Free x={1} /><ui.Item /></>;",
            Exec::ReRender
        )
        .as_deref(),
        Ok("ops=6 <C:component :is> <C:Free :x> <C:component :is> ")
    );
}

#[test]
fn text_and_string_children_become_text_and_fragments_splice() {
    assert_eq!(
        lower(
            "const A = <p>\n  Hello\n  <b>{' '}</b><>x</>{/* c */}\n</p>;",
            Exec::ReRender
        )
        .as_deref(),
        Ok("ops=5 <p> \"Hello\" <b> \" \" \"x\" ")
    );
}

#[test]
fn unlanded_constructs_refuse_the_whole_root() {
    for source in [
        "const A = <div v-show={x} />;",
        "const A = <Comp vSlots={{ default: () => 'x' }} />;",
        "const A = <div vSlots={slots} />;",
        "const A = <B vSlots:default={slots} />;",
        "const A = <div>a {b}</div>;",
        "const A = <div>{ok && <i />}</div>;",
        "const A = <div>{...items}</div>;",
        "const A = <Comp>{x}</Comp>;",
        "const A = <div a={<i />} />;",
        "const A = <svg:rect />;",
        "const A = <main><style scoped>{`a{}`}</style></main>;",
    ] {
        assert!(lower(source, Exec::ReRender).is_err(), "{source}");
    }
    assert_eq!(
        lower("const A = <div />;", Exec::RunOnce),
        Err(JsxRefusal::Dialect)
    );
}

#[test]
fn text_cleaning_follows_the_babel_rule() {
    assert_eq!(
        clean_text("\n      Hello\n      World\n    "),
        "Hello World"
    );
    assert_eq!(clean_text("a   b"), "a   b");
    assert_eq!(clean_text("  hi  "), "  hi  ");
    assert_eq!(clean_text("\n   \n   \n"), "");
    assert_eq!(clean_text("   "), "   ");
    assert_eq!(clean_text("a\r\n  b"), "a b");
}
