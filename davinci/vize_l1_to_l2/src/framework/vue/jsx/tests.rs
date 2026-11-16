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
    BABEL, BooleanBindings, JsxComponent, JsxDialect, JsxFile, JsxRefusal, JsxRoot, VIZE,
    VUE_JSX_VAPOR, clean_text, lower_component,
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
    let booleans = BooleanBindings::collect(&parsed.program, Some(&scoping));
    let def = lower_component(
        &allocator,
        &JsxFile {
            source: allocator.alloc_str(source),
            scoping: Some(&scoping),
            booleans: &booleans,
        },
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
    for (id, facts) in def.body.scopes.sorted_entries() {
        write!(out, "scope@{}=[", id.index()).unwrap();
        for binding in &facts.bindings {
            write!(out, "{:?} ", binding.name).unwrap();
        }
        out.push_str("] ");
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
        Op::If(if_op) => {
            out.push_str("if[ ");
            for branch in &if_op.branches {
                match &branch.condition {
                    Some(condition) => write!(out, "{}? ", condition.source()).unwrap(),
                    None => out.push_str("else: "),
                }
                for child in &branch.region.ops {
                    describe(child, out);
                }
            }
            out.push_str("] ");
            return;
        }
        Op::For(for_op) => {
            let binding = &for_op.binding;
            write!(out, "for[{:?}", binding.value.source()).unwrap();
            if let Some(key) = &binding.key {
                write!(out, ",{:?}", key.source()).unwrap();
            }
            write!(out, " in {}: ", binding.source.source()).unwrap();
            for child in &for_op.region.ops {
                describe(child, out);
            }
            out.push_str("] ");
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
        "const A = <div>{ok ? <i /> : null}</div>;",
        "const A = <div>{ok ? <><i /></> : <b />}</div>;",
        "const A = <div>{(a > b) && <></>}</div>;",
        "const A = <ul>{xs.map(x => { const y = x; return <li>{y}</li>; })}</ul>;",
        "const A = <ul>{xs.map((x, i, all) => <li>{all}</li>)}</ul>;",
        "const A = <ul>{xs.map?.(x => <li />)}</ul>;",
        "const A = <ul>{xs.map(x => <li />)}{y}</ul>;",
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

#[test]
fn boolean_and_and_ternary_chains_become_ui_if() {
    assert_eq!(
        lower(
            "const on = a > 1;\nconst A = <div>{on && <i />}{!x && <b />}</div>;",
            Exec::ReRender
        )
        .as_deref(),
        Ok("ops=5 <div> if[ on? <i> ] if[ !x? <b> ] ")
    );
    assert_eq!(
        lower(
            "const A = <p>{a ? <i /> : (b) ? <b /> : <s />}</p>;",
            Exec::ReRender
        )
        .as_deref(),
        Ok("ops=5 <p> if[ a? <i> (b)? <b> else: <s> ] ")
    );
    assert_eq!(
        lower(
            "const A = <p>{a ? (b ? <i /> : <b />) : <s />}</p>;",
            Exec::ReRender
        )
        .as_deref(),
        Ok("ops=6 <p> if[ a? if[ b? <i> else: <b> ] else: <s> ] ")
    );
}

#[test]
fn map_calls_become_ui_for_with_their_parameters_in_scope() {
    assert_eq!(
        lower(
            "const A = <ul>{xs.map((x, i) => <li key={i}>{x}</li>)}</ul>;",
            Exec::ReRender
        )
        .as_deref(),
        Ok("ops=5 <ul> for[\"x\",\"i\" in xs: <li :key> {x} ] scope@1=[\"x\" \"i\" ] ")
    );
    assert_eq!(
        lower(
            "const A = <ul>{xs.map(function ({ id }) { return <li />; })}</ul>;",
            Exec::ReRender
        )
        .as_deref(),
        Ok("ops=3 <ul> for[\"{ id }\" in xs: <li> ] scope@1=[] ")
    );
    assert_eq!(
        lower(
            "const A = <ul>{rows.map(() => ok ? <i /> : <b />)}</ul>;",
            Exec::ReRender
        )
        .as_deref(),
        Ok("ops=5 <ul> for[\"\" in rows: if[ ok? <i> else: <b> ] ] scope@1=[] ")
    );
}

#[test]
fn component_children_admit_control_flow_only() {
    assert_eq!(
        lower(
            "import Card from './c';\nconst A = <Card>{a ? <i /> : <b />}</Card>;",
            Exec::ReRender
        )
        .as_deref(),
        Ok("ops=5 <C:component :is> if[ a? <i> else: <b> ] ")
    );
    for source in [
        "const A = <Card>{() => <i />}</Card>;",
        "const A = <Card>{{ default: () => <i /> }}</Card>;",
    ] {
        assert_eq!(
            lower(source, Exec::ReRender),
            Err(JsxRefusal::Child),
            "{source}"
        );
    }
}

#[test]
fn value_children_that_are_not_control_flow_stay_jsx_child_values() {
    assert_eq!(
        lower("const A = <p>{a ? 'x' : 'y'}{}</p>;", Exec::ReRender),
        Err(JsxRefusal::Child)
    );
    assert_eq!(
        lower("const A = <p>{xs.map(format)}</p>;", Exec::ReRender).as_deref(),
        Ok("ops=2 <p> {xs.map(format)} ")
    );
    assert_eq!(
        lower("const A = <p>{a || b}</p>;", Exec::ReRender).as_deref(),
        Ok("ops=2 <p> {a || b} ")
    );
}
