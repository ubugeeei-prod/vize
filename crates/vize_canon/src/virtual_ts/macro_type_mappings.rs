//! Authored mappings for module-level types synthesized from compiler macros.

use std::ops::Range;

use vize_carton::{FxHashSet, String, cstr};
use vize_croquis::macros::{MacroCall, ModelDefinition};

use crate::virtual_ts::helpers::push_ts_string_literal;

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
        let Some(generated) = ts.get(generated_start..) else {
            return;
        };
        let declaration = cstr!("export type {export_name}");
        let Some(generated_type_start) = generated
            .find(declaration.as_str())
            .and_then(|start| Some(start + generated.get(start..)?.find(" = ")? + 3))
            .and_then(|start| Some(start + generated.get(start..)?.find(emitted)?))
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

    pub(crate) fn authored_text(&self, range: (u32, u32)) -> Option<&str> {
        self.script?.get(range.0 as usize..range.1 as usize)
    }

    /// Map only retained static slot keys in the unchanged public type copy.
    /// Payload references keep their original setup-scope diagnostic ownership.
    pub(crate) fn map_slot_keys(
        &mut self,
        generated: Range<usize>,
        declarations: &vize_croquis::macros::MacroTracker,
    ) {
        let Some((inner_start, inner_end)) = declarations.slot_type_argument_range() else {
            return;
        };
        if inner_end.saturating_sub(inner_start) as usize != generated.len() {
            return;
        }
        let inner_start = inner_start as usize;
        for authored in declarations.static_slot_declarations() {
            let Some(start) = (authored.0 as usize).checked_sub(inner_start) else {
                continue;
            };
            let Some(end) = (authored.1 as usize).checked_sub(inner_start) else {
                continue;
            };
            if start < end && end <= generated.len() {
                self.map_exact(generated.start + start..generated.start + end, authored);
            }
        }
    }

    pub(crate) fn map_model_props(
        &mut self,
        ts: &str,
        generated_start: usize,
        models: &[ModelDefinition],
        declarations: &vize_croquis::macros::MacroTracker,
        emitted_model_names: &FxHashSet<String>,
    ) {
        let generated = ts.get(generated_start..).unwrap_or_default();
        for model in models {
            if !emitted_model_names
                .iter()
                .any(|name| name.as_str() == model.name.as_str())
            {
                continue;
            }
            let Some(authored) = declarations.model_declaration(model.name.as_str()) else {
                continue;
            };
            let mut needle = String::from("  ");
            push_ts_string_literal(&mut needle, model.name.as_str());
            let Some(start) = generated.find(needle.as_str()) else {
                continue;
            };
            let start = generated_start + start + 2;
            self.map_model_symbol(
                ts,
                start..start + needle.len() - 2,
                authored,
                model.name.as_str(),
            );
        }
    }

    /// Keep quoted model names and synthesized update keys on the retained
    /// authored name. Whole literal definitions and interior references use
    /// different native spans; the update prefix has no authored counterpart.
    pub(crate) fn map_model_symbol(
        &mut self,
        ts: &str,
        generated: Range<usize>,
        authored: (u32, u32),
        name: &str,
    ) {
        let source_name = self.authored_text(authored).and_then(quoted_content);
        let generated_name = ts.get(generated.clone()).and_then(quoted_content);
        let verified = !name.is_empty()
            && source_name == Some(name)
            && generated_name
                .is_some_and(|text| text == name || text.strip_prefix("update:") == Some(name));
        if !verified {
            self.map_whole_symbol(generated, authored);
            return;
        }
        let content = (authored.0 + 1, authored.1 - 1);
        let inner = generated.start + 1..generated.end - 1;
        self.map_whole_symbol(generated, content);
        if inner.len() != name.len() {
            self.map_whole_symbol(inner.clone(), content);
        }
        self.map_exact(inner.end - name.len()..inner.end, content);
    }

    pub(crate) fn map_exact(&mut self, generated: Range<usize>, authored: (u32, u32)) {
        let Some(authored_len) = authored.1.checked_sub(authored.0).map(|len| len as usize) else {
            return;
        };
        if generated.len() != authored_len {
            return;
        }
        let authored_start = (self.source_offset)(authored.0 as usize);
        self.mappings.push(VizeMapping {
            gen_range: generated,
            src_range: authored_start..authored_start + authored_len,
            sub_spans: Vec::new(),
        });
    }

    pub(crate) fn map_whole_symbol(&mut self, generated: Range<usize>, authored: (u32, u32)) {
        let Some(authored_len) = authored.1.checked_sub(authored.0).map(|len| len as usize) else {
            return;
        };
        let authored_start = (self.source_offset)(authored.0 as usize);
        self.mappings.push(VizeMapping {
            gen_range: generated,
            src_range: authored_start..authored_start + authored_len,
            sub_spans: Vec::new(),
        });
    }
}

fn quoted_content(text: &str) -> Option<&str> {
    let quote = *text.as_bytes().first()?;
    if matches!(quote, b'\'' | b'"' | b'`') && text.as_bytes().last() == Some(&quote) {
        text.get(1..text.len().checked_sub(1)?)
    } else {
        None
    }
}

#[cfg(test)]
mod model_symbols_tests;
