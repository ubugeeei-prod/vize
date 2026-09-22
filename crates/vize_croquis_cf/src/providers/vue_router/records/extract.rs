//! Extracting one router's records from its `createRouter(...)` call.

use oxc_ast::ast::{
    ArrayExpressionElement, CallExpression, Expression, ObjectExpression, ObjectPropertyKind,
};
use oxc_span::GetSpan;
use vize_carton::{CompactString, Span};

use super::super::path::join_route_path;
use super::super::resolve::{Export, Import, Imported, Script, with_script};
use super::super::{OpenReason, RouteRecord, RouterTree};
use super::first_argument;
use super::router_options::{instance_exports, routes_property, static_string};
use crate::providers::{ModuleId, ModuleKind, ProjectSources};

/// How far `const`/import chains are followed before the tree is declared
/// open instead (also the cycle guard).
const MAX_DEPTH: u32 = 32;

/// Where a record sits: its parent record and the parent's full path.
enum Parent {
    Root,
    Known(u32, CompactString),
    Unknown(u32),
}

impl Parent {
    fn index(&self) -> Option<u32> {
        match self {
            Parent::Root => None,
            Parent::Known(index, _) | Parent::Unknown(index) => Some(*index),
        }
    }

    fn full_path(&self, path: &str) -> Option<CompactString> {
        match self {
            Parent::Root => Some(join_route_path(None, path)),
            Parent::Known(_, parent) => Some(join_route_path(Some(parent), path)),
            Parent::Unknown(_) => path.starts_with('/').then(|| CompactString::new(path)),
        }
    }
}

/// A property value that a later spread may have overridden.
pub(super) enum Slot<T> {
    Absent,
    Known(T),
    Unknown,
}

/// The tree of the router `call` creates.
pub(super) fn router<'a>(
    project: &ProjectSources,
    module: ModuleId,
    script: &Script<'a>,
    call: &CallExpression<'a>,
) -> RouterTree {
    let mut tree = RouterTree {
        module,
        span: script.span(call.span),
        records: Vec::new(),
        exports: instance_exports(script, call.span),
        open: Vec::new(),
    };
    let mut extractor = Extractor {
        project,
        tree: &mut tree,
        depth: 0,
    };
    let routes = match first_argument(call).map(Expression::get_inner_expression) {
        Some(Expression::ObjectExpression(options)) => routes_property(options),
        _ => Slot::Unknown,
    };
    match routes {
        Slot::Known(routes) => extractor.value(module, script, routes, &Parent::Root),
        Slot::Absent | Slot::Unknown => extractor.open_routes(module, script.span(call.span)),
    }
    tree
}

struct Extractor<'p, 't> {
    project: &'p ProjectSources,
    tree: &'t mut RouterTree,
    depth: u32,
}

impl Extractor<'_, '_> {
    fn open_routes(&mut self, module: ModuleId, span: Span) {
        self.tree
            .open
            .push(OpenReason::DynamicRoutes { module, span });
    }

    fn open_record(&mut self, module: ModuleId, span: Span) {
        self.tree
            .open
            .push(OpenReason::DynamicRecord { module, span });
    }

    /// A `routes`/`children` array, one record, or a binding of either.
    fn value<'a>(
        &mut self,
        module: ModuleId,
        script: &Script<'a>,
        expr: &Expression<'a>,
        parent: &Parent,
    ) {
        if self.depth >= MAX_DEPTH {
            return self.open_routes(module, script.span(expr.span()));
        }
        self.depth += 1;
        match expr.get_inner_expression() {
            Expression::ArrayExpression(array) => {
                for element in &array.elements {
                    match element {
                        ArrayExpressionElement::SpreadElement(spread) => {
                            self.value(module, script, &spread.argument, parent);
                        }
                        ArrayExpressionElement::Elision(_) => {}
                        element => match element.as_expression() {
                            Some(element) => self.value(module, script, element, parent),
                            None => self.open_record(module, script.span(element.span())),
                        },
                    }
                }
            }
            Expression::ObjectExpression(object) => self.record(module, script, object, parent),
            Expression::Identifier(ident) => {
                if let Some(init) = script.const_init(ident) {
                    self.value(module, script, init, parent);
                } else if let Some(import) = script.import(ident) {
                    self.imported(module, import, parent, script.span(ident.span));
                } else {
                    self.open_routes(module, script.span(ident.span));
                }
            }
            other => self.open_routes(module, script.span(other.span())),
        }
        self.depth -= 1;
    }

    fn imported(&mut self, from: ModuleId, import: Import<'_>, parent: &Parent, span: Span) {
        let target = self.project.resolve(from, import.source);
        let name = match import.imported {
            Imported::Named(name) => Some(name),
            Imported::Default => Some("default"),
            Imported::Namespace => None,
        };
        let (Some(target), Some(name)) = (target, name) else {
            return self.open_routes(from, span);
        };
        if self.depth >= MAX_DEPTH {
            return self.open_routes(from, span);
        }
        self.depth += 1;
        let mut found = false;
        for block in self.project.module(target).scripts() {
            let outcome = with_script(block, |script| match script.export(name) {
                Some(Export::Local(symbol)) => {
                    match script.const_of(symbol) {
                        Some(init) => self.value(target, script, init, parent),
                        None => self.open_routes(target, span),
                    }
                    true
                }
                Some(Export::Expr(expression)) => {
                    self.value(target, script, expression, parent);
                    true
                }
                Some(Export::From { source, name }) => {
                    let import = Import {
                        source,
                        imported: Imported::Named(name),
                    };
                    self.imported(target, import, parent, span);
                    true
                }
                None => false,
            });
            if outcome != Some(false) {
                found = outcome.is_some();
                break;
            }
        }
        if !found {
            self.open_routes(from, span);
        }
        self.depth -= 1;
    }

    fn record<'a>(
        &mut self,
        module: ModuleId,
        script: &Script<'a>,
        object: &ObjectExpression<'a>,
        parent: &Parent,
    ) {
        let mut name: Slot<(CompactString, Span)> = Slot::Absent;
        let mut path: Slot<CompactString> = Slot::Absent;
        let mut children: Slot<&Expression<'a>> = Slot::Absent;
        let mut components = Vec::new();
        for property in &object.properties {
            let property = match property {
                ObjectPropertyKind::ObjectProperty(property) if !property.computed => property,
                _ => {
                    (name, path, children) = (Slot::Unknown, Slot::Unknown, Slot::Unknown);
                    continue;
                }
            };
            match property.key.static_name().as_deref() {
                Some("name") => {
                    let value_span = script.span(property.value.span());
                    name = match static_string(&property.value) {
                        Some(literal) => Slot::Known((literal, value_span)),
                        None if property.value.get_inner_expression().is_undefined() => {
                            Slot::Absent
                        }
                        None => Slot::Unknown,
                    };
                }
                Some("path") => {
                    path = static_string(&property.value).map_or(Slot::Unknown, Slot::Known);
                }
                Some("children") => children = Slot::Known(&property.value),
                Some("component") => {
                    self.component(module, script, &property.value, &mut components)
                }
                Some("components") => {
                    if let Expression::ObjectExpression(views) =
                        property.value.get_inner_expression()
                    {
                        for view in &views.properties {
                            if let ObjectPropertyKind::ObjectProperty(view) = view {
                                self.component(module, script, &view.value, &mut components);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        let record_span = script.span(object.span);
        let (name, name_span) = match name {
            Slot::Known((name, span)) => (Some(name), span),
            Slot::Absent => (None, record_span),
            Slot::Unknown => {
                self.open_record(module, record_span);
                (None, record_span)
            }
        };
        let full_path = match &path {
            Slot::Known(path) => parent.full_path(path),
            Slot::Absent | Slot::Unknown => None,
        };
        let index = self.tree.records.len() as u32;
        self.tree.records.push(RouteRecord {
            name,
            name_span,
            path: full_path.clone(),
            module,
            parent: parent.index(),
            components,
        });
        match children {
            Slot::Known(children) => {
                let parent = match full_path {
                    Some(path) => Parent::Known(index, path),
                    None => Parent::Unknown(index),
                };
                self.value(module, script, children, &parent);
            }
            Slot::Absent => {}
            Slot::Unknown => self.open_routes(module, record_span),
        }
    }

    /// Record the `.vue` module a `component` slot renders: a default
    /// import or a `() => import('./X.vue')` of a project SFC.
    fn component<'a>(
        &mut self,
        module: ModuleId,
        script: &Script<'a>,
        value: &Expression<'a>,
        into: &mut Vec<ModuleId>,
    ) {
        let specifier = match value.get_inner_expression() {
            Expression::Identifier(ident) => script
                .import(ident)
                .filter(|import| import.imported == Imported::Default)
                .map(|import| import.source),
            Expression::ArrowFunctionExpression(arrow) if arrow.expression => arrow
                .get_expression()
                .and_then(|body| match body.get_inner_expression() {
                    Expression::ImportExpression(import) => match &import.source {
                        Expression::StringLiteral(source) => Some(source.value.as_str()),
                        _ => None,
                    },
                    _ => None,
                }),
            _ => None,
        };
        if let Some(target) =
            specifier.and_then(|specifier| self.project.resolve(module, specifier))
            && self.project.module(target).kind() == ModuleKind::Sfc
        {
            into.push(target);
        }
    }
}
