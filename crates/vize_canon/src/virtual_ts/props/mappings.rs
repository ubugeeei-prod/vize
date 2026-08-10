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

    pub(crate) fn map_exported_props_type(&mut self, ts: &str, generated_start: usize) {
        let Some(call) = self.summary.macros.define_props() else {
            return;
        };
        let Some(type_args) = call.type_args.as_deref() else {
            return;
        };
        let emitted = type_args
            .strip_prefix('<')
            .and_then(|value| value.strip_suffix('>'))
            .unwrap_or(type_args);
        let Some(script) = self.script_content else {
            return;
        };
        let Some(call_source) = script.get(call.start as usize..call.end as usize) else {
            return;
        };
        let Some(authored_type_start) = call_source
            .find(type_args)
            .and_then(|start| type_args.find(emitted).map(|inner| start + inner))
        else {
            return;
        };
        let generated = &ts[generated_start..];
        let Some(generated_type_start) = generated
            .find("export type Props")
            .and_then(|start| generated[start..].find(" = ").map(|rhs| start + rhs + 3))
            .and_then(|start| generated[start..].find(emitted).map(|inner| start + inner))
        else {
            return;
        };
        let authored_start = (self.script_source_offset)(call.start as usize + authored_type_start);
        self.mappings.push(VizeMapping {
            gen_range: generated_start + generated_type_start
                ..generated_start + generated_type_start + emitted.len(),
            src_range: authored_start..authored_start + emitted.len(),
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
