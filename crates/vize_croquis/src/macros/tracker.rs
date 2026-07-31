//! Focused [`MacroTracker`](super::MacroTracker) implementations.

use super::{MacroTracker, PropDefinition};

impl std::fmt::Debug for MacroTracker {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MacroTracker")
            .field("calls", &self.calls)
            .field("props", &self.props)
            .field("prop_declarations", &self.prop_declarations)
            .field("emits", &self.emits)
            .field("emit_calls", &self.emit_calls)
            .field("models", &self.models)
            .field("exposes", &self.exposes)
            .field("slots", &self.slots)
            .field("art", &self.art)
            .field("props_destructure", &self.props_destructure)
            .field("top_level_awaits", &self.top_level_awaits)
            .field("next_id", &self.next_id)
            .field("define_props_idx", &self.define_props_idx)
            .field("define_emits_idx", &self.define_emits_idx)
            .field("define_expose_idx", &self.define_expose_idx)
            .field("define_slots_idx", &self.define_slots_idx)
            .field("define_art_idx", &self.define_art_idx)
            .finish()
    }
}

impl MacroTracker {
    /// Add a prop definition.
    #[inline]
    pub fn add_prop(&mut self, prop: PropDefinition) {
        self.props.push(prop);
    }

    /// Add a prop together with the range of its written declaration.
    #[inline]
    pub fn add_prop_with_declaration(&mut self, prop: PropDefinition, start: u32, end: u32) {
        self.prop_declarations
            .insert(prop.name.clone(), (start, end));
        self.props.push(prop);
    }

    /// Get all props.
    #[inline]
    pub fn props(&self) -> &[PropDefinition] {
        &self.props
    }

    /// Get a prop's written declaration range, relative to its script block.
    #[inline]
    pub fn prop_declaration(&self, name: &str) -> Option<(u32, u32)> {
        self.prop_declarations.get(name).copied()
    }

    /// Shift every stored script-relative source offset by `delta`.
    pub fn shift_offsets(&mut self, delta: u32) {
        for call in &mut self.calls {
            call.start = call.start.saturating_add(delta);
            call.end = call.end.saturating_add(delta);
        }
        for range in self.prop_declarations.values_mut() {
            range.0 = range.0.saturating_add(delta);
            range.1 = range.1.saturating_add(delta);
        }
        for call in &mut self.emit_calls {
            call.start = call.start.saturating_add(delta);
            call.end = call.end.saturating_add(delta);
        }
        for await_expr in &mut self.top_level_awaits {
            await_expr.start = await_expr.start.saturating_add(delta);
            await_expr.end = await_expr.end.saturating_add(delta);
        }
    }
}
