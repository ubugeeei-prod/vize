//! Finding outermost JSX render roots and their component context.
//!
//! A JSX/TSX module can embed JSX anywhere an expression is allowed (arrow
//! bodies, `return` statements, ternaries, …). We treat every *outermost* JSX
//! element or fragment — one not nested inside another JSX node — as a render
//! root and lower it immediately, while the OXC node is still live, so no JSX
//! references escape the parse arena.
//!
//! While walking we maintain a stack of enclosing function scopes so each root
//! can record:
//! - the nearest `"use vue:vapor"` / `"use vue:vdom"` directive prologue, and
//! - the enclosing component function's name (`function App` or
//!   `const App = () => …`).

use oxc_ast::ast::{
    ArrowFunctionExpression, Function, FunctionBody, JSXElement, JSXFragment, Program,
    VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_syntax::scope::ScopeFlags;
use vize_carton::String;

use crate::LoweredRoot;
use crate::lower::Lowerer;
use crate::mode::JsxOutputMode;

/// Lower every outermost JSX root in `program` into a [`LoweredRoot`].
pub(crate) fn lower_program_roots<'a>(
    program: &Program<'_>,
    lowerer: &mut Lowerer<'a, '_, '_>,
) -> std::vec::Vec<LoweredRoot<'a>> {
    let mut collector = RootLowerer {
        lowerer,
        roots: std::vec::Vec::new(),
        scopes: std::vec::Vec::new(),
        pending_name: None,
    };
    collector.visit_program(program);
    collector.roots
}

/// An enclosing function scope.
struct FnScope {
    mode: Option<JsxOutputMode>,
    name: Option<String>,
}

struct RootLowerer<'l, 'a, 'm, 's> {
    lowerer: &'l mut Lowerer<'a, 'm, 's>,
    roots: std::vec::Vec<LoweredRoot<'a>>,
    scopes: std::vec::Vec<FnScope>,
    /// Name captured from a `const X = ...` declarator, claimed by the next
    /// function/arrow we enter.
    pending_name: Option<String>,
}

impl RootLowerer<'_, '_, '_, '_> {
    fn current_mode(&self) -> Option<JsxOutputMode> {
        self.scopes.iter().rev().find_map(|scope| scope.mode)
    }

    fn current_name(&self) -> Option<String> {
        self.scopes
            .iter()
            .rev()
            .find_map(|scope| scope.name.clone())
    }

    fn push_scope(&mut self, body: Option<&FunctionBody<'_>>, name: Option<String>) {
        let mode = body.and_then(body_mode);
        self.scopes.push(FnScope { mode, name });
    }
}

impl<'ast> Visit<'ast> for RootLowerer<'_, '_, '_, '_> {
    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'ast>) {
        // Capture `const App = ...` so an immediately-initialized function or
        // arrow can adopt the binding name.
        if let Some(name) = it.id.get_identifier_name() {
            self.pending_name = Some(String::from(name.as_str()));
        }
        walk::walk_variable_declarator(self, it);
        self.pending_name = None;
    }

    fn visit_function(&mut self, it: &Function<'ast>, flags: ScopeFlags) {
        let name = it
            .id
            .as_ref()
            .map(|id| String::from(id.name.as_str()))
            .or_else(|| self.pending_name.take());
        self.push_scope(it.body.as_deref(), name);
        walk::walk_function(self, it, flags);
        self.scopes.pop();
    }

    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'ast>) {
        let name = self.pending_name.take();
        self.push_scope(Some(&it.body), name);
        walk::walk_arrow_function_expression(self, it);
        self.scopes.pop();
    }

    fn visit_jsx_element(&mut self, element: &JSXElement<'ast>) {
        // Lower this root and intentionally do NOT descend: nested JSX is
        // lowered as part of this root's children, not as separate roots.
        let root = self.lowerer.lower_element_root(element);
        self.roots.push(LoweredRoot {
            root,
            mode: self.current_mode(),
            component_name: self.current_name(),
        });
    }

    fn visit_jsx_fragment(&mut self, fragment: &JSXFragment<'ast>) {
        let root = self.lowerer.lower_fragment_root(fragment);
        self.roots.push(LoweredRoot {
            root,
            mode: self.current_mode(),
            component_name: self.current_name(),
        });
    }
}

/// The first JSX output-mode directive in a function body's prologue, if any.
fn body_mode(body: &FunctionBody<'_>) -> Option<JsxOutputMode> {
    body.directives
        .iter()
        .find_map(|directive| JsxOutputMode::from_directive(directive.directive.as_str()))
}
