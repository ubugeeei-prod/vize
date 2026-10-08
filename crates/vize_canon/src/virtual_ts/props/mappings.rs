//! Authored anchors for prop bindings synthesized into template scope.

use std::ops::Range;

use vize_carton::{String, append};
use vize_croquis::Croquis;

use super::super::helpers::to_safe_identifier;
use crate::virtual_ts::{VizeMapping, VizeSemanticLink, VizeSemanticLinkKind};

pub(crate) struct PropsSource<'a> {
    pub(crate) mappings: &'a mut Vec<VizeMapping>,
    pub(crate) summary: &'a Croquis,
    pub(crate) script: Option<&'a str>,
    pub(crate) offset: &'a dyn Fn(usize) -> usize,
}

pub(crate) fn prop_source<'a>(
    mappings: &'a mut Vec<VizeMapping>,
    summary: &'a Croquis,
    script: Option<&'a str>,
    offset: &'a dyn Fn(usize) -> usize,
) -> PropsSource<'a> {
    PropsSource {
        mappings,
        summary,
        script,
        offset,
    }
}

pub(crate) struct PropBindingMappings<'a> {
    mappings: &'a mut Vec<VizeMapping>,
    semantic_links: &'a mut Vec<VizeSemanticLink>,
    summary: &'a Croquis,
    script_content: Option<&'a str>,
    script_source_offset: &'a dyn Fn(usize) -> usize,
}

impl<'a> PropBindingMappings<'a> {
    pub(crate) fn new(
        mappings: &'a mut Vec<VizeMapping>,
        semantic_links: &'a mut Vec<VizeSemanticLink>,
        summary: &'a Croquis,
        script_content: Option<&'a str>,
        script_source_offset: &'a dyn Fn(usize) -> usize,
    ) -> Self {
        Self {
            mappings,
            semantic_links,
            summary,
            script_content,
            script_source_offset,
        }
    }

    pub(super) fn emit(
        &mut self,
        ts: &mut String,
        props_type_ref: &str,
        name: &str,
        has_default: bool,
    ) {
        let binding = to_safe_identifier(name);
        let start = ts.len() + "  const ".len();
        if has_default {
            append!(
                *ts,
                "  const {binding} = props[\"{name}\"] as Exclude<{props_type_ref}[\"{name}\"], undefined>;\n"
            );
        } else {
            append!(*ts, "  const {binding} = props[\"{name}\"];\n");
        }
        append!(*ts, "  void {binding};\n");
        let property_start = start + binding.len() + " = props[\"".len();
        self.link(
            start..start + binding.len(),
            property_start..property_start + name.len(),
        );

        let Some(original) = self.authored_name_range(name) else {
            return;
        };
        for &(key_start, key_end) in self.summary.macros.with_defaults_key_ranges(name) {
            let source = (self.script_source_offset)(key_start as usize)
                ..(self.script_source_offset)(key_end as usize);
            // Only an exact copied setup mapping can project this retained AST
            // key. Synthetic name mappings and text searches grant no edge.
            for mapping in self.mappings.iter() {
                if mapping.sub_spans.is_empty()
                    && mapping.gen_range.len() == mapping.src_range.len()
                    && mapping.src_range.start <= source.start
                    && source.end <= mapping.src_range.end
                {
                    let generated =
                        mapping.gen_range.start + source.start - mapping.src_range.start;
                    let target = generated..generated + source.len();
                    let authored = self
                        .script_content
                        .and_then(|script| script.get(key_start as usize..key_end as usize));
                    if authored.is_none() || ts.get(target.clone()) != authored {
                        continue;
                    }
                    self.semantic_links.push(VizeSemanticLink {
                        source_range: start..start + binding.len(),
                        target_range: target,
                        kind: VizeSemanticLinkKind::VueTemplatePropBinding,
                    });
                }
            }
        }
        self.mappings.push(VizeMapping {
            gen_range: start..start + binding.len(),
            src_range: original,
            sub_spans: Vec::new(),
        });
    }

    pub(super) fn link(&mut self, binding: Range<usize>, property: Range<usize>) {
        self.semantic_links.push(VizeSemanticLink {
            source_range: binding,
            target_range: property,
            kind: VizeSemanticLinkKind::VueTemplatePropBinding,
        });
    }

    /// The existing setup/template shadow edge also joins the props object.
    /// Endpoints are recorded during emission, never inferred from spelling.
    pub(super) fn link_setup_shadow(&mut self, source: Range<usize>, target: Range<usize>) {
        self.semantic_links.push(VizeSemanticLink {
            source_range: source,
            target_range: target,
            kind: VizeSemanticLinkKind::VueSetupTemplateRefUnwrap,
        });
    }

    fn authored_name_range(&self, name: &str) -> Option<Range<usize>> {
        let script = self.script_content?;
        let (start, end) = self.summary.macros.prop_declaration(name)?;
        let declaration = script.get(start as usize..end as usize)?;
        let relative = declaration.find(name)?;
        let start = (self.script_source_offset)(start as usize + relative);
        Some(start..start + name.len())
    }
}
