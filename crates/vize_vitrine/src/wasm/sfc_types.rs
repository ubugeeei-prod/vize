//! WASM-serializable representations of SFC descriptors and compile results,
//! plus conversions from the internal SFC types.

use serde::Serialize;
use std::collections::BTreeMap;

use crate::CompileResult;
use vize_atelier_sfc::{SfcDescriptor, SfcMacroArtifact};

/// SFC compile result for WASM
#[derive(Serialize)]
pub struct SfcWasmResult {
    pub descriptor: SfcDescriptorWasm,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<CompileResult>,
    pub script: SfcScriptResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css: Option<String>,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", rename = "bindingMetadata")]
    pub binding_metadata: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Vec::is_empty", rename = "macroArtifacts")]
    pub macro_artifacts: Vec<SfcMacroArtifactWasm>,
}

/// Script compilation result
#[derive(Serialize)]
pub struct SfcScriptResult {
    pub code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bindings: Option<serde_json::Value>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcMacroArtifactWasm {
    pub kind: String,
    pub name: String,
    pub source: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module_code: Option<String>,
    pub start: usize,
    pub end: usize,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcDescriptorWasm {
    pub filename: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub template: Option<SfcTemplateBlockWasm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<SfcScriptBlockWasm>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_setup: Option<SfcScriptBlockWasm>,
    pub styles: Vec<SfcStyleBlockWasm>,
    pub custom_blocks: Vec<SfcCustomBlockWasm>,
    pub css_vars: Vec<String>,
    pub slotted: bool,
    pub should_force_reload: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcTemplateBlockWasm {
    pub content: String,
    pub loc: SfcBlockLocationWasm,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    pub attrs: BTreeMap<String, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcScriptBlockWasm {
    pub content: String,
    pub loc: SfcBlockLocationWasm,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    pub setup: bool,
    pub attrs: BTreeMap<String, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcStyleBlockWasm {
    pub content: String,
    pub loc: SfcBlockLocationWasm,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lang: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub src: Option<String>,
    pub scoped: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<String>,
    pub attrs: BTreeMap<String, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcCustomBlockWasm {
    pub r#type: String,
    pub content: String,
    pub attrs: BTreeMap<String, String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcBlockLocationWasm {
    pub start: usize,
    pub end: usize,
    pub tag_start: usize,
    pub tag_end: usize,
    pub start_line: usize,
    pub start_column: usize,
    pub end_line: usize,
    pub end_column: usize,
}

fn attrs_to_wasm(
    attrs: &vize_carton::FxHashMap<std::borrow::Cow<'_, str>, std::borrow::Cow<'_, str>>,
) -> BTreeMap<String, String> {
    attrs
        .iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect()
}

fn loc_to_wasm(loc: &vize_atelier_sfc::BlockLocation) -> SfcBlockLocationWasm {
    SfcBlockLocationWasm {
        start: loc.start,
        end: loc.end,
        tag_start: loc.tag_start,
        tag_end: loc.tag_end,
        start_line: loc.start_line,
        start_column: loc.start_column,
        end_line: loc.end_line,
        end_column: loc.end_column,
    }
}

fn template_block_to_wasm(block: &vize_atelier_sfc::SfcTemplateBlock<'_>) -> SfcTemplateBlockWasm {
    SfcTemplateBlockWasm {
        content: block.content.to_string(),
        loc: loc_to_wasm(&block.loc),
        lang: block.lang.as_ref().map(|value| value.to_string()),
        src: block.src.as_ref().map(|value| value.to_string()),
        attrs: attrs_to_wasm(&block.attrs),
    }
}

fn script_block_to_wasm(block: &vize_atelier_sfc::SfcScriptBlock<'_>) -> SfcScriptBlockWasm {
    SfcScriptBlockWasm {
        content: block.content.to_string(),
        loc: loc_to_wasm(&block.loc),
        lang: block.lang.as_ref().map(|value| value.to_string()),
        src: block.src.as_ref().map(|value| value.to_string()),
        setup: block.setup,
        attrs: attrs_to_wasm(&block.attrs),
    }
}

fn style_block_to_wasm(block: &vize_atelier_sfc::SfcStyleBlock<'_>) -> SfcStyleBlockWasm {
    SfcStyleBlockWasm {
        content: block.content.to_string(),
        loc: loc_to_wasm(&block.loc),
        lang: block.lang.as_ref().map(|value| value.to_string()),
        src: block.src.as_ref().map(|value| value.to_string()),
        scoped: block.scoped,
        module: block.module.as_ref().map(|value| value.to_string()),
        attrs: attrs_to_wasm(&block.attrs),
    }
}

fn custom_block_to_wasm(block: &vize_atelier_sfc::SfcCustomBlock<'_>) -> SfcCustomBlockWasm {
    SfcCustomBlockWasm {
        r#type: block.block_type.to_string(),
        content: block.content.to_string(),
        attrs: attrs_to_wasm(&block.attrs),
    }
}

pub(crate) fn descriptor_to_wasm(descriptor: &SfcDescriptor<'_>) -> SfcDescriptorWasm {
    SfcDescriptorWasm {
        filename: descriptor.filename.to_string(),
        source: descriptor.source.to_string(),
        template: descriptor.template.as_ref().map(template_block_to_wasm),
        script: descriptor.script.as_ref().map(script_block_to_wasm),
        script_setup: descriptor.script_setup.as_ref().map(script_block_to_wasm),
        styles: descriptor.styles.iter().map(style_block_to_wasm).collect(),
        custom_blocks: descriptor
            .custom_blocks
            .iter()
            .map(custom_block_to_wasm)
            .collect(),
        css_vars: descriptor
            .css_vars
            .iter()
            .map(|value| value.to_string())
            .collect(),
        slotted: descriptor.slotted,
        should_force_reload: descriptor.should_force_reload,
    }
}

pub(crate) fn macro_artifact_to_wasm(artifact: &SfcMacroArtifact) -> SfcMacroArtifactWasm {
    SfcMacroArtifactWasm {
        kind: artifact.kind.to_string(),
        name: artifact.name.to_string(),
        source: artifact.source.to_string(),
        content: artifact.content.to_string(),
        module_code: artifact.module_code.as_ref().map(ToString::to_string),
        start: artifact.start,
        end: artifact.end,
    }
}
