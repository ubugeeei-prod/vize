//! Compare all observable hooks with the original pinned OXC visitor, including
//! early returns. The reference keeps only spans, so it needs no widened borrow.

use super::super::{LentNodes, walk_program};
use oxc_allocator::Allocator;
use oxc_ast::ast::{CallExpression, JSXElement, JSXFragment};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

#[derive(Debug, PartialEq, Eq)]
struct Event(u8, u32, u32);

struct Trace {
    events: Vec<Event>,
    policy: u8,
}

impl Trace {
    fn new(policy: u8) -> Self {
        Self {
            events: Vec::new(),
            policy,
        }
    }

    fn enter(&mut self, kind: u8, span: Span) -> bool {
        self.record(kind, span);
        match self.policy {
            0 => true,
            1 => false,
            _ => span.start % 3 != 0,
        }
    }

    fn record(&mut self, kind: u8, span: Span) {
        self.events.push(Event(kind, span.start, span.end));
    }
}

impl<'a> LentNodes<'a> for Trace {
    fn enter_jsx_element(&mut self, element: &'a JSXElement<'a>) -> bool {
        self.enter(0, element.span)
    }

    fn leave_jsx_element(&mut self, element: &'a JSXElement<'a>) {
        self.record(1, element.span);
    }

    fn enter_jsx_fragment(&mut self, fragment: &'a JSXFragment<'a>) -> bool {
        self.enter(2, fragment.span)
    }

    fn leave_jsx_fragment(&mut self, fragment: &'a JSXFragment<'a>) {
        self.record(3, fragment.span);
    }

    fn call_expression(&mut self, call: &'a CallExpression<'a>) {
        self.record(4, call.span);
    }
}

struct Original(Trace);

impl<'a> Visit<'a> for Original {
    fn visit_jsx_element(&mut self, element: &JSXElement<'a>) {
        if self.0.enter(0, element.span) {
            walk::walk_jsx_element(self, element);
            self.0.record(1, element.span);
        }
    }

    fn visit_jsx_fragment(&mut self, fragment: &JSXFragment<'a>) {
        if self.0.enter(2, fragment.span) {
            walk::walk_jsx_fragment(self, fragment);
            self.0.record(3, fragment.span);
        }
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        self.0.record(4, call.span);
        walk::walk_call_expression(self, call);
    }
}

const SOURCES: &[&str] = &[
    super::SOURCE,
    "const view = <><stop>{nested(<x />)}</stop><keep /></>;",
    "export default defineComponent({ setup() { return () => <main>{format(value)}</main>; } });",
    "import value from 'package'; export function view<T>(x: T) { return condition(x) ? <yes attr={value(x)} /> : <no>{fallback()}</no>; }",
    "class View extends base(make()) { field = <field>{read()}</field>; method(value = init(<input />)) { return <method>{value}</method>; } static { register(<static />); } }",
    "const view = async () => { for (const value of values()) { await visit(<item key={key(value)} />); } try { return <ok />; } catch ({ message = getMessage() }) { throw failure(<error>{message}</error>); } finally { cleanup(); } };",
    "const { [compute()]: value = defaultValue(<fallback />), ...rest } = source(); ({ [key()]: target.field } = resolve()); const result = render?.(<optional />)?.value ?? fallback(<empty />);",
    "namespace UI { export class Widget { get view() { return <getter />; } set view(value = create()) { save(<setter>{value}</setter>); } } } const typed = (<type /> satisfies JSX.Element) as JSX.Element;",
    "const view = <UI.Widget {...props()} event={() => dispatch(<event />)}>{items.map(value => <>{format(value)}<leaf /></>)}</UI.Widget>;",
];

#[test]
fn all_hooks_match_the_pinned_oxc_walk_with_each_descent_policy() {
    for source in SOURCES {
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, source, SourceType::tsx()).parse();
        assert!(
            parsed.diagnostics.is_empty(),
            "{source}: {:?}",
            parsed.diagnostics
        );
        for policy in 0..3 {
            let mut original = Original(Trace::new(policy));
            original.visit_program(&parsed.program);
            let mut lent = Trace::new(policy);
            walk_program(&mut lent, &parsed.program);
            assert_eq!(lent.events, original.0.events, "policy {policy}: {source}");
            assert!(!lent.events.is_empty(), "the fixture exercises hooks");
        }
    }
}
