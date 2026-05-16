//! Formatting and building utilities for VIR output.
//!
//! Contains `SummaryBuilder` for constructing `Croquis` instances from
//! tracker data, and the `Croquis::to_vir()` method for TOML-like output.

use crate::css::CssTracker;
use crate::hoist::HoistTracker;
use crate::macros::MacroTracker;
use crate::optimization::OptimizationTracker;

use super::{
    BlockDisplay, Croquis, CssDisplay, Diagnostic, EmitDisplay, EventCacheDisplay, HoistDisplay,
    MacroDisplay, MemoCacheDisplay, OnceCacheDisplay, PropDisplay, SelectorDisplay, Severity,
    TopLevelAwaitDisplay,
};
use vize_carton::append;
use vize_carton::FxHashMap;
use vize_carton::String;
use vize_carton::ToCompactString;

/// Builder for Croquis
#[derive(Debug, Default)]
pub struct SummaryBuilder {
    summary: Croquis,
}

impl SummaryBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add macro tracker data
    pub fn with_macros(mut self, tracker: &MacroTracker) -> Self {
        // Add macro calls
        for call in tracker.all_calls() {
            self.summary.macros.push(MacroDisplay {
                name: call.name.to_compact_string(),
                #[allow(clippy::disallowed_macros)]
                kind: format!("{:?}", call.kind).into(),
                start: call.start,
                end: call.end,
            });
        }

        // Add props
        for prop in tracker.props() {
            self.summary.props.push(PropDisplay {
                name: prop.name.to_compact_string(),
                prop_type: prop.prop_type.as_ref().map(|s| s.to_compact_string()),
                required: prop.required,
                has_default: prop.default_value.is_some(),
            });
        }

        // Add emits
        for emit in tracker.emits() {
            self.summary.emits.push(EmitDisplay {
                name: emit.name.to_compact_string(),
                payload_type: emit.payload_type.as_ref().map(|s| s.to_compact_string()),
            });
        }

        // Add top-level awaits
        self.summary.is_async = tracker.is_async();
        for await_expr in tracker.top_level_awaits() {
            self.summary.top_level_awaits.push(TopLevelAwaitDisplay {
                expression: await_expr.expression.to_compact_string(),
                start: await_expr.start,
                end: await_expr.end,
            });
        }

        self.summary.stats.prop_count = tracker.props().len() as u32;
        self.summary.stats.emit_count = tracker.emits().len() as u32;
        self.summary.stats.model_count = tracker.models().len() as u32;

        self
    }

    /// Add optimization tracker data
    pub fn with_optimization(mut self, tracker: &OptimizationTracker) -> Self {
        // Add blocks
        for block in tracker.blocks() {
            self.summary.optimization.blocks.push(BlockDisplay {
                id: block.id,
                #[allow(clippy::disallowed_macros)]
                block_type: format!("{:?}", block.block_type).into(),
                parent_id: block.parent_id,
                dynamic_children: block.dynamic_children_count,
            });
        }

        // Add event cache
        for event in tracker.event_cache() {
            self.summary
                .optimization
                .event_cache
                .push(EventCacheDisplay {
                    cache_index: event.cache_index,
                    event_name: event.event_name.to_compact_string(),
                    handler: event.handler.to_compact_string(),
                    is_inline: event.is_inline,
                });
        }

        // Add once cache
        for once in tracker.once_cache() {
            self.summary.optimization.once_cache.push(OnceCacheDisplay {
                cache_index: once.cache_index,
                content: once.content.to_compact_string(),
                start: once.start,
                end: once.end,
            });
        }

        // Add memo cache
        for memo in tracker.memo_cache() {
            self.summary.optimization.memo_cache.push(MemoCacheDisplay {
                cache_index: memo.cache_index,
                deps: memo.deps.to_compact_string(),
                content: memo.content.to_compact_string(),
                start: memo.start,
                end: memo.end,
            });
        }

        self.summary.stats.cache_count = tracker.current_cache_index();

        self
    }

    /// Add hoist tracker data
    pub fn with_hoists(mut self, tracker: &HoistTracker) -> Self {
        for hoist in tracker.hoists() {
            self.summary.hoists.push(HoistDisplay {
                id: hoist.id.as_u32(),
                #[allow(clippy::disallowed_macros)]
                level: format!("{:?}", hoist.level).into(),
                content: hoist.content.to_compact_string(),
            });
        }

        self.summary.stats.hoist_count = tracker.count() as u32;

        self
    }

    /// Add CSS tracker data
    pub fn with_css(mut self, tracker: &CssTracker) -> Self {
        let stats = tracker.stats();

        self.summary.css = Some(CssDisplay {
            selectors: tracker
                .selectors()
                .iter()
                .map(|s| SelectorDisplay {
                    raw: s.raw.to_compact_string(),
                    scoped: true, // Assume scoped by default
                })
                .collect(),
            v_bind_count: stats.v_bind_count,
            has_deep: stats.deep_selectors > 0,
            has_slotted: stats.slotted_selectors > 0,
            has_global: stats.global_selectors > 0,
        });

        self
    }

    /// Add diagnostic
    pub fn add_diagnostic(mut self, diagnostic: Diagnostic) -> Self {
        self.summary.diagnostics.push(diagnostic);
        self
    }

    /// Build the summary
    pub fn build(self) -> Croquis {
        self.summary
    }
}

impl Croquis {
    /// Convert to VIR (TOML-like) format
    pub fn to_vir(&self) -> String {
        let mut output = String::with_capacity(4096);

        output.push_str("[analysis]\n");
        append!(output, "is_async = {}\n", self.is_async);
        append!(output, "scope_count = {}\n", self.stats.scope_count);
        append!(output, "binding_count = {}\n", self.stats.binding_count);
        output.push('\n');

        // Scopes
        if !self.scopes.is_empty() {
            output.push_str("[scopes]\n");

            // Assign display IDs per prefix type (separate counters for #, ~, !)
            let mut prefix_counters: FxHashMap<&str, u32> = FxHashMap::default();
            prefix_counters.insert("#", 0);
            prefix_counters.insert("~", 0);
            prefix_counters.insert("!", 0);

            // Map internal id -> (prefix, display_id)
            let mut display_id_map: FxHashMap<u32, (String, u32)> = FxHashMap::default();
            for scope in &self.scopes {
                let prefix = scope.kind.prefix();
                let counter = prefix_counters.entry(prefix).or_insert(0);
                let display_id = *counter;
                *counter += 1;
                display_id_map.insert(scope.id, (prefix.to_compact_string(), display_id));
            }

            for scope in &self.scopes {
                let (prefix, display_id) = display_id_map.get(&scope.id).unwrap();
                // Format: prefix+display_id kind @start:end {bindings} $ parent_refs
                append!(
                    output,
                    "{prefix}{display_id} {} @{}:{}",
                    scope.kind.to_display(),
                    scope.start,
                    scope.end
                );

                // Bindings
                if !scope.bindings.is_empty() {
                    output.push_str(" {");
                    let mut first = true;
                    for (name, _) in &scope.bindings {
                        if !first {
                            output.push_str(", ");
                        }
                        output.push_str(name);
                        first = false;
                    }
                    output.push('}');
                }

                // Parent references (at the end)
                if !scope.parent_ids.is_empty() {
                    output.push_str(" $ ");
                    let mut first = true;
                    for parent_id in &scope.parent_ids {
                        if !first {
                            output.push_str(", ");
                        }
                        if let Some((p_prefix, p_display_id)) = display_id_map.get(parent_id) {
                            append!(output, "{p_prefix}{p_display_id}");
                        } else {
                            append!(output, "#{parent_id}");
                        }
                        first = false;
                    }
                }
                output.push('\n');
            }
            output.push('\n');
        }

        // Props
        if !self.props.is_empty() {
            output.push_str("[props]\n");
            for prop in &self.props {
                append!(
                    output,
                    "  {} = {{ type = {:?}, required = {} }}\n",
                    prop.name,
                    prop.prop_type.as_deref().unwrap_or("any"),
                    prop.required
                );
            }
            output.push('\n');
        }

        // Emits
        if !self.emits.is_empty() {
            output.push_str("[emits]\n");
            for emit in &self.emits {
                append!(output, "  {} = {:?}\n", emit.name, emit.payload_type);
            }
            output.push('\n');
        }

        // Top-level awaits
        if !self.top_level_awaits.is_empty() {
            output.push_str("[top_level_await]\n");
            for await_expr in &self.top_level_awaits {
                append!(
                    output,
                    "  {{ expression = \"{}\", span = [{}, {}] }}\n",
                    await_expr.expression,
                    await_expr.start,
                    await_expr.end
                );
            }
            output.push('\n');
        }

        // Event cache
        if !self.optimization.event_cache.is_empty() {
            output.push_str("[event_cache]\n");
            for event in &self.optimization.event_cache {
                append!(
                    output,
                    "  _cache[{}] = {{ event = \"{}\", handler = \"{}\" }}\n",
                    event.cache_index,
                    event.event_name,
                    event.handler
                );
            }
            output.push('\n');
        }

        // Blocks
        if !self.optimization.blocks.is_empty() {
            output.push_str("[blocks]\n");
            for block in &self.optimization.blocks {
                append!(
                    output,
                    "  block_{} = {{ type = \"{}\", dynamic_children = {} }}\n",
                    block.id,
                    block.block_type,
                    block.dynamic_children
                );
            }
            output.push('\n');
        }

        // Diagnostics
        if !self.diagnostics.is_empty() {
            output.push_str("[diagnostics]\n");
            for diag in &self.diagnostics {
                let severity = match diag.severity {
                    Severity::Error => "error",
                    Severity::Warning => "warning",
                    Severity::Info => "info",
                    Severity::Hint => "hint",
                };
                append!(
                    output,
                    "  {{ severity = \"{severity}\", message = \"{}\" }}\n",
                    diag.message
                );
            }
        }

        output
    }
}
