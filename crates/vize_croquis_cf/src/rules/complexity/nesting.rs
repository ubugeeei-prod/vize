use super::ComplexityInput;
use crate::graph::{DependencyEdge, DependencyGraph};
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::{FxHashMap, FxHashSet};
use vize_croquis::croquis::ComponentUsage;
use vize_croquis::{Croquis, ScopeId, ScopeKind, TemplateExpressionKind};

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct TemplateNesting {
    v_if: usize,
    v_for: usize,
    scoped_slot: usize,
    structures: usize,
    cognitive: usize,
}

impl TemplateNesting {
    fn add(self, other: Self) -> Self {
        let parent_depth = self
            .v_if
            .saturating_add(self.v_for)
            .saturating_add(self.scoped_slot);

        Self {
            v_if: self.v_if.saturating_add(other.v_if),
            v_for: self.v_for.saturating_add(other.v_for),
            scoped_slot: self.scoped_slot.saturating_add(other.scoped_slot),
            structures: self.structures.saturating_add(other.structures),
            cognitive: self
                .cognitive
                .saturating_add(other.cognitive)
                .saturating_add(other.structures.saturating_mul(parent_depth)),
        }
    }

    fn max_assign(&mut self, other: Self) {
        self.v_if = self.v_if.max(other.v_if);
        self.v_for = self.v_for.max(other.v_for);
        self.scoped_slot = self.scoped_slot.max(other.scoped_slot);
        self.cognitive = self.cognitive.max(other.cognitive);
    }
}

struct ComponentProfile {
    local: TemplateNesting,
    usages: Vec<UsageTemplateNesting>,
}

struct UsageTemplateNesting {
    child: FileId,
    nesting: TemplateNesting,
}

pub(super) fn add_component_tree_template_nesting(
    input: &mut ComplexityInput,
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
) {
    let profiles = component_profiles(registry, graph);
    let mut cache = FxHashMap::default();
    let mut best = TemplateNesting::default();

    for entry in registry.vue_components() {
        let mut visiting = FxHashSet::default();
        best.max_assign(component_tree_nesting(
            entry.id,
            &profiles,
            &mut visiting,
            &mut cache,
        ));
    }

    input.component_tree_v_if_max_depth = best.v_if;
    input.component_tree_v_for_max_depth = best.v_for;
    input.component_tree_scoped_slot_max_depth = best.scoped_slot;
    input.component_tree_template_nesting_score = best.cognitive;
}

fn component_profiles(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
) -> FxHashMap<FileId, ComponentProfile> {
    let mut profiles = FxHashMap::default();

    for entry in registry.vue_components() {
        let mut profile = ComponentProfile {
            local: local_template_nesting(&entry.analysis),
            usages: Vec::new(),
        };

        for usage in &entry.analysis.component_usages {
            let Some(child) = graph.find_by_component(usage.name.as_str()) else {
                continue;
            };
            if !has_component_usage_edge(graph, entry.id, child) {
                continue;
            }

            profile.usages.push(UsageTemplateNesting {
                child,
                nesting: component_usage_nesting(&entry.analysis, usage),
            });
        }

        profiles.insert(entry.id, profile);
    }

    profiles
}

fn component_tree_nesting(
    id: FileId,
    profiles: &FxHashMap<FileId, ComponentProfile>,
    visiting: &mut FxHashSet<FileId>,
    cache: &mut FxHashMap<FileId, TemplateNesting>,
) -> TemplateNesting {
    if let Some(cached) = cache.get(&id) {
        return *cached;
    }

    let Some(profile) = profiles.get(&id) else {
        return TemplateNesting::default();
    };

    if !visiting.insert(id) {
        return TemplateNesting::default();
    }

    let mut best = profile.local;
    for usage in &profile.usages {
        let child = component_tree_nesting(usage.child, profiles, visiting, cache);
        best.max_assign(usage.nesting.add(child));
    }

    visiting.remove(&id);
    cache.insert(id, best);
    best
}

fn local_template_nesting(analysis: &Croquis) -> TemplateNesting {
    let mut nesting = TemplateNesting::default();

    for expr in analysis
        .template_expressions
        .iter()
        .filter(|expr| expr.kind == TemplateExpressionKind::VIf)
    {
        let guard_depth = v_if_guard_depth(analysis, expr.vif_guard.as_deref());
        let scope_depth = scope_template_depth(analysis, expr.scope_id);
        let depth = guard_depth.saturating_add(1);
        nesting.v_if = nesting.v_if.max(depth);
        nesting.structures = nesting.structures.saturating_add(1);
        nesting.cognitive = nesting
            .cognitive
            .saturating_add(depth)
            .saturating_add(scope_depth);
    }

    for scope in analysis.scopes.iter() {
        match scope.kind {
            ScopeKind::VFor => {
                let depth = scope_kind_depth(analysis, scope.id, ScopeKind::VFor);
                nesting.v_for = nesting.v_for.max(depth);
                nesting.structures = nesting.structures.saturating_add(1);
                nesting.cognitive = nesting.cognitive.saturating_add(depth);
            }
            ScopeKind::VSlot => {
                let depth = scope_kind_depth(analysis, scope.id, ScopeKind::VSlot);
                nesting.scoped_slot = nesting.scoped_slot.max(depth);
                nesting.structures = nesting.structures.saturating_add(1);
                nesting.cognitive = nesting.cognitive.saturating_add(depth);
            }
            _ => {}
        }
    }

    for usage in &analysis.component_usages {
        nesting.v_if = nesting
            .v_if
            .max(v_if_guard_depth(analysis, usage.vif_guard.as_deref()));
        if usage.slots.iter().any(|slot| slot.has_scope) {
            let scoped_depth =
                scope_kind_depth(analysis, usage.scope_id, ScopeKind::VSlot).saturating_add(1);
            nesting.scoped_slot = nesting.scoped_slot.max(scoped_depth);
        }
    }

    nesting
}

fn component_usage_nesting(analysis: &Croquis, usage: &ComponentUsage) -> TemplateNesting {
    let v_if_depth = v_if_guard_depth(analysis, usage.vif_guard.as_deref());
    let v_for_depth = scope_kind_depth(analysis, usage.scope_id, ScopeKind::VFor);
    let scoped_slot_depth = scope_kind_depth(analysis, usage.scope_id, ScopeKind::VSlot);
    let own_scoped_slot_depth = usize::from(usage.slots.iter().any(|slot| slot.has_scope));
    let scoped_slot = scoped_slot_depth.saturating_add(own_scoped_slot_depth);
    let mut cognitive = 0usize;
    let mut structures = 0usize;
    let usage_depth = v_if_depth
        .saturating_add(v_for_depth)
        .saturating_add(scoped_slot_depth);

    if v_if_depth > 0 {
        structures = structures.saturating_add(1);
        cognitive = cognitive.saturating_add(usage_depth.max(1));
    }
    if own_scoped_slot_depth > 0 {
        structures = structures.saturating_add(1);
        cognitive = cognitive.saturating_add(usage_depth.saturating_add(1));
    }

    TemplateNesting {
        v_if: v_if_depth,
        v_for: v_for_depth,
        scoped_slot,
        structures,
        cognitive,
    }
}

fn scope_kind_depth(analysis: &Croquis, start: ScopeId, kind: ScopeKind) -> usize {
    let mut depth = 0usize;
    let mut current = Some(start);
    let mut seen = FxHashSet::default();

    while let Some(id) = current {
        if !seen.insert(id) {
            break;
        }

        let Some(scope) = analysis.scopes.get_scope(id) else {
            break;
        };
        if scope.kind == kind {
            depth = depth.saturating_add(1);
        }
        current = scope.parent();
    }

    depth
}

fn scope_template_depth(analysis: &Croquis, start: ScopeId) -> usize {
    scope_kind_depth(analysis, start, ScopeKind::VFor).saturating_add(scope_kind_depth(
        analysis,
        start,
        ScopeKind::VSlot,
    ))
}

fn v_if_guard_depth(analysis: &Croquis, guard: Option<&str>) -> usize {
    let Some(guard) = guard else {
        return 0;
    };

    analysis
        .template_expressions
        .iter()
        .filter(|expr| expr.kind == TemplateExpressionKind::VIf)
        .filter(|expr| !expr.content.is_empty() && guard.contains(expr.content.as_str()))
        .count()
        .max(1)
}

fn has_component_usage_edge(graph: &DependencyGraph, parent: FileId, child: FileId) -> bool {
    graph
        .dependencies(parent)
        .any(|(id, edge)| id == child && edge == DependencyEdge::ComponentUsage)
}
