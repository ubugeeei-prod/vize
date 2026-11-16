use alloc::vec::Vec;
use core::fmt::Write;

use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_l0::{Allocator, Span};
use vize_l2::component::Exec;

use super::{
    EnteredFunction, FunctionNode, JsxRefusal, JsxRoot, ModuleOptions, ModuleVisitor,
    PrologueIssue, RootContext, VIZE, lower_module, walk_module,
};

const OPTIONS: ModuleOptions = ModuleOptions {
    dialect: &VIZE,
    default_exec: Exec::ReRender,
};

/// Each root as `start..end name exec ops|refusal`, then each issue.
fn dump(source: &str, options: ModuleOptions) -> vize_l0::String {
    let allocator = Allocator::new();
    let parsed = Parser::new(allocator.as_oxc(), source, SourceType::tsx()).parse();
    assert_eq!(parsed.diagnostics.len(), 0, "{source}");
    let scoping = SemanticBuilder::new()
        .build(&parsed.program)
        .semantic
        .into_scoping();
    let source = allocator.alloc_str(source);
    let module = lower_module(&allocator, source, &parsed.program, Some(&scoping), options);
    let mut out = vize_l0::String::default();
    for root in module.roots() {
        let span = root.span();
        write!(
            out,
            "{}..{} {:?} {:?} ",
            span.start,
            span.end,
            root.name(),
            root.exec()
        )
        .unwrap();
        match root.component() {
            Ok(def) => {
                assert_eq!(
                    (def.span, def.name, def.exec),
                    (span, root.name(), root.exec())
                );
                writeln!(out, "ops={}", def.op_count).unwrap();
            }
            Err(refusal) => writeln!(out, "{refusal:?}").unwrap(),
        }
    }
    for issue in module.issues() {
        writeln!(out, "{issue:?}").unwrap();
    }
    out
}

#[test]
fn every_outermost_root_lowers_in_authored_order() {
    let source = "function App() { return <div>{ok ? <i /> : <b />}<p a={<s />} /></div>; }\n\
                  const List = () => xs.map((x) => <li>{x}</li>);\n\
                  export default { render() { return <><span /></>; } };";
    assert_eq!(
        dump(source, OPTIONS),
        "24..70 Some(\"App\") ReRender Attribute\n\
         107..119 Some(\"List\") ReRender ops=2\n\
         157..170 None ReRender ops=1\n"
    );
}

#[test]
fn the_nearest_prologue_selects_the_execution_model() {
    let source = "const Fast = () => {\n  \"use vue:vapor\";\n  const row = () => <i />;\n  return <div />;\n};\n\
                  function Slow() { \"use vue:vdom\"; return <p />; }";
    assert_eq!(
        dump(source, OPTIONS),
        "60..65 Some(\"row\") RunOnce Dialect\n\
         76..83 Some(\"Fast\") RunOnce Dialect\n\
         129..134 Some(\"Slow\") ReRender ops=1\n"
    );
    assert_eq!(
        dump(
            "const A = () => <i />;",
            ModuleOptions {
                default_exec: Exec::RunOnce,
                ..OPTIONS
            }
        ),
        "16..21 Some(\"A\") RunOnce Dialect\n"
    );
}

#[test]
fn unknown_and_conflicting_prologues_are_issues() {
    let source = "function A() { \"use vue:vdomx\"; \"use vue:vdom\"; \"use vue:vapor\"; \"use vue:vdom\"; return <i />; }";
    assert_eq!(
        dump(source, OPTIONS),
        "88..93 Some(\"A\") ReRender ops=1\n\
         Unknown { span: Span { start: 15, end: 30 }, directive: \"use vue:vdomx\" }\n\
         Conflict { span: Span { start: 48, end: 63 }, first: ReRender, second: RunOnce }\n"
    );
}

#[test]
fn components_are_every_definition_or_the_first_refusal() {
    let spans = |source: &str| {
        let allocator = Allocator::new();
        let parsed = Parser::new(allocator.as_oxc(), source, SourceType::jsx()).parse();
        lower_module(&allocator, source, &parsed.program, None, OPTIONS)
            .into_components()
            .map(|defs| defs.iter().map(|def| def.span).collect::<Vec<_>>())
    };
    assert_eq!(
        spans("const a = <i />;\nconst b = <><b /></>;"),
        Ok(alloc::vec![Span::new(10, 15), Span::new(27, 37)])
    );
    assert_eq!(
        spans("const a = <i />;\nconst b = <div v-show={x} />;\nconst c = <div>a {b}</div>;"),
        Err(JsxRefusal::Attribute)
    );
}

/// Every walk event as one line.
#[derive(Default)]
struct Events(vize_l0::String);

impl<'x> ModuleVisitor<'x> for Events {
    fn enter_function(&mut self, function: EnteredFunction<'_, 'x>) {
        let kind = match function.node {
            FunctionNode::Arrow(_) => "arrow",
            FunctionNode::Function(_) => "function",
        };
        writeln!(self.0, "enter {kind} {:?}", function.declaration).unwrap();
    }

    fn exit_function(&mut self) {
        self.0.push_str("exit\n");
    }

    fn prologue_issue(&mut self, issue: PrologueIssue<'x>) {
        writeln!(self.0, "{issue:?}").unwrap();
    }

    fn root(&mut self, root: JsxRoot<'_, 'x>, context: RootContext<'x>) {
        let kind = match root {
            JsxRoot::Element(_) => "element",
            JsxRoot::Fragment(_) => "fragment",
        };
        writeln!(self.0, "{kind} {context:?}").unwrap();
    }
}

#[test]
fn functions_claim_the_declaration_they_initialize_alone() {
    let source = "const A = () => { return <i />; };\nconst b = 1, C = () => <></>;\nconst D = wrap(() => <p />);";
    let allocator = Allocator::new();
    let parsed = Parser::new(allocator.as_oxc(), source, SourceType::jsx()).parse();
    let mut events = Events::default();
    walk_module(&parsed.program, &mut events);
    assert_eq!(
        events.0,
        "enter arrow Some(Span { start: 0, end: 34 })\n\
         element RootContext { span: Span { start: 25, end: 30 }, name: Some(\"A\"), prologue: None }\n\
         exit\n\
         enter arrow None\n\
         fragment RootContext { span: Span { start: 58, end: 63 }, name: Some(\"C\"), prologue: None }\n\
         exit\n\
         enter arrow None\n\
         element RootContext { span: Span { start: 86, end: 91 }, name: None, prologue: None }\n\
         exit\n"
    );
}
