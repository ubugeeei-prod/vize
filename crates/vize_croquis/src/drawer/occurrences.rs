//! Demand-only capture during the existing identifier walk.

use crate::binding_occurrences::{BindingOccurrences, OccurrenceBlock};
use crate::scope::ScopeKind;
use vize_carton::{CompactString, FxHashMap};

use super::{Drawer, IdentifierRef};

pub(super) struct OccurrenceCapture {
    pub(super) references: FxHashMap<CompactString, Vec<IdentifierRef>>,
    packet: BindingOccurrences,
    pub(super) valid: bool,
}

impl Default for OccurrenceCapture {
    fn default() -> Self {
        Self {
            references: FxHashMap::default(),
            packet: BindingOccurrences::default(),
            valid: true,
        }
    }
}

impl Drawer {
    /// Retain authored references during the existing template expression walk.
    /// Croquis itself and its public dump/snapshot formats stay unchanged.
    #[doc(hidden)]
    pub fn with_binding_occurrences(mut self) -> Self {
        let capture = OccurrenceCapture {
            valid: !self.script_drawn,
            ..Default::default()
        };
        self.occurrence_capture = Some(capture);
        self
    }

    /// Continue an already-owned packet through the existing template walk.
    #[doc(hidden)]
    pub fn with_binding_occurrence_packet(mut self, mut packet: BindingOccurrences) -> Self {
        let valid = packet
            .resolve_script_globals(&self.croquis.binding_spans)
            .is_some();
        self.occurrence_capture = Some(OccurrenceCapture {
            packet,
            valid,
            ..Default::default()
        });
        self
    }

    /// Return the ordinary Croquis and the separately-owned editor fact packet.
    #[doc(hidden)]
    pub fn finish_with_binding_occurrences(self) -> (crate::Croquis, Option<BindingOccurrences>) {
        self.finish_occurrence_packet(true)
    }

    /// Preserve source-witnessed setup reads until the actual split-script join.
    #[doc(hidden)]
    pub fn finish_script_occurrences(self) -> (crate::Croquis, Option<BindingOccurrences>) {
        self.finish_occurrence_packet(false)
    }

    fn finish_occurrence_packet(
        self,
        resolve: bool,
    ) -> (crate::Croquis, Option<BindingOccurrences>) {
        let packet = self.occurrence_capture.and_then(|mut capture| {
            if !capture.valid {
                return None;
            }
            if resolve {
                capture
                    .packet
                    .resolve_script_globals(&self.croquis.binding_spans)?;
            }
            Some(capture.packet)
        });
        (self.croquis, packet)
    }

    pub(super) fn take_script_occurrences(
        &mut self,
        result: &mut crate::script_parser::ScriptParseResult,
        source: &str,
    ) {
        let Some(capture) = self.occurrence_capture.as_mut() else {
            return;
        };
        let Some(script) = result.occurrence_capture.take() else {
            capture.valid = false;
            return;
        };
        if script
            .finish(
                &result.scopes,
                &result.binding_spans,
                source,
                &mut capture.packet,
            )
            .is_none()
        {
            capture.valid = false;
        }
    }

    pub(super) fn note_expression_occurrences(
        &mut self,
        content: &str,
        base_offset: u32,
        scope_vars: &[CompactString],
    ) {
        let Some(capture) = self.occurrence_capture.as_mut() else {
            return;
        };
        let Some(references) = capture.references.get(content) else {
            return;
        };
        for reference in references {
            let name = reference.name.as_str();
            let binding = if let Some((scope, declaration)) = self.croquis.scopes.lookup(name)
                && matches!(
                    scope.kind,
                    ScopeKind::VFor
                        | ScopeKind::VSlot
                        | ScopeKind::EventHandler
                        | ScopeKind::Callback
                        | ScopeKind::Closure
                        | ScopeKind::Block
                ) {
                let start = declaration.declaration_offset;
                let Some(end) = start.checked_add(name.len() as u32) else {
                    capture.valid = false;
                    return;
                };
                if self.template_source.get(start as usize..end as usize) != Some(name) {
                    // Implicit handler parameters and transformed aliases have
                    // no authored declaration. They cannot become a setup read.
                    continue;
                }
                capture.packet.note_declaration(
                    scope.id,
                    name,
                    start,
                    end,
                    OccurrenceBlock::Template,
                )
            } else {
                if scope_vars.iter().any(|local| local.as_str() == name) {
                    continue;
                }
                let Some(&(start, end)) = self.croquis.binding_spans.get(name) else {
                    continue;
                };
                let Some(binding) = capture.packet.declaration(name, start, end) else {
                    capture.valid = false;
                    return;
                };
                binding
            };
            let Some(start) = base_offset.checked_add(reference.offset) else {
                capture.valid = false;
                return;
            };
            let Some(end) = start.checked_add(name.len() as u32) else {
                capture.valid = false;
                return;
            };
            capture
                .packet
                .note_reference(binding, start, end, OccurrenceBlock::Template);
        }
    }
}

#[cfg(test)]
mod tests;
