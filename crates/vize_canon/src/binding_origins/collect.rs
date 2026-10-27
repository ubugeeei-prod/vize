//! Read only the caller's existing AST/semantic snapshot; no parser or I/O.
//! A borrowed declaration index avoids requiring Semantic's optional AST-node
//! store. Only compact owned facts survive this call.
use super::{
    BindingTarget, LocalSymbol, SourceFacts, SourceRevision,
    declarations::{DeclarationIndex, Syntax, SyntaxDeclarations},
};
use oxc_ast::ast::{Program, VariableDeclarationKind};
use oxc_semantic::Semantic;
use std::collections::VecDeque;
use vize_carton::FxHashMap;

pub(super) struct Collector<'s, 'a> {
    pub(super) semantic: &'s Semantic<'a>,
    pub(super) facts: SourceFacts,
    pub(super) declarations: DeclarationIndex<'s, 'a>,
    pub(super) pending: VecDeque<LocalSymbol>,
}

/// `global_script` is the host's effective module classification, including
/// moduleDetection and package context, not a filename/spelling heuristic.
pub fn collect<'s, 'a>(
    program: &'s Program<'a>,
    semantic: &'s Semantic<'a>,
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
        declarations: DeclarationIndex::default(),
        pending: requested.iter().copied().collect(),
    };
    for statement in &program.body {
        collector.statement(
            statement,
            global_script,
            program.source_type.is_typescript_definition(),
        );
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
impl<'s, 'a> Collector<'s, 'a> {
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
        match self.declarations.bindings.get(&local) {
            None => BindingTarget::Opaque,
            Some(SyntaxDeclarations::One(syntax)) => self.syntax_target(local, *syntax),
            Some(SyntaxDeclarations::Many(syntax)) => BindingTarget::Candidates(
                syntax
                    .iter()
                    .map(|syntax| self.syntax_target(*syntax))
                    .collect(),
            ),
        }
    }
    fn syntax_target(&self, local: LocalSymbol, syntax: Syntax<'s, 'a>) -> BindingTarget {
        match syntax {
            Syntax::Opaque => BindingTarget::Opaque,
            Syntax::Import {
                declaration,
                specifier,
            } => BindingTarget::Import {
                module: declaration.source.value.as_str().into(),
                exported: specifier.map_or_else(
                    || "default".into(),
                    |specifier| specifier.imported.name().as_str().into(),
                ),
                namespace: local.namespace,
                type_only: declaration.import_kind.is_type()
                    || specifier.is_some_and(|specifier| specifier.import_kind.is_type()),
                usage: None,
            },
            Syntax::TypeAlias(alias) if alias.type_parameters.is_none() => {
                self.annotation(&alias.type_annotation)
            }
            Syntax::TypeAlias(_) => BindingTarget::Opaque,
            Syntax::Variable {
                declaration,
                ambient,
            } => {
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
                } else if ambient {
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
        }
    }
}
