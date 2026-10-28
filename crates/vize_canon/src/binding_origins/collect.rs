//! Read only the caller's existing AST/semantic snapshot; no parser or I/O.
//! A borrowed declaration index avoids requiring Semantic's optional AST-node
//! store. Only compact owned facts survive this call.
use super::{
    BindingTarget, LocalSymbol, OriginError, SourceFacts, SourceRevision,
    declarations::{DeclarationIndex, Syntax, SyntaxDeclarations},
};
use oxc_ast::ast::{Program, VariableDeclarationKind};
use oxc_semantic::Semantic;
use std::collections::VecDeque;
use vize_carton::{FxHashMap, FxHashSet, String};

pub(super) struct Collector<'s, 'a> {
    pub(super) semantic: &'s Semantic<'a>,
    pub(super) facts: SourceFacts,
    pub(super) declarations: DeclarationIndex<'s, 'a>,
    pub(super) pending: VecDeque<LocalSymbol>,
    pub(super) global_names: Option<FxHashSet<String>>,
    pub(super) global_symbols: FxHashSet<LocalSymbol>,
    pub(super) missing_semantic_scope: bool,
}

/// `global_script` is the host's effective module classification, including
/// moduleDetection and package context, not a filename/spelling heuristic.
pub fn collect<'s, 'a>(
    program: &'s Program<'a>,
    semantic: &'s Semantic<'a>,
    revision: SourceRevision,
    global_script: bool,
    requested: &[LocalSymbol],
) -> Result<SourceFacts, OriginError> {
    collect_impl(program, semantic, revision, global_script, requested, None)
}

/// Lower only the named global roots plus their actual local dependencies.
/// Scope maps come from the caller's existing semantic pass. This does not
/// establish whole-project parse, resolution or mutation completeness.
pub fn collect_requested_globals<'s, 'a>(
    program: &'s Program<'a>,
    semantic: &'s Semantic<'a>,
    revision: SourceRevision,
    global_script: bool,
    requested: &[LocalSymbol],
    global_names: &[&str],
) -> Result<SourceFacts, OriginError> {
    collect_impl(
        program,
        semantic,
        revision,
        global_script,
        requested,
        Some(
            global_names
                .iter()
                .map(|name| String::from(*name))
                .collect(),
        ),
    )
}
fn collect_impl<'s, 'a>(
    program: &'s Program<'a>,
    semantic: &'s Semantic<'a>,
    revision: SourceRevision,
    global_script: bool,
    requested: &[LocalSymbol],
    global_names: Option<FxHashSet<String>>,
) -> Result<SourceFacts, OriginError> {
    if program.scope_id.get() != Some(semantic.scoping().root_scope_id()) {
        return Err(OriginError::MissingSemanticScope);
    }
    let mut collector = Collector {
        semantic,
        facts: SourceFacts {
            revision,
            bindings: FxHashMap::default(),
            globals: Vec::new(),
        },
        declarations: DeclarationIndex::default(),
        pending: requested.iter().copied().collect(),
        global_names,
        global_symbols: FxHashSet::default(),
        missing_semantic_scope: false,
    };
    for statement in &program.body {
        collector.statement(
            statement,
            global_script,
            program.source_type.is_typescript_definition(),
        );
    }
    if global_script {
        collector.global_scope(semantic.scoping().root_scope_id());
    }
    if collector.missing_semantic_scope {
        return Err(OriginError::MissingSemanticScope);
    }
    while let Some(local) = collector.pending.pop_front() {
        if collector.facts.bindings.contains_key(&local) {
            continue;
        }
        let target = collector.symbol_target(local);
        collector.retain_aliases(&target);
        collector.facts.bindings.insert(local, target);
    }
    Ok(collector.facts)
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
        let declarations = self.declarations.bindings.get(&local);
        if self.variable_declarations_are_untrusted(local, declarations) {
            return BindingTarget::Opaque;
        }
        match declarations {
            None => BindingTarget::Opaque,
            Some(SyntaxDeclarations::One(syntax)) => self.syntax_target(local, *syntax),
            Some(SyntaxDeclarations::Many(syntax)) => BindingTarget::Candidates(
                syntax
                    .iter()
                    .map(|syntax| self.syntax_target(local, *syntax))
                    .collect(),
            ),
        }
    }
    fn variable_declarations_are_untrusted(
        &self,
        local: LocalSymbol,
        declarations: Option<&SyntaxDeclarations<'s, 'a>>,
    ) -> bool {
        let Some(declarations) = declarations else {
            return false;
        };
        let one_node = |syntax: &Syntax<'s, 'a>| match syntax {
            Syntax::Variable { declaration, .. } => Some(declaration.node_id.get()),
            _ => None,
        };
        let covered = match declarations {
            SyntaxDeclarations::One(syntax) => {
                let Some(node) = one_node(syntax) else {
                    return false;
                };
                self.semantic
                    .scoping()
                    .symbol_declarations(local.symbol)
                    .all(|declared| declared == node)
            }
            SyntaxDeclarations::Many(syntax) => {
                let nodes = syntax.iter().filter_map(one_node).collect::<FxHashSet<_>>();
                if nodes.is_empty() {
                    return false;
                }
                self.semantic
                    .scoping()
                    .symbol_declarations(local.symbol)
                    .all(|node| nodes.contains(&node))
            }
        };
        // A for-var/block-var redeclaration is a write even when no identifier
        // reference represents the binding. Compare declaration IDs once.
        !covered
            || self
                .semantic
                .scoping()
                .get_resolved_references(local.symbol)
                .any(|reference| reference.is_write())
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
                    if declaration.kind == VariableDeclarationKind::Const {
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
