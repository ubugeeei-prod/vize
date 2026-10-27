//! Read only the caller's existing AST/semantic snapshot; no parser or I/O.
use super::{
    BindingRef, BindingTarget, LocalSymbol, Namespace, SourceFacts, SourceRevision, Usage,
};
use oxc_ast::{
    AstKind,
    ast::{
        BindingPattern, Declaration, Function, Program, Statement, TSTypeAliasDeclaration,
        VariableDeclaration, VariableDeclarationKind,
    },
};
use oxc_semantic::Semantic;
use std::collections::VecDeque;
use vize_carton::{FxHashMap, String};

pub(super) struct Collector<'s, 'a> {
    pub(super) semantic: &'s Semantic<'a>,
    pub(super) facts: SourceFacts,
    pending: VecDeque<LocalSymbol>,
    declaration_file: bool,
}

/// `global_script` is the host's effective module classification, including
/// moduleDetection and package context, not a filename/spelling heuristic.
pub fn collect<'a>(
    program: &Program<'a>,
    semantic: &Semantic<'a>,
    revision: SourceRevision,
    global_script: bool,
    requested: &[LocalSymbol],
) -> SourceFacts {
    let mut collector = Collector {
        semantic,
        facts: SourceFacts {
            revision,
            bindings: FxHashMap::default(),
            globals: Vec::new(),
        },
        pending: VecDeque::new(),
        declaration_file: program.source_type.is_typescript_definition(),
    };
    collector.pending.extend(requested.iter().copied());
    for statement in &program.body {
        collector.statement(statement, global_script);
    }
    while let Some(local) = collector.pending.pop_front() {
        if collector.facts.bindings.contains_key(&local) {
            continue;
        }
        let target = collector.symbol_target(local);
        collector.retain_aliases(&target);
        collector.facts.bindings.insert(local, target);
    }
    collector.facts
}
impl<'a> Collector<'_, 'a> {
    fn global(
        &mut self,
        name: &str,
        symbol: Option<oxc_syntax::symbol::SymbolId>,
        namespace: Namespace,
    ) {
        if let Some(symbol) = symbol {
            let local = LocalSymbol { symbol, namespace };
            self.facts.globals.push((String::from(name), local));
            self.pending.push_back(local);
        }
    }
    fn variables(&mut self, declaration: &VariableDeclaration<'a>) {
        for declarator in &declaration.declarations {
            if let BindingPattern::BindingIdentifier(id) = &declarator.id {
                self.global(id.name.as_str(), id.symbol_id.get(), Namespace::Value);
            }
        }
    }
    fn function(&mut self, function: &Function<'a>) {
        if let Some(id) = &function.id {
            self.global(id.name.as_str(), id.symbol_id.get(), Namespace::Value);
        }
    }
    fn type_alias(&mut self, alias: &TSTypeAliasDeclaration<'a>) {
        self.global(
            alias.id.name.as_str(),
            alias.id.symbol_id.get(),
            Namespace::Type,
        );
    }
    fn statement(&mut self, statement: &Statement<'a>, global: bool) {
        match statement {
            Statement::TSGlobalDeclaration(declaration) => {
                for statement in &declaration.body.body {
                    self.statement(statement, true);
                }
            }
            Statement::VariableDeclaration(declaration) if global => self.variables(declaration),
            Statement::FunctionDeclaration(function) if global => self.function(function),
            Statement::TSTypeAliasDeclaration(alias) if global => self.type_alias(alias),
            Statement::ExportNamedDeclaration(export) => {
                if let Some(declaration) = &export.declaration {
                    match declaration {
                        Declaration::TSGlobalDeclaration(declaration) => {
                            for statement in &declaration.body.body {
                                self.statement(statement, true);
                            }
                        }
                        Declaration::VariableDeclaration(declaration) if global => {
                            self.variables(declaration)
                        }
                        Declaration::FunctionDeclaration(function) if global => {
                            self.function(function)
                        }
                        Declaration::TSTypeAliasDeclaration(alias) if global => {
                            self.type_alias(alias)
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    fn retain_aliases(&mut self, target: &BindingTarget) {
        let mut pending = Vec::new();
        let mut first = Some(target);
        while let Some(target) = first.take().or_else(|| pending.pop()) {
            match target {
                BindingTarget::Alias { binding, .. }
                    if binding.source == self.facts.revision.source =>
                {
                    self.pending.push_back(binding.local)
                }
                BindingTarget::Candidates(targets) => pending.extend(targets),
                _ => {}
            }
        }
    }
    fn symbol_target(&self, local: LocalSymbol) -> BindingTarget {
        let mut declarations = self
            .semantic
            .scoping()
            .symbol_declarations(local.symbol)
            .peekable();
        let Some(first) = declarations.next() else {
            return BindingTarget::Opaque;
        };
        if declarations.peek().is_none() {
            return self.declaration_target(local, first);
        }
        BindingTarget::Candidates(
            std::iter::once(first)
                .chain(declarations)
                .map(|node| self.declaration_target(local, node))
                .collect(),
        )
    }
    fn is_ambient(&self, node: oxc_syntax::node::NodeId) -> bool {
        self.declaration_file
            || self
                .semantic
                .nodes()
                .ancestor_kinds(node)
                .any(|kind| match kind {
                    AstKind::VariableDeclaration(declaration) => declaration.declare,
                    AstKind::TSGlobalDeclaration(_) => true,
                    AstKind::TSModuleDeclaration(module) => module.declare,
                    _ => false,
                })
    }
    fn declaration_target(
        &self,
        local: LocalSymbol,
        node: oxc_syntax::node::NodeId,
    ) -> BindingTarget {
        let node = self.semantic.nodes().get_node(node);
        match node.kind() {
            AstKind::ImportSpecifier(import) => {
                let AstKind::ImportDeclaration(declaration) =
                    self.semantic.nodes().parent_kind(node.id())
                else {
                    return BindingTarget::Opaque;
                };
                BindingTarget::Import {
                    module: String::from(declaration.source.value.as_str()),
                    exported: String::from(import.imported.name().as_str()),
                    namespace: local.namespace,
                    type_only: import.import_kind.is_type() || declaration.import_kind.is_type(),
                    usage: None,
                }
            }
            AstKind::ImportDefaultSpecifier(_) => {
                let AstKind::ImportDeclaration(declaration) =
                    self.semantic.nodes().parent_kind(node.id())
                else {
                    return BindingTarget::Opaque;
                };
                BindingTarget::Import {
                    module: String::from(declaration.source.value.as_str()),
                    exported: String::from("default"),
                    namespace: local.namespace,
                    type_only: declaration.import_kind.is_type(),
                    usage: None,
                }
            }
            AstKind::TSTypeAliasDeclaration(alias) if local.namespace == Namespace::Type => {
                if alias.type_parameters.is_none() {
                    self.annotation(&alias.type_annotation)
                } else {
                    BindingTarget::Opaque
                }
            }
            AstKind::VariableDeclarator(declaration) if local.namespace == Namespace::Value => {
                if let Some(initializer) = &declaration.init {
                    if declaration.kind == VariableDeclarationKind::Const
                        && !self
                            .semantic
                            .scoping()
                            .get_resolved_references(local.symbol)
                            .any(|reference| reference.is_write())
                    {
                        self.value_alias(initializer)
                    } else {
                        BindingTarget::Opaque
                    }
                } else if self.is_ambient(node.id()) {
                    declaration
                        .type_annotation
                        .as_ref()
                        .map_or(BindingTarget::Opaque, |annotation| {
                            self.annotation(&annotation.type_annotation)
                        })
                } else {
                    BindingTarget::Opaque
                }
            }
            _ => BindingTarget::Opaque,
        }
    }
}
