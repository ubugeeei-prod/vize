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
mod tests {
    use super::*;
    use crate::{DrawerOptions, binding_occurrences::BindingOccurrence};
    use vize_relief::Allocator;

    const SCRIPT: &str =
        "const label = 'Name';\nconst hint = 'Shown below the field';\nconst id = 'field';";
    const TEMPLATE: &str = "<label :for=\"id\">{{ label }}</label>\n<input :id=\"id\" />\n<p :id=\"`${id}-hint`\">{{ hint }}</p>";

    fn capture(script: &str, template: &str) -> BindingOccurrences {
        let allocator = Allocator::default();
        let (root, errors) = vize_armature::parse(&allocator, template);
        assert!(errors.is_empty());
        let mut drawer = Drawer::with_options(DrawerOptions::for_lint()).with_binding_occurrences();
        drawer.draw_script_setup(script);
        drawer.draw_template(&root);
        drawer.finish_with_binding_occurrences().1.unwrap()
    }

    fn references<'a>(packet: &'a BindingOccurrences, name: &str) -> Vec<&'a BindingOccurrence> {
        let binding = packet
            .bindings()
            .find(|binding| {
                binding.name == name && binding.identity.block == OccurrenceBlock::Script
            })
            .unwrap();
        packet
            .occurrences()
            .iter()
            .filter(|occurrence| occurrence.binding == binding.identity)
            .collect()
    }

    #[test]
    fn entire_reported_template_has_only_owned_binding_reads() {
        let packet = capture(SCRIPT, TEMPLATE);
        assert_eq!(references(&packet, "label").len(), 1);
        assert_eq!(references(&packet, "hint").len(), 1);
        assert_eq!(references(&packet, "id").len(), 3);
        for name in ["label", "hint", "id"] {
            let binding = packet
                .bindings()
                .find(|binding| {
                    binding.name == name && binding.identity.block == OccurrenceBlock::Script
                })
                .unwrap();
            assert_eq!(
                SCRIPT
                    .get(binding.identity.start as usize..binding.identity.end as usize)
                    .expect("binding range stays within authored UTF-8 boundaries"),
                name
            );
            for reference in references(&packet, name) {
                assert_eq!(reference.block, OccurrenceBlock::Template);
                assert_eq!(
                    TEMPLATE
                        .get(reference.start as usize..reference.end as usize)
                        .expect("reference range stays within authored UTF-8 boundaries"),
                    name
                );
            }
        }
        assert_eq!(
            references(&packet, "label")[0].start as usize,
            TEMPLATE.find("{{ label").unwrap() + 3
        );
        assert_eq!(
            references(&packet, "hint")[0].start as usize,
            TEMPLATE.find("{{ hint").unwrap() + 3
        );
        // Every `id` read is the bound value, never the `:id` attribute name.
        assert_eq!(
            references(&packet, "id")
                .iter()
                .map(|reference| reference.start)
                .collect::<Vec<_>>(),
            [13, 49, 67]
        );
    }

    #[test]
    fn repeat_cache_keeps_each_location_and_lexical_owner() {
        let template = "<p>{{ id }}</p><p v-for=\"id in [1]\">{{ id }}</p><p>{{ id }}</p>";
        let packet = capture(SCRIPT, template);
        let reads = references(&packet, "id");
        assert_eq!(reads.len(), 2);
        assert_eq!(reads[0].start as usize, template.find("{{ id").unwrap() + 3);
        assert_eq!(
            reads[1].start as usize,
            template.rfind("{{ id").unwrap() + 3
        );
        assert_ne!(reads[0].start, reads[1].start);
    }

    #[test]
    fn property_keys_strings_comments_and_expression_locals_are_not_global_reads() {
        let template = "<p :title=\"({ id: label, text: 'hint', [id]: hint })\">{{ /* id */ label }}{{ ((id) => id)(1) }}</p>";
        let packet = capture(SCRIPT, template);
        assert_eq!(references(&packet, "label").len(), 2);
        assert_eq!(references(&packet, "hint").len(), 1);
        assert_eq!(references(&packet, "id").len(), 1);
        let id = references(&packet, "id")[0];
        assert_eq!(id.start as usize, template.find("[id]").unwrap() + 1);
        let label = references(&packet, "label")[1];
        assert_eq!(
            label.start as usize,
            template.find("/* id */ label").unwrap() + 9
        );
    }

    #[test]
    fn unicode_and_crlf_keep_authored_byte_offsets_before_utf16_projection() {
        let script = "// 日本語😀\r\nconst 名称 = 'name';\r\n";
        let template = "<p title=\"名称😀\">\r\n{{ 名称 }}\r\n</p>";
        let packet = capture(script, template);
        let reads = references(&packet, "名称");
        assert_eq!(reads.len(), 1);
        assert_eq!(
            reads[0].start as usize,
            template.find("{{ 名称").unwrap() + 3
        );
        assert_eq!(reads[0].end - reads[0].start, "名称".len() as u32);
        let declaration = packet
            .bindings()
            .find(|binding| binding.name == "名称")
            .unwrap();
        assert_eq!(
            declaration.identity.start as usize,
            script.find("名称 =").unwrap()
        );
    }

    #[test]
    fn capture_leaves_ordinary_semantic_snapshot_unchanged() {
        let allocator = Allocator::default();
        let (root, errors) = vize_armature::parse(&allocator, TEMPLATE);
        assert!(errors.is_empty());
        let mut ordinary = Drawer::with_options(DrawerOptions::for_lint());
        ordinary.draw_script_setup(SCRIPT).draw_template(&root);
        let expected = serde_json::to_value(ordinary.finish().semantic_snapshot()).unwrap();
        let mut captured =
            Drawer::with_options(DrawerOptions::for_lint()).with_binding_occurrences();
        captured.draw_script_setup(SCRIPT);
        captured.draw_template(&root);
        let (croquis, packet) = captured.finish_with_binding_occurrences();
        assert_eq!(
            serde_json::to_value(croquis.semantic_snapshot()).unwrap(),
            expected
        );
        assert_eq!(packet.unwrap().occurrences().len(), 5);
    }

    #[test]
    fn ordinary_capture_storage_is_empty_and_pointer_sized() {
        use crate::script_parser::ScriptParseResult;
        use std::mem::{size_of, size_of_val};

        assert_eq!(
            size_of::<Option<Box<super::OccurrenceCapture>>>(),
            size_of::<usize>()
        );
        let parsed = ScriptParseResult::default();
        assert_eq!(size_of_val(&parsed.occurrence_capture), size_of::<usize>());
        assert!(parsed.occurrence_capture.is_none());
        let mut ordinary = Drawer::with_options(DrawerOptions::for_lint());
        assert!(ordinary.occurrence_capture.is_none());
        ordinary.draw_script_setup(SCRIPT);
        assert!(ordinary.occurrence_capture.is_none());
        assert!(
            ordinary
                .with_binding_occurrences()
                .occurrence_capture
                .is_some()
        );
    }

    #[test]
    fn full_analysis_keeps_occurrences_explicit_and_complete_public_outputs_equal() {
        let allocator = Allocator::default();
        let (root, errors) = vize_armature::parse(&allocator, TEMPLATE);
        assert!(errors.is_empty());
        let mut ordinary = Drawer::with_options(DrawerOptions::full());
        assert!(ordinary.occurrence_capture.is_none());
        ordinary.draw_script_setup(SCRIPT).draw_template(&root);
        assert!(ordinary.occurrence_capture.is_none());
        let mut expected = ordinary.finish();
        assert!(expected.setup_context.take_ssr_occurrences().is_none());
        let mut captured = Drawer::with_options(DrawerOptions::full()).with_binding_occurrences();
        captured.draw_script_setup(SCRIPT).draw_template(&root);
        let (actual, packet) = captured.finish_with_binding_occurrences();
        assert_eq!(actual.to_vir(), expected.to_vir());
        assert_eq!(
            serde_json::to_value(actual.semantic_snapshot()).unwrap(),
            serde_json::to_value(expected.semantic_snapshot()).unwrap()
        );
        assert_eq!(packet.unwrap().occurrences().len(), 5);
    }
}
