//! Zero-copy SFC descriptor types.

mod errors;

pub use errors::SfcError;
pub use vize_relief::options::{BindingMetadata, BindingType};

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use vize_carton::{FxHashMap, String};

/// Parsed result of a Vue Single File Component.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcDescriptor<'a> {
    /// Filename.
    #[serde(borrow)]
    pub filename: Cow<'a, str>,
    /// Source code.
    #[serde(borrow)]
    pub source: Cow<'a, str>,
    /// Template block.
    pub template: Option<SfcTemplateBlock<'a>>,
    /// Script block (options API or `<script>` without setup).
    pub script: Option<SfcScriptBlock<'a>>,
    /// Script setup block.
    pub script_setup: Option<SfcScriptBlock<'a>>,
    /// Style blocks.
    pub styles: Vec<SfcStyleBlock<'a>>,
    /// Custom blocks.
    pub custom_blocks: Vec<SfcCustomBlock<'a>>,
    /// CSS variables from `<style>` v-bind expressions.
    #[serde(borrow)]
    pub css_vars: Vec<Cow<'a, str>>,
    /// Whether the SFC uses slots.
    #[serde(default)]
    pub slotted: bool,
    /// Whether the component should force a reload.
    #[serde(default)]
    pub should_force_reload: bool,
}

impl<'a> Default for SfcDescriptor<'a> {
    fn default() -> Self {
        Self {
            filename: Cow::Borrowed(""),
            source: Cow::Borrowed(""),
            template: None,
            script: None,
            script_setup: None,
            styles: Vec::new(),
            custom_blocks: Vec::new(),
            css_vars: Vec::new(),
            slotted: false,
            should_force_reload: false,
        }
    }
}

impl<'a> SfcDescriptor<'a> {
    /// Convert to an owned descriptor for serialization or storage.
    pub fn into_owned(self) -> SfcDescriptor<'static> {
        SfcDescriptor {
            filename: Cow::Owned(self.filename.into_owned()),
            source: Cow::Owned(self.source.into_owned()),
            template: self.template.map(SfcTemplateBlock::into_owned),
            script: self.script.map(SfcScriptBlock::into_owned),
            script_setup: self.script_setup.map(SfcScriptBlock::into_owned),
            styles: self
                .styles
                .into_iter()
                .map(SfcStyleBlock::into_owned)
                .collect(),
            custom_blocks: self
                .custom_blocks
                .into_iter()
                .map(SfcCustomBlock::into_owned)
                .collect(),
            css_vars: self
                .css_vars
                .into_iter()
                .map(|value| Cow::Owned(value.into_owned()))
                .collect(),
            slotted: self.slotted,
            should_force_reload: self.should_force_reload,
        }
    }

    /// Compute the template block content hash.
    pub fn template_hash(&self) -> Option<String> {
        self.template
            .as_ref()
            .map(|template| vize_carton::hash::content_hash(&template.content))
    }

    /// Compute the combined style block content hash.
    pub fn style_hash(&self) -> Option<String> {
        if self.styles.is_empty() {
            return None;
        }
        let mut combined = String::default();
        for style in &self.styles {
            combined.push_str(&style.content);
            combined.push('\0');
        }
        Some(vize_carton::hash::content_hash(&combined))
    }

    /// Compute the combined script block content hash.
    pub fn script_hash(&self) -> Option<String> {
        let script = self.script.as_ref().map(|block| block.content.as_ref());
        let setup = self
            .script_setup
            .as_ref()
            .map(|block| block.content.as_ref());
        match (script, setup) {
            (None, None) => None,
            (Some(script), None) => Some(vize_carton::hash::content_hash(script)),
            (None, Some(setup)) => Some(vize_carton::hash::content_hash(setup)),
            (Some(script), Some(setup)) => {
                let mut combined = String::with_capacity(script.len() + setup.len() + 1);
                combined.push_str(script);
                combined.push('\0');
                combined.push_str(setup);
                Some(vize_carton::hash::content_hash(&combined))
            }
        }
    }
}

/// Template block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcTemplateBlock<'a> {
    /// Block content.
    #[serde(borrow)]
    pub content: Cow<'a, str>,
    /// Block location in the source.
    pub loc: BlockLocation,
    /// Template language (HTML by default).
    #[serde(default, borrow)]
    pub lang: Option<Cow<'a, str>>,
    /// Source attribute for an external template.
    #[serde(default, borrow)]
    pub src: Option<Cow<'a, str>>,
    /// Additional attributes.
    #[serde(default)]
    pub attrs: FxHashMap<Cow<'a, str>, Cow<'a, str>>,
}

impl<'a> SfcTemplateBlock<'a> {
    /// Convert to an owned block.
    pub fn into_owned(self) -> SfcTemplateBlock<'static> {
        SfcTemplateBlock {
            content: Cow::Owned(self.content.into_owned()),
            loc: self.loc,
            lang: self.lang.map(|value| Cow::Owned(value.into_owned())),
            src: self.src.map(|value| Cow::Owned(value.into_owned())),
            attrs: owned_attrs(self.attrs),
        }
    }
}

/// Script block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcScriptBlock<'a> {
    /// Block content.
    #[serde(borrow)]
    pub content: Cow<'a, str>,
    /// Block location in the source.
    pub loc: BlockLocation,
    /// Script language (JavaScript or TypeScript).
    #[serde(default, borrow)]
    pub lang: Option<Cow<'a, str>>,
    /// Source attribute for an external script.
    #[serde(default, borrow)]
    pub src: Option<Cow<'a, str>>,
    /// Whether this is a script-setup block.
    #[serde(default)]
    pub setup: bool,
    /// Additional attributes.
    #[serde(default)]
    pub attrs: FxHashMap<Cow<'a, str>, Cow<'a, str>>,
    /// Binding metadata filled after analysis.
    #[serde(default)]
    pub bindings: Option<BindingMetadata>,
}

impl<'a> SfcScriptBlock<'a> {
    /// Convert to an owned block.
    pub fn into_owned(self) -> SfcScriptBlock<'static> {
        SfcScriptBlock {
            content: Cow::Owned(self.content.into_owned()),
            loc: self.loc,
            lang: self.lang.map(|value| Cow::Owned(value.into_owned())),
            src: self.src.map(|value| Cow::Owned(value.into_owned())),
            setup: self.setup,
            attrs: owned_attrs(self.attrs),
            bindings: self.bindings,
        }
    }
}

/// Style block.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcStyleBlock<'a> {
    /// Block content.
    #[serde(borrow)]
    pub content: Cow<'a, str>,
    /// Block location in the source.
    pub loc: BlockLocation,
    /// Style language.
    #[serde(default, borrow)]
    pub lang: Option<Cow<'a, str>>,
    /// Source attribute for an external style.
    #[serde(default, borrow)]
    pub src: Option<Cow<'a, str>>,
    /// Whether the style is scoped.
    #[serde(default)]
    pub scoped: bool,
    /// CSS module name when this is a module block.
    #[serde(default, borrow)]
    pub module: Option<Cow<'a, str>>,
    /// Additional attributes.
    #[serde(default)]
    pub attrs: FxHashMap<Cow<'a, str>, Cow<'a, str>>,
}

impl<'a> SfcStyleBlock<'a> {
    /// Convert to an owned block.
    pub fn into_owned(self) -> SfcStyleBlock<'static> {
        SfcStyleBlock {
            content: Cow::Owned(self.content.into_owned()),
            loc: self.loc,
            lang: self.lang.map(|value| Cow::Owned(value.into_owned())),
            src: self.src.map(|value| Cow::Owned(value.into_owned())),
            scoped: self.scoped,
            module: self.module.map(|value| Cow::Owned(value.into_owned())),
            attrs: owned_attrs(self.attrs),
        }
    }
}

/// Custom block such as `<i18n>` or `<docs>`.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SfcCustomBlock<'a> {
    /// Block type or tag name.
    #[serde(rename = "type", borrow)]
    pub block_type: Cow<'a, str>,
    /// Block content.
    #[serde(borrow)]
    pub content: Cow<'a, str>,
    /// Block location in the source.
    pub loc: BlockLocation,
    /// Additional attributes.
    #[serde(default)]
    pub attrs: FxHashMap<Cow<'a, str>, Cow<'a, str>>,
}

impl<'a> SfcCustomBlock<'a> {
    /// Convert to an owned block.
    pub fn into_owned(self) -> SfcCustomBlock<'static> {
        SfcCustomBlock {
            block_type: Cow::Owned(self.block_type.into_owned()),
            content: Cow::Owned(self.content.into_owned()),
            loc: self.loc,
            attrs: owned_attrs(self.attrs),
        }
    }
}

/// Source location of an SFC block.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlockLocation {
    /// Start offset of content after the opening tag.
    pub start: usize,
    /// End offset of content before the closing tag.
    pub end: usize,
    /// Start offset of the opening tag.
    #[serde(default)]
    pub tag_start: usize,
    /// End offset of the closing tag.
    #[serde(default)]
    pub tag_end: usize,
    /// One-based line of the first content byte.
    pub start_line: usize,
    /// One-based byte column of the first content byte.
    pub start_column: usize,
    /// One-based line just after the content.
    pub end_line: usize,
    /// One-based byte column just after the content.
    pub end_column: usize,
}

/// SFC parse options.
#[derive(Debug, Clone, Default)]
pub struct SfcParseOptions {
    /// Filename.
    pub filename: String,
    /// Whether source maps should be generated by downstream consumers.
    pub source_map: bool,
    /// Block padding strategy.
    pub pad: PadOption,
    /// Whether empty blocks should be ignored.
    pub ignore_empty: bool,
    /// Template parser options.
    pub template_parse_options: Option<vize_relief::options::ParserOptions>,
}

/// Padding option for source-map alignment.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum PadOption {
    /// No padding.
    #[default]
    None,
    /// Pad with newlines.
    Line,
    /// Pad with spaces.
    Space,
}

fn owned_attrs(
    attrs: FxHashMap<Cow<'_, str>, Cow<'_, str>>,
) -> FxHashMap<Cow<'static, str>, Cow<'static, str>> {
    attrs
        .into_iter()
        .map(|(key, value)| (Cow::Owned(key.into_owned()), Cow::Owned(value.into_owned())))
        .collect()
}
