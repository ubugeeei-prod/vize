//! Navigation sites: every place a route location object reaches a router.
//!
//! Recognized sites, and nothing else:
//!
//! - `<receiver>.push|replace|resolve({ … })` in a script, where the
//!   receiver is proven to be a router: a `const` bound to `useRouter()`
//!   imported from `vue-router`, `useRouter()` itself, `this.$router`, the
//!   `const` a `createRouter` call initializes, or an import of a module
//!   export a router tree records;
//! - `$router.push|replace|resolve({ … })` in a template expression;
//! - `<RouterLink :to="{ … }">` / `<router-link v-bind:to="{ … }">`, unless
//!   the component name is rebound anywhere in the project.
//!
//! A location carrying a spread, a computed key, a `path`, or a non-literal
//! `name` is not a named navigation the provider can reason about and is
//! skipped whole.

mod location;

pub(super) use location::{Params, ValueKind};

use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, CallExpression, Expression, ObjectExpression};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use vize_carton::{CompactString, Span};
use vize_s1::{SurfaceChild, SurfaceTree};

use super::RouterTree;
use super::records::VUE_ROUTER;
use super::resolve::{Imported, Script, with_script};
use crate::providers::{ModuleId, ProjectModule, ProjectSources};

/// Which router a site navigates with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Target {
    /// One known router, by `RouteTree` key.
    Router(u32),
    /// The application's router (`useRouter()`, `$router`, `<RouterLink>`).
    App,
}

/// One named navigation.
#[derive(Debug)]
pub(super) struct NavSite {
    pub module: ModuleId,
    pub target: Target,
    pub name: CompactString,
    pub name_span: Span,
    pub params: Params,
}

const METHODS: &[&str] = &["push", "replace", "resolve"];

/// Every navigation site in `project`, in module then source order.
pub(super) fn collect(project: &ProjectSources, trees: &[(u32, &RouterTree)]) -> Vec<NavSite> {
    let link_rebound = project.modules().any(|(_, module)| {
        module.scripts().any(|block| {
            [
                "'RouterLink'",
                "\"RouterLink\"",
                "'router-link'",
                "\"router-link\"",
            ]
            .iter()
            .any(|literal| block.text.contains(literal))
        })
    });
    let mut sites = Vec::new();
    for (id, module) in project.modules() {
        for block in module.scripts() {
            if !METHODS.iter().any(|method| block.text.contains(method)) {
                continue;
            }
            let found = with_script(block, |script| {
                let mut finder = ScriptSites {
                    project,
                    module: id,
                    script,
                    trees,
                    sites: Vec::new(),
                };
                finder.visit_program(script.program);
                finder.sites
            });
            sites.extend(found.into_iter().flatten());
        }
        if let Some(template) = module.template() {
            let link_ok = !link_rebound && !rebinds_router_link(module);
            template_sites(id, template.text, template.offset, link_ok, &mut sites);
        }
    }
    sites
}

/// Whether a script of `module` names `RouterLink` anywhere but in a named
/// import from `vue-router`.
fn rebinds_router_link(module: &ProjectModule) -> bool {
    module.scripts().any(|block| {
        let mentions =
            block.text.matches("RouterLink").count() + block.text.matches("router-link").count();
        if mentions == 0 {
            return false;
        }
        let imported = with_script(block, |script| {
            script
                .program
                .body
                .iter()
                .filter_map(|statement| statement.as_module_declaration())
                .filter_map(|declaration| match declaration {
                    oxc_ast::ast::ModuleDeclaration::ImportDeclaration(import)
                        if import.source.value == VUE_ROUTER =>
                    {
                        Some(import)
                    }
                    _ => None,
                })
                .map(|import| {
                    let span = import.span;
                    script.program.source_text[span.start as usize..span.end as usize]
                        .matches("RouterLink")
                        .count()
                })
                .sum::<usize>()
        });
        imported != Some(mentions)
    })
}

struct ScriptSites<'p, 's, 'a> {
    project: &'p ProjectSources,
    module: ModuleId,
    script: &'s Script<'a>,
    trees: &'p [(u32, &'p RouterTree)],
    sites: Vec<NavSite>,
}

impl<'a> ScriptSites<'_, '_, 'a> {
    fn target(&self, receiver: &Expression<'a>) -> Option<Target> {
        match receiver.get_inner_expression() {
            Expression::CallExpression(call) => self.is_use_router(call).then_some(Target::App),
            Expression::StaticMemberExpression(member) => (member.property.name == "$router"
                && matches!(
                    member.object.get_inner_expression(),
                    Expression::ThisExpression(_)
                ))
            .then_some(Target::App),
            Expression::Identifier(ident) => {
                let symbol = self.script.symbol(ident)?;
                if let Some(init) = self.script.const_of(symbol) {
                    return match init.get_inner_expression() {
                        Expression::CallExpression(call) if self.is_use_router(call) => {
                            Some(Target::App)
                        }
                        init => {
                            let span = self.script.span(init.span());
                            self.router_where(|tree| {
                                tree.module == self.module && tree.span == span
                            })
                        }
                    };
                }
                let import = self.script.import_of(symbol)?;
                let name = match import.imported {
                    Imported::Named(name) => name,
                    Imported::Default => "default",
                    Imported::Namespace => return None,
                };
                let target = self.project.resolve(self.module, import.source)?;
                self.router_where(|tree| {
                    tree.module == target && tree.exports.iter().any(|export| export == name)
                })
            }
            _ => None,
        }
    }

    fn router_where(&self, matches: impl Fn(&RouterTree) -> bool) -> Option<Target> {
        self.trees
            .iter()
            .find(|(_, tree)| matches(tree))
            .map(|(key, _)| Target::Router(*key))
    }

    fn is_use_router(&self, call: &CallExpression<'a>) -> bool {
        call.arguments.is_empty()
            && matches!(call.callee.get_inner_expression(), Expression::Identifier(ident)
                if self.script.is_import_of(ident, VUE_ROUTER, "useRouter"))
    }
}

impl<'a> Visit<'a> for ScriptSites<'_, '_, 'a> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(member) = it.callee.get_inner_expression()
            && METHODS.contains(&member.property.name.as_str())
            && let Some(location) = location_argument(it)
            && let Some(target) = self.target(&member.object)
        {
            let script = self.script;
            let global =
                |name: &oxc_ast::ast::IdentifierReference<'a>| script.symbol(name).is_none();
            let span = |span: oxc_span::Span| script.span(span);
            if let Some(site) = location::named(self.module, target, location, &span, &global) {
                self.sites.push(site);
            }
        }
        walk::walk_call_expression(self, it);
    }
}

fn location_argument<'b, 'a>(call: &'b CallExpression<'a>) -> Option<&'b ObjectExpression<'a>> {
    match call.arguments.first()? {
        Argument::ObjectExpression(object) => Some(object),
        argument => match argument.as_expression()?.get_inner_expression() {
            Expression::ObjectExpression(object) => Some(object),
            _ => None,
        },
    }
}

fn template_sites(
    module: ModuleId,
    text: &str,
    offset: u32,
    link_ok: bool,
    sites: &mut Vec<NavSite>,
) {
    let mentions_link = text.contains("RouterLink") || text.contains("router-link");
    if !(link_ok && mentions_link) && !text.contains("$router") {
        return;
    }
    let allocator = vize_carton::Allocator::default();
    let (tree, _) = vize_s1::parse(&allocator, text);
    let mut walker = TemplateSites {
        tree: &tree,
        module,
        offset,
        link_ok,
        sites,
    };
    walker.children(&tree.children);
}

struct TemplateSites<'t, 'a, 'o> {
    tree: &'t SurfaceTree<'a>,
    module: ModuleId,
    offset: u32,
    link_ok: bool,
    sites: &'o mut Vec<NavSite>,
}

impl TemplateSites<'_, '_, '_> {
    fn children(&mut self, children: &[SurfaceChild<'_>]) {
        for child in children {
            let SurfaceChild::Element(element) = child else {
                continue;
            };
            let is_link = matches!(element.tag(), "RouterLink" | "router-link");
            for attr in element.open.attrs.iter() {
                let Some(value) = &attr.value else { continue };
                let name = attr.name.text;
                let content = value.content.text;
                let at = content.as_ptr() as usize - self.tree.source.as_ptr() as usize;
                if is_link && self.link_ok && matches!(name, ":to" | "v-bind:to") {
                    self.expression(content, at, true);
                } else if content.contains("$router")
                    && (name.starts_with('@')
                        || name.starts_with("v-on:")
                        || name.starts_with(':')
                        || name.starts_with("v-bind:"))
                {
                    self.expression(content, at, false);
                }
            }
            self.children(&element.children);
        }
    }

    fn expression(&mut self, content: &str, at: usize, is_to: bool) {
        let allocator = Allocator::default();
        let Ok(expression) = Parser::new(&allocator, content, SourceType::ts()).parse_expression()
        else {
            return;
        };
        let base = self.offset + at as u32;
        let span = |span: oxc_span::Span| Span::new(span.start + base, span.end + base);
        let global = |_: &oxc_ast::ast::IdentifierReference<'_>| true;
        if is_to {
            if let Expression::ObjectExpression(location) = expression.get_inner_expression()
                && let Some(site) =
                    location::named(self.module, Target::App, location, &span, &global)
            {
                self.sites.push(site);
            }
            return;
        }
        let mut calls = TemplateCalls {
            module: self.module,
            base,
            sites: self.sites,
        };
        calls.visit_expression(&expression);
    }
}

/// `$router.push|replace|resolve({ … })` inside one template expression.
struct TemplateCalls<'o> {
    module: ModuleId,
    base: u32,
    sites: &'o mut Vec<NavSite>,
}

impl<'a> Visit<'a> for TemplateCalls<'_> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        if let Expression::StaticMemberExpression(member) = it.callee.get_inner_expression()
            && METHODS.contains(&member.property.name.as_str())
            && member
                .object
                .get_inner_expression()
                .is_specific_id("$router")
            && let Some(location) = location_argument(it)
        {
            let base = self.base;
            let span = |span: oxc_span::Span| Span::new(span.start + base, span.end + base);
            let global = |_: &oxc_ast::ast::IdentifierReference<'_>| true;
            if let Some(site) = location::named(self.module, Target::App, location, &span, &global)
            {
                self.sites.push(site);
            }
        }
        walk::walk_call_expression(self, it);
    }
}
