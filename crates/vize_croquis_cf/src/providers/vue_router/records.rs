//! `RouteTree` production: find every `createRouter` call and `addRoute`
//! call in the project, then extract each router's records.
//!
//! A module whose text names `createRouter` is **accounted for** only when
//! every occurrence is either an import specifier from `vue-router` or the
//! callee of a recognized call. Anything else — a wrapper imported from
//! elsewhere, `vue-router/auto`, a re-export, a comment, a module that does
//! not parse — means a router may exist that the provider cannot see, so
//! every tree is marked open with [`OpenReason::DynamicRoutes`] there.

pub(super) mod extract;
pub(super) mod router_options;

use oxc_ast::ast::{
    Argument, CallExpression, Expression, ImportDeclaration, ImportDeclarationSpecifier,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::GetSpan;
use vize_carton::Span;

use super::resolve::{Imported, Script, with_script};
use super::{OpenReason, RouterTree};
use crate::providers::{ModuleId, ProjectSources};

pub(super) const VUE_ROUTER: &str = "vue-router";

/// Every router in `project`, in module order.
pub(super) fn collect(project: &ProjectSources) -> Vec<RouterTree> {
    let mut trees = Vec::new();
    let mut global_open = Vec::new();
    for (module, source) in project.modules() {
        for block in source.scripts() {
            let names_router = block.text.contains("createRouter");
            if !names_router && !block.text.contains("addRoute") {
                continue;
            }
            let whole = Span::new(block.offset, block.offset + block.text.len() as u32);
            let found = with_script(block, |script| {
                let mut finder = Finder {
                    project,
                    module,
                    script,
                    trees: Vec::new(),
                    open: Vec::new(),
                    accounted: 0,
                };
                finder.visit_program(script.program);
                let accounted = finder.accounted == block.text.matches("createRouter").count();
                (finder.trees, finder.open, accounted)
            });
            match found {
                Some((local, open, accounted)) => {
                    trees.extend(local);
                    global_open.extend(open);
                    if !accounted {
                        global_open.push(OpenReason::DynamicRoutes {
                            module,
                            span: whole,
                        });
                    }
                }
                None if names_router => {
                    global_open.push(OpenReason::DynamicRoutes {
                        module,
                        span: whole,
                    });
                }
                None => global_open.push(OpenReason::AddRoute {
                    module,
                    span: whole,
                }),
            }
        }
    }
    for tree in &mut trees {
        tree.open.extend(global_open.iter().copied());
    }
    trees
}

/// Extracts recognized `createRouter(…)` calls and records every
/// `.addRoute(…)` call.
struct Finder<'p, 's, 'a> {
    project: &'p ProjectSources,
    module: ModuleId,
    script: &'s Script<'a>,
    trees: Vec<RouterTree>,
    open: Vec<OpenReason>,
    /// `createRouter` occurrences recognized so far.
    accounted: usize,
}

impl<'a> Finder<'_, '_, 'a> {
    fn is_create_router(&self, callee: &Expression<'a>) -> bool {
        match callee.get_inner_expression() {
            Expression::Identifier(ident) => {
                self.script.is_import_of(ident, VUE_ROUTER, "createRouter")
            }
            Expression::StaticMemberExpression(member) => {
                member.property.name == "createRouter"
                    && matches!(member.object.get_inner_expression(),
                        Expression::Identifier(object) if self.script.import(object)
                            .is_some_and(|import| import.source == VUE_ROUTER
                                && import.imported == Imported::Namespace))
            }
            _ => false,
        }
    }
}

impl<'a> Visit<'a> for Finder<'_, '_, 'a> {
    fn visit_import_declaration(&mut self, it: &ImportDeclaration<'a>) {
        if it.source.value != VUE_ROUTER {
            return;
        }
        for specifier in it.specifiers.iter().flatten() {
            if let ImportDeclarationSpecifier::ImportSpecifier(named) = specifier
                && named.imported.name() == "createRouter"
            {
                let span = specifier.span();
                let text = &self.script.program.source_text[span.start as usize..span.end as usize];
                self.accounted += text.matches("createRouter").count();
            }
        }
    }

    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if self.is_create_router(&it.callee) {
            let callee = it.callee.span();
            let text = &self.script.program.source_text[callee.start as usize..callee.end as usize];
            self.accounted += text.matches("createRouter").count();
            let tree = extract::router(self.project, self.module, self.script, it);
            self.trees.push(tree);
        } else if let Some(member) = it.callee.get_inner_expression().as_member_expression()
            && member.static_property_name() == Some("addRoute")
        {
            self.open.push(OpenReason::AddRoute {
                module: self.module,
                span: self.script.span(it.span),
            });
        }
        walk::walk_call_expression(self, it);
    }
}

/// The first argument of `call`, when it is a plain expression.
pub(super) fn first_argument<'b, 'a>(call: &'b CallExpression<'a>) -> Option<&'b Expression<'a>> {
    match call.arguments.first()? {
        Argument::SpreadElement(_) => None,
        argument => argument.as_expression(),
    }
}
