//! A whole JSX/TSX module. [`walk_module`] finds every outermost render root
//! in one walk over the parsed Program, with the context it renders in;
//! [`lower_module`] lowers each root into a [`ComponentDef`].
//!
//! A root is an element or fragment that no other markup encloses; markup
//! nested in it belongs to it. Its context comes from the functions around it:
//! the execution model from the nearest `"use vue:vapor"` / `"use vue:vdom"`
//! prologue (else the configured default), and the component name from the
//! nearest named function (`function App`, `const App = () => …`).

use alloc::vec::Vec;

use oxc_ast::ast::{
    ArrowFunctionExpression, Expression, Function, FunctionBody, JSXElement, JSXFragment, Program,
    VariableDeclaration, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::Scoping;
use oxc_syntax::scope::ScopeFlags;
use vize_l0::{Allocator, Span};
use vize_l2::module::{ComponentDef, Exec};

use super::{
    BooleanBindings, JsxComponent, JsxDialect, JsxFile, JsxRefusal, JsxRoot, lower_component, span,
};

/// The prefix every execution-model prologue starts with.
const PROLOGUE_PREFIX: &str = "use vue:";

/// A function the walk enters.
#[derive(Debug, Clone, Copy)]
pub enum FunctionNode<'r, 'x> {
    Arrow(&'r ArrowFunctionExpression<'x>),
    Function(&'r Function<'x>),
}

/// A function as the walk enters it.
#[derive(Debug, Clone, Copy)]
pub struct EnteredFunction<'r, 'x> {
    pub node: FunctionNode<'r, 'x>,
    /// The `const X = …` declaration this function initializes as its only
    /// declarator.
    pub declaration: Option<Span>,
}

/// Where an outermost root renders.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RootContext<'x> {
    /// The root's authored span.
    pub span: Span,
    /// The nearest enclosing named function.
    pub name: Option<&'x str>,
    /// The execution model of the nearest enclosing prologue.
    pub prologue: Option<Exec>,
}

/// A prologue that selects no execution model, reported when the walk enters
/// its function.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrologueIssue<'s> {
    /// `"use vue:…"` naming no execution model (`"use vue:vdomx"`).
    Unknown { span: Span, directive: &'s str },
    /// A prologue selecting another model than an earlier one, which wins.
    Conflict {
        span: Span,
        first: Exec,
        second: Exec,
    },
}

/// A caller's work at each step of [`walk_module`], done while the nodes are
/// live.
pub trait ModuleVisitor<'x> {
    /// Entering a function; [`Self::exit_function`] follows its body.
    fn enter_function(&mut self, _function: EnteredFunction<'_, 'x>) {}
    fn exit_function(&mut self) {}
    fn prologue_issue(&mut self, issue: PrologueIssue<'x>);
    /// An outermost root, in authored order.
    fn root(&mut self, root: JsxRoot<'_, 'x>, context: RootContext<'x>);
}

/// Visit every function, prologue issue and outermost root of `program`.
pub fn walk_module<'x>(program: &Program<'x>, visitor: &mut impl ModuleVisitor<'x>) {
    Walker {
        visitor,
        scopes: Vec::new(),
        pending_name: None,
        pending_declaration: None,
    }
    .visit_program(program);
}

/// How a module's roots lower.
#[derive(Debug, Clone, Copy)]
pub struct ModuleOptions {
    pub dialect: &'static JsxDialect,
    /// The execution model of a root under no prologue.
    pub default_exec: Exec,
}

/// One outermost root of a module and its lowering.
#[derive(Debug)]
pub struct ModuleRoot<'a> {
    span: Span,
    name: Option<&'a str>,
    exec: Exec,
    component: Result<ComponentDef<'a>, JsxRefusal>,
}

impl<'a> ModuleRoot<'a> {
    /// The root's authored span.
    #[must_use]
    pub const fn span(&self) -> Span {
        self.span
    }

    /// The component name, from the nearest enclosing named function.
    #[must_use]
    pub const fn name(&self) -> Option<&'a str> {
        self.name
    }

    #[must_use]
    pub const fn exec(&self) -> Exec {
        self.exec
    }

    /// The root's definition, or why it has none yet.
    pub fn component(&self) -> Result<&ComponentDef<'a>, JsxRefusal> {
        self.component.as_ref().map_err(|refusal| *refusal)
    }
}

/// A lowered module: its roots in authored order and its prologue issues.
#[derive(Debug)]
pub struct JsxModule<'a> {
    roots: Vec<ModuleRoot<'a>>,
    issues: Vec<PrologueIssue<'a>>,
}

impl<'a> JsxModule<'a> {
    #[must_use]
    pub fn roots(&self) -> &[ModuleRoot<'a>] {
        &self.roots
    }

    #[must_use]
    pub fn issues(&self) -> &[PrologueIssue<'a>] {
        &self.issues
    }

    /// Every root's definition in authored order, or the first refusal.
    pub fn into_components(self) -> Result<Vec<ComponentDef<'a>>, JsxRefusal> {
        self.roots.into_iter().map(|root| root.component).collect()
    }
}

/// Lower every outermost root of `program`, parsed from `source`.
pub fn lower_module<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    program: &Program<'_>,
    scoping: Option<&Scoping>,
    options: ModuleOptions,
) -> JsxModule<'a> {
    let booleans = BooleanBindings::collect(program, scoping);
    let mut lowering = Lowering {
        allocator,
        file: JsxFile {
            source,
            scoping,
            booleans: &booleans,
        },
        options,
        module: JsxModule {
            roots: Vec::new(),
            issues: Vec::new(),
        },
    };
    walk_module(program, &mut lowering);
    lowering.module
}

struct Lowering<'a, 's> {
    allocator: &'a Allocator,
    file: JsxFile<'a, 's>,
    options: ModuleOptions,
    module: JsxModule<'a>,
}

impl<'x> ModuleVisitor<'x> for Lowering<'_, '_> {
    fn prologue_issue(&mut self, issue: PrologueIssue<'x>) {
        self.module.issues.push(match issue {
            PrologueIssue::Unknown { span, directive } => PrologueIssue::Unknown {
                span,
                directive: self.allocator.alloc_str(directive),
            },
            PrologueIssue::Conflict {
                span,
                first,
                second,
            } => PrologueIssue::Conflict {
                span,
                first,
                second,
            },
        });
    }

    fn root(&mut self, root: JsxRoot<'_, 'x>, context: RootContext<'x>) {
        let allocator = self.allocator;
        let name = context.name.map(|name| allocator.alloc_str(name));
        let exec = context.prologue.unwrap_or(self.options.default_exec);
        let component = lower_component(
            allocator,
            &self.file,
            self.options.dialect,
            JsxComponent { name, exec },
            root,
        );
        self.module.roots.push(ModuleRoot {
            span: context.span,
            name,
            exec,
            component,
        });
    }
}

struct Scope<'x> {
    prologue: Option<Exec>,
    name: Option<&'x str>,
}

struct Walker<'v, 'x, V> {
    visitor: &'v mut V,
    scopes: Vec<Scope<'x>>,
    /// The name of a `const X = …` declarator, claimed by the function it
    /// initializes.
    pending_name: Option<&'x str>,
    /// The single-declarator declaration being walked, claimed likewise.
    pending_declaration: Option<Span>,
}

impl<'x, V: ModuleVisitor<'x>> Walker<'_, 'x, V> {
    fn enter(
        &mut self,
        node: FunctionNode<'_, 'x>,
        body: Option<&FunctionBody<'x>>,
        name: Option<&'x str>,
        declaration: Option<Span>,
    ) {
        let prologue = body.and_then(|body| self.prologue(body));
        self.visitor
            .enter_function(EnteredFunction { node, declaration });
        self.scopes.push(Scope { prologue, name });
    }

    fn exit(&mut self) {
        self.scopes.pop();
        self.visitor.exit_function();
    }

    /// The model `body`'s prologue selects. The first selecting prologue
    /// wins; a conflicting or unknown one is an issue.
    fn prologue(&mut self, body: &FunctionBody<'x>) -> Option<Exec> {
        let mut selected = None;
        for directive in &body.directives {
            let raw = directive.directive.as_str();
            let at = span(directive.expression.span);
            match (prologue_exec(raw), selected) {
                (Some(exec), None) => selected = Some(exec),
                (Some(second), Some(first)) if first != second => {
                    self.visitor.prologue_issue(PrologueIssue::Conflict {
                        span: at,
                        first,
                        second,
                    });
                }
                (Some(_), Some(_)) => {}
                (None, _) if raw.starts_with(PROLOGUE_PREFIX) => {
                    self.visitor.prologue_issue(PrologueIssue::Unknown {
                        span: at,
                        directive: raw,
                    });
                }
                (None, _) => {}
            }
        }
        selected
    }

    fn root(&mut self, root: JsxRoot<'_, 'x>, at: oxc_span::Span) {
        let context = RootContext {
            span: span(at),
            name: self.scopes.iter().rev().find_map(|scope| scope.name),
            prologue: self.scopes.iter().rev().find_map(|scope| scope.prologue),
        };
        self.visitor.root(root, context);
    }
}

fn prologue_exec(directive: &str) -> Option<Exec> {
    match directive {
        "use vue:vapor" => Some(Exec::RunOnce),
        "use vue:vdom" => Some(Exec::ReRender),
        _ => None,
    }
}

impl<'x, V: ModuleVisitor<'x>> Visit<'x> for Walker<'_, 'x, V> {
    fn visit_variable_declaration(&mut self, it: &VariableDeclaration<'x>) {
        let previous = self.pending_declaration;
        self.pending_declaration = (it.declarations.len() == 1).then_some(span(it.span));
        walk::walk_variable_declaration(self, it);
        self.pending_declaration = previous;
    }

    fn visit_variable_declarator(&mut self, it: &VariableDeclarator<'x>) {
        let declaration = self.pending_declaration;
        let initializes_function = it.init.as_ref().is_some_and(|init| {
            matches!(
                init.get_inner_expression(),
                Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
            )
        });
        if !initializes_function {
            self.pending_declaration = None;
        } else if let Some(name) = it.id.get_identifier_name() {
            self.pending_name = Some(name.as_str());
        }
        walk::walk_variable_declarator(self, it);
        self.pending_name = None;
        self.pending_declaration = declaration;
    }

    fn visit_function(&mut self, it: &Function<'x>, flags: ScopeFlags) {
        let pending = self.pending_name.take();
        let declaration = self.pending_declaration.take();
        let name = it.id.as_ref().map(|id| id.name.as_str()).or(pending);
        self.enter(
            FunctionNode::Function(it),
            it.body.as_deref(),
            name,
            declaration,
        );
        walk::walk_function(self, it, flags);
        self.exit();
    }

    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'x>) {
        let name = self.pending_name.take();
        let declaration = self.pending_declaration.take();
        self.enter(FunctionNode::Arrow(it), Some(&it.body), name, declaration);
        walk::walk_arrow_function_expression(self, it);
        self.exit();
    }

    // A root's nested markup is part of it, so the walk does not descend.
    fn visit_jsx_element(&mut self, it: &JSXElement<'x>) {
        self.root(JsxRoot::Element(it), it.span);
    }

    fn visit_jsx_fragment(&mut self, it: &JSXFragment<'x>) {
        self.root(JsxRoot::Fragment(it), it.span);
    }
}
