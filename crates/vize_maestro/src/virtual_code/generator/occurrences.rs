//! Checked physical projections of the existing authored occurrence producer.

use vize_atelier_sfc::{SfcDescriptor, croquis::SfcCroquisAnalysis};
use vize_carton::{CompactString, FxHashMap, FxHashSet};
use vize_croquis::binding_occurrences::{BindingIdentity, BindingOccurrences, OccurrenceBlock};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct PhysicalReference {
    pub(crate) binding: BindingIdentity,
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) block: OccurrenceBlock,
}

#[derive(Debug, Clone)]
pub(crate) struct PhysicalBinding {
    pub(crate) identity: BindingIdentity,
    pub(crate) name: CompactString,
    pub(crate) lens: bool,
    pub(crate) lens_group: u8,
}

#[derive(Debug, Default)]
pub(crate) struct PhysicalOccurrences {
    pub(crate) bindings: FxHashMap<BindingIdentity, PhysicalBinding>,
    pub(crate) references: Vec<PhysicalReference>,
    template_style_counts: FxHashMap<BindingIdentity, usize>,
}

impl PhysicalOccurrences {
    /// Select only original blocks owned by this existing analysis path.
    pub(crate) fn supports_sfc_descriptor(descriptor: &SfcDescriptor<'_>) -> bool {
        if [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
            .into_iter()
            .flatten()
            .any(|script| {
                script.src.is_some()
                    || script
                        .lang
                        .as_deref()
                        .is_some_and(|lang| !matches!(lang, "js" | "ts" | "jsx" | "tsx"))
            })
            || descriptor.styles.iter().any(|style| style.src.is_some())
        {
            return false;
        }
        match descriptor.template.as_ref() {
            Some(template) => {
                template.src.is_none()
                    && template.lang.as_deref().is_none_or(|lang| lang == "html")
                    && !template.has_root_match()
            }
            None => {
                descriptor.script_setup.is_some()
                    && descriptor.script.is_none()
                    && descriptor.styles.is_empty()
            }
        }
    }
    pub(crate) fn from_sfc(
        packet: BindingOccurrences,
        analysis: &SfcCroquisAnalysis,
        descriptor: &SfcDescriptor<'_>,
        source: &str,
    ) -> Option<Self> {
        let mut output = Self::project(packet, source, |block, start, end| {
            let (start, end) = match block {
                OccurrenceBlock::Script => {
                    let script = analysis.script_content_ref()?;
                    script.get(start as usize..end as usize)?;
                    let physical_start = analysis.script_source_offset(descriptor, start) as usize;
                    let physical_end = analysis.script_source_offset(descriptor, end) as usize;
                    let in_block = [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
                        .into_iter()
                        .flatten()
                        .any(|script| {
                            physical_start >= script.loc.start && physical_end <= script.loc.end
                        });
                    if !in_block
                        || physical_end.checked_sub(physical_start)? != (end - start) as usize
                    {
                        return None;
                    }
                    (physical_start, physical_end)
                }
                OccurrenceBlock::Template => {
                    let template = descriptor.template.as_ref()?;
                    template.content.get(start as usize..end as usize)?;
                    (
                        template.loc.start.checked_add(start as usize)?,
                        template.loc.start.checked_add(end as usize)?,
                    )
                }
                OccurrenceBlock::Style(index) => {
                    let style = descriptor.styles.get(index as usize)?;
                    style.content.get(start as usize..end as usize)?;
                    (
                        style.loc.start.checked_add(start as usize)?,
                        style.loc.start.checked_add(end as usize)?,
                    )
                }
            };
            Some((start, end))
        })?;
        for binding in output.bindings.values_mut() {
            binding.lens_group = if descriptor.script_setup.as_ref().is_some_and(|setup| {
                binding.identity.start as usize >= setup.loc.start
                    && binding.identity.end as usize <= setup.loc.end
            }) {
                0
            } else {
                1
            };
        }
        Some(output)
    }

    pub(crate) fn from_fragment(
        packet: BindingOccurrences,
        script_offset: u32,
        template_offset: u32,
        source: &str,
    ) -> Option<Self> {
        Self::project(packet, source, |block, start, end| {
            let base = match block {
                OccurrenceBlock::Script => script_offset,
                OccurrenceBlock::Template => template_offset,
                OccurrenceBlock::Style(_) => return None,
            };
            Some((
                base.checked_add(start)? as usize,
                base.checked_add(end)? as usize,
            ))
        })
    }

    fn project(
        packet: BindingOccurrences,
        source: &str,
        mut offsets: impl FnMut(OccurrenceBlock, u32, u32) -> Option<(usize, usize)>,
    ) -> Option<Self> {
        let mut output = Self::default();
        let mut identities = FxHashMap::default();
        for binding in packet.bindings() {
            let id = binding.identity;
            let (start, end) = offsets(id.block, id.start, id.end)?;
            if source.get(start..end) != Some(binding.name.as_str()) {
                return None;
            }
            let identity = BindingIdentity {
                start: u32::try_from(start).ok()?,
                end: u32::try_from(end).ok()?,
                ..id
            };
            identities.insert(id, identity);
            output.bindings.insert(
                identity,
                PhysicalBinding {
                    identity,
                    name: binding.name.clone(),
                    lens: binding.lens,
                    lens_group: 0,
                },
            );
        }
        for occurrence in packet.occurrences() {
            let binding = *identities.get(&occurrence.binding)?;
            let name = output.bindings.get(&binding)?.name.as_str();
            let (start, end) = offsets(occurrence.block, occurrence.start, occurrence.end)?;
            if source.get(start..end) != Some(name) {
                return None;
            }
            if occurrence.block != OccurrenceBlock::Script {
                *output.template_style_counts.entry(binding).or_default() += 1;
            }
            output.references.push(PhysicalReference {
                binding,
                start,
                end,
                block: occurrence.block,
            });
        }
        output
            .references
            .sort_unstable_by_key(|reference| (reference.start, reference.end));
        Some(output)
    }

    pub(crate) fn merge(&mut self, other: Self) {
        self.bindings.extend(other.bindings);
        let mut seen: FxHashSet<_> = self.references.iter().cloned().collect();
        for reference in other.references {
            if seen.insert(reference.clone()) {
                if reference.block != OccurrenceBlock::Script {
                    *self
                        .template_style_counts
                        .entry(reference.binding)
                        .or_default() += 1;
                }
                self.references.push(reference);
            }
        }
        self.references
            .sort_unstable_by_key(|reference| (reference.start, reference.end));
    }

    pub(crate) fn template_style_count(&self, binding: BindingIdentity) -> usize {
        self.template_style_counts
            .get(&binding)
            .copied()
            .unwrap_or(0)
    }

    pub(crate) fn binding_at(&self, offset: usize) -> Option<BindingIdentity> {
        self.bindings
            .keys()
            .copied()
            .find(|binding| offset >= binding.start as usize && offset <= binding.end as usize)
            .or_else(|| {
                self.references
                    .iter()
                    .find(|reference| offset >= reference.start && offset <= reference.end)
                    .map(|reference| reference.binding)
            })
    }
}
