//! Authored anchors for prop bindings synthesized into template scope.

use std::ops::Range;

use vize_carton::{String, append};
use vize_croquis::Croquis;

use super::super::helpers::to_safe_identifier;
use crate::virtual_ts::VizeMapping;

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
    summary: &'a Croquis,
    script_content: Option<&'a str>,
    script_source_offset: &'a dyn Fn(usize) -> usize,
}

impl<'a> PropBindingMappings<'a> {
    pub(crate) fn new(
        mappings: &'a mut Vec<VizeMapping>,
        summary: &'a Croquis,
        script_content: Option<&'a str>,
        script_source_offset: &'a dyn Fn(usize) -> usize,
    ) -> Self {
        Self {
            mappings,
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

        let Some(original) = self.authored_name_range(name) else {
            return;
        };
        self.mappings.push(VizeMapping {
            gen_range: start..start + binding.len(),
            src_range: original,
            sub_spans: Vec::new(),
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
