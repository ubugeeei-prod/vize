//! Immutable catalog identities resolve independently of document/typechecker state.

#[cfg(feature = "native")]
use super::{data, documentation, values, vue};
use super::{
    data::{CssEntry, CssValue},
    values::StandardValue,
    vue::VueFeature,
};
use serde_json::{Value, json};
#[cfg(feature = "native")]
use tower_lsp::lsp_types::CompletionItem;
use tower_lsp::lsp_types::CompletionItemKind;

const KEY: &str = "vizeCss";

#[derive(Clone, Copy)]
pub(super) enum Selected {
    Entry(&'static str, &'static CssEntry),
    Value(&'static CssEntry, &'static CssValue),
    Wide(&'static CssEntry, &'static StandardValue),
    Color(&'static CssEntry, &'static CssValue),
    Function(&'static CssEntry, &'static StandardValue),
    Vue(&'static VueFeature),
}

impl Selected {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Entry(_, entry) => entry.name,
            Self::Value(_, value) | Self::Color(_, value) => value.name,
            Self::Wide(_, value) => value.name,
            Self::Function(_, value) => {
                if value.name == "var" {
                    "var()"
                } else {
                    "calc()"
                }
            }
            Self::Vue(feature) => feature.name,
        }
    }

    pub(super) fn kind(self) -> CompletionItemKind {
        match self {
            Self::Entry("property", _) => CompletionItemKind::PROPERTY,
            Self::Entry("at-rule", _) => CompletionItemKind::KEYWORD,
            Self::Entry(_, _) | Self::Function(_, _) | Self::Vue(_) => CompletionItemKind::FUNCTION,
            _ => CompletionItemKind::VALUE,
        }
    }

    pub(super) fn data(self) -> Value {
        let (kind, name, property) = match self {
            Self::Entry(kind, entry) => (kind, entry.name, None),
            Self::Value(property, value) => ("value", value.name, Some(property.name)),
            Self::Wide(property, value) => ("wide", value.name, Some(property.name)),
            Self::Color(property, value) => ("color", value.name, Some(property.name)),
            Self::Function(property, value) => ("function", value.name, Some(property.name)),
            Self::Vue(feature) => ("vue", feature.name, None),
        };
        json!({ KEY: { "kind": kind, "name": name, "property": property } })
    }
}

/// Return whether this is CSS data, including invalid data, so it never reaches Corsa.
#[cfg(feature = "native")]
pub(crate) fn resolve(item: &mut CompletionItem) -> bool {
    let Some(payload) = item.data.as_ref().and_then(|data| data.get(KEY)) else {
        return false;
    };
    if let Some(selected) = decode(payload)
        && selected.label() == item.label
        && Some(selected.kind()) == item.kind
    {
        item.documentation = Some(super::super::markup::markdown_documentation(
            documentation::markdown(selected),
        ));
    }
    true
}

#[cfg(feature = "native")]
fn decode(payload: &Value) -> Option<Selected> {
    let kind = payload.get("kind")?.as_str()?;
    let name = payload.get("name")?.as_str()?;
    let property = || data::property(payload.get("property")?.as_str()?);
    Some(match kind {
        "vue" => Selected::Vue(vue::feature(name)?),
        "property" => Selected::Entry("property", data::property(name)?),
        "pseudo" => Selected::Entry(
            "pseudo",
            data::pseudo_class(name).or_else(|| data::pseudo_element(name))?,
        ),
        "at-rule" => Selected::Entry("at-rule", data::at_rule(name)?),
        "value" => {
            let property = property()?;
            Selected::Value(
                property,
                property.values.iter().find(|value| value.name == name)?,
            )
        }
        "wide" => Selected::Wide(property()?, values::lookup(&values::WIDE, name)?),
        "color" => {
            let property = property()?;
            if !values::accepts_colors(property) {
                return None;
            }
            Selected::Color(property, data::color(name)?)
        }
        "function" => {
            let property = property()?;
            let value = values::lookup(&values::FUNCTIONS, name)?;
            if value.name == "calc" && !values::accepts_calc(property) {
                return None;
            }
            Selected::Function(property, value)
        }
        _ => return None,
    })
}
