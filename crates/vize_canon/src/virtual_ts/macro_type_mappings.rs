//! Authored mappings for module-level types synthesized from compiler macros.

use vize_carton::cstr;
use vize_croquis::macros::MacroCall;

use super::VizeMapping;

pub(crate) struct MacroTypeMappings<'a> {
    mappings: &'a mut Vec<VizeMapping>,
    script: Option<&'a str>,
    source_offset: &'a dyn Fn(usize) -> usize,
}

impl<'a> MacroTypeMappings<'a> {
    pub(crate) fn new(
        mappings: &'a mut Vec<VizeMapping>,
        script: Option<&'a str>,
        source_offset: &'a dyn Fn(usize) -> usize,
    ) -> Self {
        Self {
            mappings,
            script,
            source_offset,
        }
    }

    pub(crate) fn map_exported_type(
        &mut self,
        ts: &str,
        generated_start: usize,
        call: Option<&MacroCall>,
        export_name: &str,
    ) {
        let Some(call) = call else {
            return;
        };
        let Some(type_args) = call.type_args.as_deref() else {
            return;
        };
        let emitted = type_args
            .strip_prefix('<')
            .and_then(|value| value.strip_suffix('>'))
            .unwrap_or(type_args);
        let Some(script) = self.script else {
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
        let declaration = cstr!("export type {export_name}");
        let Some(generated_type_start) = generated
            .find(declaration.as_str())
            .and_then(|start| generated[start..].find(" = ").map(|rhs| start + rhs + 3))
            .and_then(|start| generated[start..].find(emitted).map(|inner| start + inner))
        else {
            return;
        };
        let authored_start = (self.source_offset)(call.start as usize + authored_type_start);
        self.mappings.push(VizeMapping {
            gen_range: generated_start + generated_type_start
                ..generated_start + generated_type_start + emitted.len(),
            src_range: authored_start..authored_start + emitted.len(),
            sub_spans: Vec::new(),
        });
    }
}
