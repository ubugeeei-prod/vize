//! Application injection is established by installation, never router discovery.
//!
//! Only an unconditional top-level Vue application with an imported SFC root
//! and one directly resolved router plugin proves an application route table.
//! Other plugins and conditional/factory installation stay unknown. Shared
//! modules retain only their independently proven app contexts; an unknown
//! context refuses the claim. Direct router navigation does not use this gate.

mod imports;

use oxc_ast::ast::{
    ArrowFunctionExpression, CallExpression, Class, ConditionalExpression, Expression, Function,
    LogicalExpression, Statement,
};
use oxc_ast_visit::{Visit, walk};
use oxc_semantic::ScopeFlags;
use oxc_span::GetSpan;
use vize_carton::{FxHashMap, FxHashSet, Span};

use super::RouterTree;
use super::resolve::{Imported, Script, with_script};
use crate::providers::{ModuleId, ModuleKind, ProjectSources};

/// Proven application contexts, each with exactly one installed router.
pub(super) fn collect(
    project: &ProjectSources,
    routers: &[(u32, &RouterTree)],
) -> FxHashMap<ModuleId, FxHashSet<u32>> {
    let mut owners: FxHashMap<ModuleId, Option<FxHashSet<u32>>> = FxHashMap::default();
    let mut unknown_root = false;
    let mut edges = vec![Vec::new(); project.len()];
    let mut applications = Vec::new();
    for (module, source) in project.modules() {
        for block in source.scripts() {
            if !block.text.contains("import")
                && !block.text.contains("use")
                && !block.text.contains("createApp")
                && !block.text.contains("createSSRApp")
            {
                continue;
            }
            let found = with_script(block, |script| {
                let mut finder = Finder {
                    project,
                    module,
                    script,
                    routers,
                    installed: FxHashMap::default(),
                    nested: 0,
                    unknown_root: false,
                };
                finder.visit_program(script.program);
                (
                    finder.installed,
                    finder.unknown_root,
                    imports::collect(project, module, script),
                )
            });
            let Some((installed, unresolved, imported)) = found else {
                unknown_root |=
                    block.text.contains("createApp") || block.text.contains("createSSRApp");
                continue;
            };
            unknown_root |= unresolved;
            edges[module.index()].extend(imported);
            applications.extend(installed.into_values());
        }
    }
    for installed in applications {
        let router = (!installed.unknown && installed.routers.len() == 1)
            .then(|| installed.routers.iter().next().copied())
            .flatten();
        let mut roots = vec![installed.root];
        if let Some(key) = router
            && let Some((_, tree)) = routers.iter().find(|(candidate, _)| *candidate == key)
        {
            roots.extend(
                tree.records
                    .iter()
                    .flat_map(|record| &record.components)
                    .copied(),
            );
        }
        for owned in imports::reachable(&edges, &roots) {
            owners
                .entry(owned)
                .and_modify(|existing| {
                    if let Some(key) = router
                        && let Some(keys) = existing.as_mut()
                    {
                        keys.insert(key);
                    } else {
                        *existing = None;
                    }
                })
                .or_insert_with(|| router.map(|key| FxHashSet::from_iter([key])));
        }
    }
    if unknown_root {
        return FxHashMap::default();
    }
    owners
        .into_iter()
        .filter_map(|(module, router)| router.map(|key| (module, key)))
        .collect()
}

struct Installed {
    root: ModuleId,
    routers: FxHashSet<u32>,
    unknown: bool,
}

struct Finder<'p, 's, 'a> {
    project: &'p ProjectSources,
    module: ModuleId,
    script: &'s Script<'a>,
    routers: &'p [(u32, &'p RouterTree)],
    installed: FxHashMap<Span, Installed>,
    nested: usize,
    unknown_root: bool,
}

impl<'a> Finder<'_, '_, 'a> {
    fn constructor(&self, call: &CallExpression<'a>) -> bool {
        self.creator(&call.callee, 0)
    }

    fn creator(&self, value: &Expression<'a>, depth: usize) -> bool {
        if depth > 16 {
            return false;
        }
        match value.get_inner_expression() {
            Expression::Identifier(ident) => {
                if let Some(init) = self.script.const_init(ident) {
                    return self.creator(init, depth + 1);
                }
                self.script.runtime_import(ident).is_some_and(|import| {
                    import.source == "vue"
                        && matches!(
                            import.imported,
                            Imported::Named("createApp" | "createSSRApp")
                        )
                })
            }
            Expression::StaticMemberExpression(member)
                if matches!(member.property.name.as_str(), "createApp" | "createSSRApp") =>
            {
                matches!(member.object.get_inner_expression(), Expression::Identifier(ident)
                    if self.script.runtime_import(ident).is_some_and(|import|
                        import.source == "vue" && import.imported == Imported::Namespace))
            }
            _ => false,
        }
    }

    fn root(&self, call: &CallExpression<'a>) -> Option<ModuleId> {
        let root = super::records::first_argument(call)?;
        let Expression::Identifier(ident) = root.get_inner_expression() else {
            return None;
        };
        let import = self.script.runtime_import(ident)?;
        if import.imported != Imported::Default {
            return None;
        }
        let root = self.project.resolve(self.module, import.source)?;
        (self.project.module(root)?.kind() == ModuleKind::Sfc).then_some(root)
    }

    fn application(&self, value: &Expression<'a>, depth: usize) -> Option<(Span, ModuleId)> {
        if depth > 16 {
            return None;
        }
        match value.get_inner_expression() {
            Expression::Identifier(ident) => {
                self.application(self.script.const_init(ident)?, depth + 1)
            }
            Expression::CallExpression(call) if self.constructor(call) => {
                Some((self.script.span(call.span), self.root(call)?))
            }
            Expression::CallExpression(call) => {
                let Expression::StaticMemberExpression(member) = call.callee.get_inner_expression()
                else {
                    return None;
                };
                (member.property.name == "use")
                    .then(|| self.application(&member.object, depth + 1))
                    .flatten()
            }
            _ => None,
        }
    }

    fn router(&self, value: &Expression<'a>, depth: usize) -> Option<u32> {
        if depth > 16 {
            return None;
        }
        let span = self.script.span(value.get_inner_expression().span());
        if let Some((key, _)) = self
            .routers
            .iter()
            .find(|(_, tree)| tree.module == self.module && tree.span == span)
        {
            return Some(*key);
        }
        let Expression::Identifier(ident) = value.get_inner_expression() else {
            return None;
        };
        if let Some(init) = self.script.const_init(ident) {
            return self.router(init, depth + 1);
        }
        let import = self.script.runtime_import(ident)?;
        let name = match import.imported {
            Imported::Named(name) => name,
            Imported::Default => "default",
            Imported::Namespace => return None,
        };
        let module = self.project.resolve(self.module, import.source)?;
        self.routers
            .iter()
            .find(|(_, tree)| {
                tree.module == module && tree.exports.iter().any(|export| export == name)
            })
            .map(|(key, _)| *key)
    }

    fn record(&mut self, span: Span, root: ModuleId, router: Option<u32>) {
        let installed = self.installed.entry(span).or_insert_with(|| Installed {
            root,
            routers: FxHashSet::default(),
            unknown: false,
        });
        installed.unknown |= self.nested > 0 || router.is_none();
        if let Some(router) = router {
            installed.routers.insert(router);
        }
    }
}

impl<'a> Visit<'a> for Finder<'_, '_, 'a> {
    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if self.constructor(call) {
            if let Some(root) = self.root(call) {
                let installed = self
                    .installed
                    .entry(self.script.span(call.span))
                    .or_insert_with(|| Installed {
                        root,
                        routers: FxHashSet::default(),
                        unknown: false,
                    });
                installed.unknown |= self.nested > 0;
            } else {
                self.unknown_root = true;
            }
        } else if let Expression::StaticMemberExpression(member) =
            call.callee.get_inner_expression()
            && member.property.name == "use"
            && let Some((span, root)) = self.application(&member.object, 0)
        {
            let router = (call.arguments.len() == 1)
                .then(|| {
                    super::records::first_argument(call).and_then(|value| self.router(value, 0))
                })
                .flatten();
            self.record(span, root, router);
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_statement(&mut self, statement: &Statement<'a>) {
        let nested = !matches!(
            statement,
            Statement::VariableDeclaration(_)
                | Statement::ExpressionStatement(_)
                | Statement::ImportDeclaration(_)
                | Statement::ExportNamedDeclaration(_)
                | Statement::ExportDefaultDeclaration(_)
        );
        self.nested += usize::from(nested);
        walk::walk_statement(self, statement);
        self.nested -= usize::from(nested);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        self.nested += 1;
        walk::walk_function(self, function, flags);
        self.nested -= 1;
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        self.nested += 1;
        walk::walk_arrow_function_expression(self, arrow);
        self.nested -= 1;
    }

    fn visit_class(&mut self, class: &Class<'a>) {
        self.nested += 1;
        walk::walk_class(self, class);
        self.nested -= 1;
    }

    fn visit_conditional_expression(&mut self, value: &ConditionalExpression<'a>) {
        self.nested += 1;
        walk::walk_conditional_expression(self, value);
        self.nested -= 1;
    }

    fn visit_logical_expression(&mut self, value: &LogicalExpression<'a>) {
        self.nested += 1;
        walk::walk_logical_expression(self, value);
        self.nested -= 1;
    }
}
