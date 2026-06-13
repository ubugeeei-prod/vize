//! Event bubbling analysis.
//!
//! Tracks event propagation through the component tree:
//! - Events emitted but not handled by any ancestor
//! - Event modifier issues (.stop, .prevent)

use crate::diagnostics::{CrossFileDiagnostic, CrossFileDiagnosticKind, DiagnosticSeverity};
use crate::graph::{DependencyEdge, DependencyGraph};
use crate::registry::{FileId, ModuleRegistry};
use vize_carton::{CompactString, FxHashMap, FxHashSet, cstr};

/// Information about event bubbling.
#[derive(Debug, Clone)]
pub struct EventBubble {
    /// Component that emits the event.
    pub source: FileId,
    /// Event name.
    pub event_name: CompactString,
    /// Chain of components the event travels through.
    pub propagation_path: Vec<FileId>,
    /// Final handler (if any).
    pub handler: Option<FileId>,
    /// Whether the event is stopped.
    pub is_stopped: bool,
    /// Whether the event is prevented.
    pub is_prevented: bool,
    /// Depth in the component tree.
    pub depth: usize,
}

/// Analyze event bubbling across the component tree.
pub fn analyze_event_bubbling(
    registry: &ModuleRegistry,
    graph: &DependencyGraph,
) -> (Vec<EventBubble>, Vec<CrossFileDiagnostic>) {
    let mut bubbles = Vec::new();
    let mut diagnostics = Vec::new();

    // Collect all emitted events with their source components
    let mut emitted_events: FxHashMap<FileId, Vec<(CompactString, u32)>> = FxHashMap::default();

    for entry in registry.vue_components() {
        for emit in entry.analysis.macros.emits() {
            emitted_events
                .entry(entry.id)
                .or_default()
                .push((emit.name.clone(), 0)); // Offset not tracked in EmitDefinition
        }
    }

    // Collect event handlers from all components
    let mut event_handlers: FxHashMap<FileId, FxHashSet<CompactString>> = FxHashMap::default();
    let mut event_modifiers: FxHashMap<FileId, FxHashMap<CompactString, Vec<CompactString>>> =
        FxHashMap::default();

    for entry in registry.vue_components() {
        let (handlers, modifiers) = extract_event_handlers(&entry.analysis);
        event_handlers.insert(entry.id, handlers);
        event_modifiers.insert(entry.id, modifiers);
    }

    // Trace event propagation for each emitted event
    for (&source_id, events) in &emitted_events {
        for (event_name, offset) in events {
            let traced_bubbles =
                trace_event_propagation(source_id, event_name, graph, &event_handlers);

            for bubble in traced_bubbles {
                let handled = bubble.handler.is_some();

                // Check for unhandled events (depth > 2 means it's propagating without being caught)
                if !handled && bubble.depth > 2 {
                    diagnostics.push(
                        CrossFileDiagnostic::new(
                            CrossFileDiagnosticKind::UnhandledEvent {
                                event_name: event_name.clone(),
                                depth: bubble.depth,
                            },
                            DiagnosticSeverity::Info,
                            source_id,
                            *offset,
                            cstr!(
                                "Event '{}' propagates {} levels without being handled",
                                event_name,
                                bubble.depth
                            ),
                        )
                        .with_suggestion(
                            "Add an event handler or consider if this event is needed",
                        ),
                    );
                }

                // Check for event modifier issues
                for file_id in &bubble.propagation_path {
                    if let Some(modifiers) = event_modifiers.get(file_id)
                        && let Some(mods) = modifiers.get(event_name)
                    {
                        for modifier in mods {
                            if modifier == "stop" || modifier == "prevent" {
                                diagnostics.push(
                                    CrossFileDiagnostic::new(
                                        CrossFileDiagnosticKind::EventModifierIssue {
                                            event_name: event_name.clone(),
                                            modifier: modifier.clone(),
                                        },
                                        DiagnosticSeverity::Info,
                                        *file_id,
                                        0,
                                        cstr!(
                                            "Event '{}' has .{} modifier which may prevent handling",
                                            event_name,
                                            modifier
                                        ),
                                    )
                                    .with_related(source_id, *offset, "Event is emitted here"),
                                );
                            }
                        }
                    }
                }

                bubbles.push(bubble);
            }
        }
    }

    (bubbles, diagnostics)
}

/// Trace event propagation from source through ancestors.
fn trace_event_propagation(
    source: FileId,
    event_name: &str,
    graph: &DependencyGraph,
    event_handlers: &FxHashMap<FileId, FxHashSet<CompactString>>,
) -> Vec<EventBubble> {
    let mut bubbles = Vec::new();
    let mut path = vec![source];
    let mut visited = FxHashSet::default();
    visited.insert(source);

    trace_event_propagation_paths(
        source,
        source,
        event_name,
        graph,
        event_handlers,
        &mut path,
        &mut visited,
        0,
        &mut bubbles,
    );

    bubbles
}

#[allow(clippy::too_many_arguments)]
fn trace_event_propagation_paths(
    source: FileId,
    current: FileId,
    event_name: &str,
    graph: &DependencyGraph,
    event_handlers: &FxHashMap<FileId, FxHashSet<CompactString>>,
    path: &mut Vec<FileId>,
    visited: &mut FxHashSet<FileId>,
    depth: usize,
    bubbles: &mut Vec<EventBubble>,
) {
    const MAX_DEPTH: usize = 50; // Prevent infinite loops

    if depth >= MAX_DEPTH {
        bubbles.push(make_event_bubble(source, event_name, path, None, depth));
        return;
    }

    let next_depth = depth + 1;
    let mut parents: Vec<_> = graph
        .dependents(current)
        .filter(|(_, edge)| *edge == DependencyEdge::ComponentUsage)
        .map(|(id, _)| id)
        .collect();
    parents.sort_by_key(|id| id.as_u32());
    parents.dedup();

    if parents.is_empty() {
        bubbles.push(make_event_bubble(
            source, event_name, path, None, next_depth,
        ));
        return;
    }

    let mut explored_parent = false;

    for parent in parents {
        if visited.contains(&parent) {
            continue;
        }

        explored_parent = true;
        path.push(parent);
        visited.insert(parent);

        if let Some(handlers) = event_handlers.get(&parent)
            && handlers.contains(event_name)
        {
            bubbles.push(make_event_bubble(
                source,
                event_name,
                path,
                Some(parent),
                next_depth,
            ));
        } else {
            trace_event_propagation_paths(
                source,
                parent,
                event_name,
                graph,
                event_handlers,
                path,
                visited,
                next_depth,
                bubbles,
            );
        }

        visited.remove(&parent);
        path.pop();
    }

    if !explored_parent {
        bubbles.push(make_event_bubble(
            source, event_name, path, None, next_depth,
        ));
    }
}

fn make_event_bubble(
    source: FileId,
    event_name: &str,
    path: &[FileId],
    handler: Option<FileId>,
    depth: usize,
) -> EventBubble {
    EventBubble {
        source,
        event_name: CompactString::new(event_name),
        propagation_path: path.to_vec(),
        handler,
        is_stopped: false,
        is_prevented: false,
        depth,
    }
}

/// Extract event handlers and their modifiers from a component.
fn extract_event_handlers(
    analysis: &vize_croquis::Croquis,
) -> (
    FxHashSet<CompactString>,
    FxHashMap<CompactString, Vec<CompactString>>,
) {
    let mut handlers = FxHashSet::default();
    let mut modifiers: FxHashMap<CompactString, Vec<CompactString>> = FxHashMap::default();

    // Look for event handler scopes
    for scope in analysis.scopes.iter() {
        if scope.kind == vize_croquis::ScopeKind::EventHandler
            && let vize_croquis::ScopeData::EventHandler(data) = scope.data()
        {
            handlers.insert(data.event_name.clone());

            // Parse modifiers from handler expression if present
            if let Some(ref expr) = data.handler_expression {
                let mods = extract_modifiers(expr);
                if !mods.is_empty() {
                    modifiers.insert(data.event_name.clone(), mods);
                }
            }
        }
    }

    (handlers, modifiers)
}

/// Extract modifiers from an event handler expression.
fn extract_modifiers(expr: &str) -> Vec<CompactString> {
    let mut modifiers = Vec::new();

    // Look for common modifiers
    if expr.contains(".stop") {
        modifiers.push(CompactString::new("stop"));
    }
    if expr.contains(".prevent") {
        modifiers.push(CompactString::new("prevent"));
    }
    if expr.contains(".capture") {
        modifiers.push(CompactString::new("capture"));
    }
    if expr.contains(".once") {
        modifiers.push(CompactString::new("once"));
    }
    if expr.contains(".passive") {
        modifiers.push(CompactString::new("passive"));
    }

    modifiers
}

#[cfg(test)]
mod tests;
