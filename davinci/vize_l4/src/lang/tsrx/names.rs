//! The names one component reads and declares, over the retained host AST of
//! its setup statements and of every embedded markup expression, the markup of
//! regions nested in those expressions included.
//! Type positions are skipped: they hold no runtime reads. A setup read that
//! a nested scope's declaration shadows is marked, so it is not read as a prop
//! ([`super::scopes`]); markup scopes are the DOM emitter's.

use alloc::vec::Vec;
use oxc_ast::ast::{
    ArrowFunctionExpression, BindingIdentifier, BlockStatement, CatchClause,
    ComputedMemberExpression, Expression, ForInStatement, ForOfStatement, ForStatement, Function,
    IdentifierReference, JSXFragment, ObjectProperty, Program, StaticMemberExpression,
    SwitchStatement, TSType, VariableDeclaration, VariableDeclarationKind,
};
use oxc_ast_visit::{Visit, walk};
use oxc_syntax::scope::ScopeFlags;
use vize_l0::Span;
use vize_l1::markup::tsrx::{
    Attribute, AttributeValue, Block, Child, Element, Node, RegionSite, TagName, TsrxModule,
};
use vize_l1_to_l2::framework::vue::tsrx::is_component;

use super::component::Component;
use super::scopes;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum Use<'a> {
    /// A plain read.
    Value,
    /// The object of `name.key` or `name["key"]`.
    Member(&'a str),
    /// A shorthand object property `{ name }`.
    Shorthand,
}

pub(super) struct Read<'a> {
    pub name: &'a str,
    pub span: Span,
    pub use_: Use<'a>,
    /// In a setup statement rather than in markup.
    pub setup: bool,
    /// In setup, under a nested scope that declares the name.
    pub shadowed: bool,
}

#[derive(Default)]
pub(super) struct Names<'a> {
    pub reads: Vec<Read<'a>>,
    /// Declarations of the component's own scope: its setup outside nested
    /// scopes. Markup declares only in nested scopes.
    declared: Vec<(&'a str, Span)>,
    /// The base names of component tags (`Foo` of `<Foo.Bar>`).
    tags: Vec<(&'a str, Span)>,
}

impl<'a> Names<'a> {
    /// Every name a component's setup and markup read, declare or use as a tag.
    pub(super) fn of_component(module: &'a TsrxModule<'a>, component: &Component<'a>) -> Self {
        let mut collector = Collector {
            names: Self::default(),
            source: module.program().source_text,
            module: Some(module),
            setup: component.body.is_some(),
            scopes: Vec::new(),
        };
        if let Some(body) = component.body {
            for statement in &body.statements {
                collector.visit_statement(statement);
            }
            collector.setup = false;
        }
        collector.node(&component.region.node);
        collector.names
    }

    /// Every name the host program reads or declares.
    pub(super) fn of_program(program: &'a Program<'a>) -> Vec<&'a str> {
        let mut collector = Collector {
            names: Self::default(),
            source: program.source_text,
            module: None,
            setup: false,
            scopes: Vec::new(),
        };
        collector.visit_program(program);
        let names = collector.names;
        names
            .reads
            .iter()
            .map(|read| read.name)
            .chain(names.declared.iter().map(|(name, _)| *name))
            .collect()
    }

    pub(super) fn value_reads<'s>(&'s self, name: &'s str) -> impl Iterator<Item = Span> + 's {
        self.reads
            .iter()
            .filter(move |read| {
                read.name == name && !read.shadowed && !matches!(read.use_, Use::Member(_))
            })
            .map(|read| read.span)
    }

    /// The keys read off `name`, first read first, each once.
    pub(super) fn member_keys(&self, name: &str) -> Vec<&'a str> {
        let mut keys: Vec<&'a str> = Vec::new();
        for read in &self.reads {
            if let Use::Member(key) = read.use_
                && read.name == name
                && !read.shadowed
                && !keys.contains(&key)
            {
                keys.push(key);
            }
        }
        keys
    }

    pub(super) fn declarations<'s>(&'s self, name: &'s str) -> impl Iterator<Item = Span> + 's {
        self.declared
            .iter()
            .filter(move |(declared, _)| *declared == name)
            .map(|(_, span)| *span)
    }

    pub(super) fn tags<'s>(&'s self, name: &'s str) -> impl Iterator<Item = Span> + 's {
        self.tags
            .iter()
            .filter(move |(tag, _)| *tag == name)
            .map(|(_, span)| *span)
    }

    /// Every name read or used as a tag, in first-use order, each once.
    pub(super) fn free(&self) -> Vec<&'a str> {
        let mut names: Vec<&'a str> = Vec::new();
        let all = self
            .reads
            .iter()
            .map(|read| read.name)
            .chain(self.tags.iter().map(|(tag, _)| *tag));
        for name in all {
            if !names.contains(&name) {
                names.push(name);
            }
        }
        names
    }
}

impl<'a> Collector<'a> {
    /// Visit a scope nested in the component's. Markup scopes are left to the
    /// DOM emitter, which resolves markup names itself.
    fn scoped(&mut self, function: bool, names: Vec<&'a str>, visit: impl FnOnce(&mut Self)) {
        if !self.setup {
            return visit(self);
        }
        self.scopes.push((function, names));
        visit(self);
        self.scopes.pop();
    }
}

struct Collector<'a> {
    names: Names<'a>,
    source: &'a str,
    /// Where nested regions are found; `None` stops at their placeholders.
    module: Option<&'a TsrxModule<'a>>,
    setup: bool,
    /// The names each setup scope nested in the component's declares, and
    /// whether it is a function's (where a `var` stops).
    scopes: Vec<(bool, Vec<&'a str>)>,
}

impl<'a> Collector<'a> {
    fn read(&mut self, it: &IdentifierReference<'a>, use_: Use<'a>) {
        let name = it.name.as_str();
        self.names.reads.push(Read {
            name,
            span: Span::new(it.span.start, it.span.end),
            use_,
            setup: self.setup,
            shadowed: self.scopes.iter().any(|(_, scope)| scope.contains(&name)),
        });
    }

    fn node(&mut self, node: &'a Node<'a>) {
        match node {
            Node::Element(element) => self.element(element),
            Node::Fragment(fragment) => self.children(&fragment.children),
            Node::CodeBlock(block) => self.block(block),
            Node::If(conditional) => {
                for branch in &conditional.branches {
                    if let Some(test) = &branch.test {
                        self.visit_expression(test);
                    }
                    self.block(&branch.body);
                }
            }
            Node::For(each) => {
                self.visit_expression(&each.right);
                self.visit_for_statement_left(&each.left);
                if let Some(index) = &each.index {
                    self.visit_binding_identifier(index);
                }
                if let Some(key) = &each.key {
                    self.visit_expression(key);
                }
                self.block(&each.body);
                if let Some(empty) = &each.empty {
                    self.block(empty);
                }
            }
            Node::Switch(switch) => {
                self.visit_expression(&switch.discriminant);
                for case in &switch.cases {
                    if let Some(test) = &case.test {
                        self.visit_expression(test);
                    }
                    self.block(&case.body);
                }
            }
            Node::Try(boundary) => {
                self.block(&boundary.body);
                if let Some(pending) = &boundary.pending {
                    self.block(pending);
                }
                if let Some(catch) = &boundary.catch {
                    if let Some(params) = &catch.params {
                        self.visit_formal_parameters(params);
                    }
                    self.block(&catch.body);
                }
            }
        }
    }

    /// Setup statements, then the rendered node.
    fn block(&mut self, block: &'a Block<'a>) {
        let outer = self.setup;
        self.setup = true;
        for statement in &block.statements {
            self.visit_statement(statement);
        }
        self.setup = outer;
        if let Some(render) = &block.render {
            self.node(render);
        }
    }

    fn element(&mut self, element: &'a Element<'a>) {
        match &element.name {
            TagName::Static(span) => {
                let tag = self
                    .source
                    .get(span.start as usize..span.end as usize)
                    .unwrap_or_default();
                if is_component(tag) {
                    let base = tag.split('.').next().unwrap_or(tag);
                    self.names.tags.push((base, *span));
                }
            }
            TagName::Dynamic { expression, .. } => self.visit_expression(expression),
        }
        for attribute in &element.attributes {
            match attribute {
                Attribute::Named {
                    value: Some(AttributeValue::Expression { expression, .. }),
                    ..
                }
                | Attribute::Shorthand { expression, .. }
                | Attribute::Spread { expression, .. } => self.visit_expression(expression),
                Attribute::Named { .. } => {}
            }
        }
        self.children(&element.children);
    }

    fn children(&mut self, children: &'a [Child<'a>]) {
        for child in children {
            match child {
                Child::Expression(child) => {
                    if let Some(expression) = &child.expression {
                        self.visit_expression(expression);
                    }
                }
                Child::Node(node) => self.node(node),
                Child::Text(_) | Child::Comment(_) => {}
            }
        }
    }
}

impl<'a> Visit<'a> for Collector<'a> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        self.read(it, Use::Value);
    }

    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        if self.setup && self.scopes.is_empty() {
            self.names
                .declared
                .push((it.name.as_str(), Span::new(it.span.start, it.span.end)));
        }
    }

    /// A function expression's own name, then its parameters (their defaults
    /// see only the parameters), then its body.
    fn visit_function(&mut self, it: &Function<'a>, _: ScopeFlags) {
        // A declaration's name binds in the enclosing scope.
        if let Some(id) = it.id.as_ref().filter(|_| it.is_declaration()) {
            self.visit_binding_identifier(id);
        }
        let name = it
            .id
            .as_ref()
            .filter(|_| it.is_expression())
            .map(|id| id.name.as_str());
        self.scoped(true, name.into_iter().collect(), |this| {
            this.scoped(true, scopes::params(&it.params), |this| {
                this.visit_formal_parameters(&it.params);
                if let Some(body) = &it.body {
                    this.scoped(true, scopes::body(&body.statements), |this| {
                        this.visit_function_body(body);
                    });
                }
            });
        });
    }

    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'a>) {
        self.scoped(true, scopes::params(&it.params), |this| {
            this.visit_formal_parameters(&it.params);
            this.scoped(true, scopes::body(&it.body.statements), |this| {
                this.visit_function_body(&it.body);
            });
        });
    }

    fn visit_block_statement(&mut self, it: &BlockStatement<'a>) {
        self.scoped(false, scopes::lexical(&it.body), |this| {
            walk::walk_block_statement(this, it);
        });
    }

    fn visit_switch_statement(&mut self, it: &SwitchStatement<'a>) {
        let names = it
            .cases
            .iter()
            .flat_map(|case| scopes::lexical(&case.consequent))
            .collect();
        self.scoped(false, names, |this| walk::walk_switch_statement(this, it));
    }

    fn visit_for_statement(&mut self, it: &ForStatement<'a>) {
        self.scoped(false, scopes::for_init(it.init.as_ref()), |this| {
            walk::walk_for_statement(this, it);
        });
    }

    fn visit_for_in_statement(&mut self, it: &ForInStatement<'a>) {
        self.scoped(false, scopes::for_left(&it.left), |this| {
            walk::walk_for_in_statement(this, it);
        });
    }

    fn visit_for_of_statement(&mut self, it: &ForOfStatement<'a>) {
        self.scoped(false, scopes::for_left(&it.left), |this| {
            walk::walk_for_of_statement(this, it);
        });
    }

    fn visit_catch_clause(&mut self, it: &CatchClause<'a>) {
        let mut names = Vec::new();
        if let Some(param) = &it.param {
            scopes::pattern(&param.pattern, &mut names);
        }
        self.scoped(false, names, |this| walk::walk_catch_clause(this, it));
    }

    /// A `var` outside every nested function declares in the component's scope.
    fn visit_variable_declaration(&mut self, it: &VariableDeclaration<'a>) {
        if self.setup
            && it.kind == VariableDeclarationKind::Var
            && !self.scopes.iter().any(|(function, _)| *function)
        {
            for declarator in &it.declarations {
                for id in declarator.id.get_binding_identifiers() {
                    self.names
                        .declared
                        .push((id.name.as_str(), Span::new(id.span.start, id.span.end)));
                }
            }
        }
        walk::walk_variable_declaration(self, it);
    }

    fn visit_static_member_expression(&mut self, it: &StaticMemberExpression<'a>) {
        match &it.object {
            Expression::Identifier(object) => {
                self.read(object, Use::Member(it.property.name.as_str()))
            }
            _ => walk::walk_static_member_expression(self, it),
        }
    }

    fn visit_computed_member_expression(&mut self, it: &ComputedMemberExpression<'a>) {
        match (&it.object, &it.expression) {
            (Expression::Identifier(object), Expression::StringLiteral(key)) => {
                self.read(object, Use::Member(key.value.as_str()));
            }
            _ => walk::walk_computed_member_expression(self, it),
        }
    }

    fn visit_object_property(&mut self, it: &ObjectProperty<'a>) {
        match &it.value {
            Expression::Identifier(value) if it.shorthand => self.read(value, Use::Shorthand),
            _ => walk::walk_object_property(self, it),
        }
    }

    fn visit_ts_type(&mut self, _: &TSType<'a>) {}

    /// A region nested in markup: its markup. A placeholder in setup is the
    /// returned region, visited as markup, or refused markup.
    fn visit_jsx_fragment(&mut self, it: &JSXFragment<'a>) {
        let region = self
            .module
            .filter(|_| !self.setup && it.children.is_empty())
            .and_then(|module| module.region(Span::new(it.span.start, it.span.end)))
            .filter(|region| region.site == RegionSite::Expression);
        match region {
            Some(region) => self.node(&region.node),
            None => walk::walk_jsx_fragment(self, it),
        }
    }
}
