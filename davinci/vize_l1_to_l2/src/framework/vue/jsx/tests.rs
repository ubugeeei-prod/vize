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
pub(super) fn lower(source: &str, exec: Exec) -> Result<vize_l0::String, JsxRefusal> {
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
        Op::ChildValue(child) => {
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
            BindingOp::Model(model) => {
                write!(out, " model={}", model.contract.read.source()).unwrap();
                if let Some(DynamicName::Static(argument)) = model.argument {
                    write!(out, ":{argument}").unwrap();
                }
                for attribute in &model.attributes {
                    match attribute.value {
                        Some(kind) => write!(out, "|{kind}").unwrap(),
                        None => write!(out, ".{}", attribute.name).unwrap(),
                    }
                }
            }
            BindingOp::SlotContent(slot) => write!(
                out,
                " #{:?}({:?})",
                slot.name,
                slot.params.map(|params| params.source())
            )
            .unwrap(),
            BindingOp::SlotsObject(slots) => {
                write!(out, " slots={}", slots.value.source()).unwrap()
            }
            BindingOp::VueShow(show) => write!(out, " show={}", show.value.source()).unwrap(),
            BindingOp::VueHtml(html) => {
                write!(out, " html={:?}", html.value.map(|v| v.source())).unwrap();
            }
            BindingOp::VueText(text) => {
                write!(out, " text={:?}", text.value.map(|v| v.source())).unwrap();
            }
            BindingOp::VueDirective(directive) => write!(
                out,
                " v-{}:{:?}{:?}={:?}",
                directive.name,
                directive.argument,
                directive.modifiers,
                directive.value.map(|v| v.source())
            )
            .unwrap(),
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
        "const A = <div v-once />;",
        "const A = <div v-if={x} />;",
        "const A = <div v-show:x={x} />;",
        "const A = <i v-show=\"x\" />;",
        "const A = <i v-show={<b />} />;",
        "const A = <a v-x:_b={v} />;",
        "const A = <a v-x_m={v} />;",
        "const A = <B v-custom={v} />;",
        "const A = <B v-html={h} />;",
        "const A = <B v-model:a_m={v} />;",
        "const A = <input v-model:a={v} />;",
        "const A = <input v-model_foo={v} />;",
        "const A = <input v-model={f()} />;",
        "const A = <input v-model={[a][0]} />;",
        "const A = <input type={t} v-model={v} />;",
        "const A = <input type=\"file\" v-model={v} />;",
        "const A = <input {...p} v-model={v} />;",
        "const A = <div v-model={v} />;",
        "const A = <B v-model />;",
        "const A = <B v-model={[a, b]} />;",
        "const A = <B v-model={[a, 'x', 'y']} />;",
        "const A = <B v-model_trim={[a]} />;",
        "const A = <B v-models={x} />;",
        "const A = <B v-models={[]} />;",
        "const A = <B v-models={[[a], [f()]]} />;",
        "const A = <div v-models={[[a]]} />;",
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
fn underscore_modifiers_split_events_and_directive_arguments() {
    assert_eq!(
        lower(
            "const A = <b onClick_stop_prevent={f} onKeyupCapture_enter={g} onSelect_item={h} />;",
            Exec::ReRender
        )
        .as_deref(),
        Ok("ops=4 <b @Some(Static(\"click\"))Vec([\"stop\", \"prevent\"]) \
             @Some(Static(\"keyup\"))Vec([\"capture\", \"enter\"]) :onSelect_item> ")
    );
    assert_eq!(
        lower("const A = <p v-mark:left_a_b={v} />;", Exec::ReRender).as_deref(),
        Ok("ops=2 <p v-mark:Some(Static(\"left\"))Vec([\"a\", \"b\"])=Some(\"v\")> ")
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

#[test]
fn models_and_directives_become_the_template_ops() {
    assert_eq!(
        lower(
            "const A = <p><input v-model={a.b} /><input type=\"checkbox\" v-model_lazy_trim={c} /><select v-model={[d, ['number']]} /></p>;",
            Exec::ReRender
        )
        .as_deref(),
        Ok("ops=7 <p> <input model=a.b|input> <input model=c|input.lazy.trim> <select model=d|select.number> ")
    );
    assert_eq!(
        lower(
            "import B from './b';\nconst A = <B v-model:title={t} v-model={[u, 'label', ['m']]} v-models={[[x], [y, 'y', ['n']]]} />;",
            Exec::ReRender
        )
        .as_deref(),
        Ok("ops=6 <C:component :is model=t:title|component model=u:label|component.m model=x|component model=y:y|component.n> ")
    );
    assert_eq!(
        lower(
            "const A = <Card v-show={on}><i v-html={h} /><b v-text={t} v-focus /></Card>;",
            Exec::ReRender
        )
        .as_deref(),
        Ok(
            "ops=7 <C:Card show=on> <i html=Some(\"h\")> <b text=Some(\"t\") v-focus:NoneVec([])=None> "
        )
    );
    assert_eq!(
        lower(
            "const A = <p><a v-mark:left={v} /><a v-mark={[v, 'top', ['a', 'b']]} /><a v-mark={[v, f]} /></p>;",
            Exec::ReRender
        )
        .as_deref(),
        Ok("ops=7 <p> <a v-mark:Some(Static(\"left\"))Vec([])=Some(\"v\")> <a v-mark:Some(Static(\"top\"))Vec([\"a\", \"b\"])=Some(\"v\")> <a v-mark:NoneVec([])=Some(\"[v, f]\")> ")
    );
}
