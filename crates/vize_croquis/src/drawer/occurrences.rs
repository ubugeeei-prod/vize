//! Demand-only capture during the existing identifier walk.

use crate::binding_occurrences::{BindingOccurrences, OccurrenceBlock};
use crate::scope::{ScopeData, ScopeKind};
use vize_carton::{CompactString, FxHashMap};

use super::{Drawer, IdentifierRef};

pub(super) struct OccurrenceCapture {
    pub(super) references: FxHashMap<CompactString, (Vec<IdentifierRef>, bool)>,
    packet: BindingOccurrences,
    runtime_reads: Option<BindingOccurrences>,
    pub(super) valid: bool,
}

impl Default for OccurrenceCapture {
    fn default() -> Self {
        Self {
            references: FxHashMap::default(),
            packet: BindingOccurrences::default(),
            runtime_reads: None,
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
        self.occurrence_capture = Some(Box::new(capture));
        self
    }

    /// Continue an already-owned packet through the existing template walk.
    #[doc(hidden)]
    pub fn with_binding_occurrence_packet(mut self, mut packet: BindingOccurrences) -> Self {
        let valid = packet
            .resolve_script_globals(&self.croquis.binding_spans)
            .is_some();
        let runtime_reads = self
            .occurrence_capture
            .as_mut()
            .and_then(|capture| capture.runtime_reads.take())
            .or_else(|| self.croquis.setup_context.take_runtime_template_reads());
        self.occurrence_capture = Some(Box::new(OccurrenceCapture {
            packet,
            runtime_reads,
            valid,
            ..Default::default()
        }));
        self
    }

    pub(super) fn with_runtime_template_read_capture(mut self) -> Self {
        if let Some(runtime_reads) = self.croquis.setup_context.take_runtime_template_reads() {
            self.occurrence_capture = Some(Box::new(OccurrenceCapture {
                runtime_reads: Some(runtime_reads),
                valid: false,
                ..Default::default()
            }));
        }
        self
    }

    /// Return the ordinary Croquis and the separately-owned editor fact packet.
    #[doc(hidden)]
    pub fn finish_with_binding_occurrences(self) -> (crate::Croquis, Option<BindingOccurrences>) {
        self.finish_occurrence_packet(true, true)
    }

    /// Preserve source-witnessed setup reads until the actual split-script join.
    #[doc(hidden)]
    pub fn finish_script_occurrences(self) -> (crate::Croquis, Option<BindingOccurrences>) {
        self.finish_occurrence_packet(false, true)
    }

    pub(super) fn finish_occurrence_packet(
        self,
        resolve: bool,
        retain: bool,
    ) -> (crate::Croquis, Option<BindingOccurrences>) {
        let demanded = self.occurrence_capture.is_some();
        let mut runtime_reads = None;
        let packet = self.occurrence_capture.and_then(|capture| {
            let mut capture = *capture;
            // Positive import facts have their own source witnesses. A refused
            // complete editor relation does not invalidate those declarations.
            runtime_reads = capture.runtime_reads;
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
        let mut croquis = self.croquis;
        if retain && demanded {
            croquis.setup_context.set_ssr_occurrences(packet.clone());
            croquis
                .setup_context
                .set_runtime_template_reads(runtime_reads);
        }
        (croquis, packet)
    }

    #[inline]
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
            capture.runtime_reads = None;
            return;
        };
        capture.runtime_reads = runtime_import_seeds(result, source);
        if (*script)
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
        let checks_binding_reads = self.checks_binding_reads();
        let Some(capture) = self.occurrence_capture.as_mut() else {
            return;
        };
        let Some((references, runtime_only)) = capture.references.get(content) else {
            return;
        };
        if *runtime_only && checks_binding_reads {
            note_runtime_import_reads(
                capture.runtime_reads.as_mut(),
                &self.croquis,
                &self.template_source,
                references,
                base_offset,
                scope_vars,
            );
        }
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

/// Import spans already come from the original script AST walk. Capture exists
/// only when the parser admitted that source, independently of editor coverage.
fn runtime_import_seeds(
    result: &crate::script_parser::ScriptParseResult,
    source: &str,
) -> Option<BindingOccurrences> {
    result.scopes.find_scope_by_kind(ScopeKind::ScriptSetup)?;
    let mut packet = BindingOccurrences::default();
    for scope in result.scopes.iter() {
        let ScopeData::ExternalModule(import) = scope.data() else {
            continue;
        };
        let parent = result.scopes.get_scope(scope.parent()?)?;
        if import.is_type_only
            || parent.kind != ScopeKind::ScriptSetup
            || !import.source.split(['?', '#']).next()?.ends_with(".vue")
        {
            continue;
        }
        for (name, binding) in scope.bindings() {
            if binding.binding_type != vize_relief::BindingType::SetupConst {
                continue;
            }
            let &(start, end) = result.binding_spans.get(name)?;
            if source.get(start as usize..end as usize) != Some(name) {
                return None;
            }
            packet.note_binding(parent.id, name, start, end);
        }
    }
    Some(packet)
}

fn note_runtime_import_reads(
    packet: Option<&mut BindingOccurrences>,
    croquis: &crate::Croquis,
    source: &str,
    references: &[IdentifierRef],
    base_offset: u32,
    scope_vars: &[CompactString],
) {
    let Some(packet) = packet else { return };
    for reference in references {
        let name = reference.name.as_str();
        if scope_vars.iter().any(|local| local.as_str() == name)
            || croquis.scopes.lookup(name).is_some_and(|(scope, _)| {
                matches!(
                    scope.kind,
                    ScopeKind::VFor
                        | ScopeKind::VSlot
                        | ScopeKind::EventHandler
                        | ScopeKind::Callback
                        | ScopeKind::Closure
                        | ScopeKind::Block
                )
            })
        {
            continue;
        }
        let Some(&(declaration_start, declaration_end)) = croquis.binding_spans.get(name) else {
            continue;
        };
        let Some(binding) = packet.declaration(name, declaration_start, declaration_end) else {
            continue;
        };
        let Some(start) = base_offset.checked_add(reference.offset) else {
            continue;
        };
        let Some(end) = start.checked_add(name.len() as u32) else {
            continue;
        };
        if source.get(start as usize..end as usize) == Some(name) {
            packet.note_reference(binding, start, end, OccurrenceBlock::Template);
        }
    }
}

#[cfg(test)]
mod tests;
