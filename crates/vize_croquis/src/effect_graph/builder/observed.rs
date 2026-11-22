//! Build the project effect graph while a second reader receives the same declarations.
use super::{
    EffectBuildContext, collect_program_effect_edges, collect_reactive_sources,
    collect_vue_api_aliases, scoped_node_name,
};
use crate::EffectGraph;
use vize_carton::{CompactString, FxHashSet};
use vize_l1::container::vue::ScriptRole;
use vize_l2::query::script::{
    DeclarationExport, DeclarationSyntaxError, ScriptDeclaration, ScriptDeclarationKind,
    ScriptDeclarations,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservedEffectError {
    Source,
    Grammar,
    Syntax(DeclarationSyntaxError),
    OverlappingBlocks,
}

/// A declaration reader using the effect graph's original source and language.
pub struct ObservedEffectScript<'p, 'a>(ScriptDeclarations<'p, 'a>);

impl<'p, 'a> ObservedEffectScript<'p, 'a> {
    pub(super) fn new(declarations: ScriptDeclarations<'p, 'a>) -> Self {
        Self(declarations)
    }
}

/// Visit both scripts in the graph's scope order. An ordinary script's Vue
/// aliases and dependencies can be inherited or shadowed by setup.
pub fn visit_sfc_effect_graph(
    ordinary: Option<&ObservedEffectScript<'_, '_>>,
    setup: Option<&ObservedEffectScript<'_, '_>>,
    mut visit: impl FnMut(ScriptRole, ScriptDeclaration<'_>),
) -> Result<EffectGraph, ObservedEffectError> {
    if let (Some(script), Some(setup)) = (ordinary, setup) {
        let a = script.0.block();
        let b = setup.0.block();
        if !core::ptr::eq(a.root_source(), b.root_source()) {
            return Err(ObservedEffectError::Source);
        }
        if a.start() < b.end() && b.start() < a.end() {
            return Err(ObservedEffectError::OverlappingBlocks);
        }
    }
    let mut graph = EffectGraph::default();
    let mut context = EffectBuildContext::default();
    if let Some(script) = ordinary {
        read(
            script,
            ScriptRole::Ordinary,
            &mut context,
            &mut graph,
            &mut visit,
        );
    }
    if let Some(script) = setup {
        read(
            script,
            ScriptRole::Setup,
            &mut context,
            &mut graph,
            &mut visit,
        );
    }
    Ok(graph)
}

fn read(
    script: &ObservedEffectScript<'_, '_>,
    role: ScriptRole,
    context: &mut EffectBuildContext,
    graph: &mut EffectGraph,
    visit: &mut impl FnMut(ScriptRole, ScriptDeclaration<'_>),
) {
    let prefix = match role {
        ScriptRole::Ordinary => "script",
        ScriptRole::Setup => "setup",
    };
    let program = &script.0.original().parsed().program;
    context.prefix = CompactString::new(prefix);
    script.0.visit(|declaration| {
        visit(role, declaration);
        if declaration.export != DeclarationExport::Default
            && matches!(
                declaration.kind,
                ScriptDeclarationKind::Import { type_only: false }
                    | ScriptDeclarationKind::Variable(_)
                    | ScriptDeclarationKind::Function
                    | ScriptDeclarationKind::Class
            )
        {
            let name = declaration.name;
            context.api_aliases.remove(name);
            context.blocked_api_names.insert(CompactString::new(name));
            context.reactive_sources.remove(name);
            context.reactive_node_ids.remove(name);
        }
    });
    for statement in &program.body {
        collect_vue_api_aliases(statement, &mut context.api_aliases);
    }
    for api in context.api_aliases.keys() {
        context.blocked_api_names.remove(api);
    }
    let mut sources = FxHashSet::default();
    for statement in &program.body {
        collect_reactive_sources(
            statement,
            &context.api_aliases,
            &context.blocked_api_names,
            &mut sources,
        );
    }
    for source in sources {
        let node = scoped_node_name(prefix, &source);
        context.reactive_sources.insert(source.clone());
        context.reactive_node_ids.insert(source, node);
    }
    collect_program_effect_edges(program, context, graph);
}
