//! The measured proxy arm, retained for spike compatibility.
#![expect(clippy::disallowed_types, reason = "N-API values use std strings")]
#![expect(clippy::disallowed_methods, reason = "N-API values use std strings")]
#![expect(clippy::disallowed_macros, reason = "N-API values use std strings")]
use super::{
    document::{self, PluginDocument},
    facts::{self, REGISTRY, TemplateScopes, resolve_demands},
    host_error,
};
use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use vize_l0::fact::FactManager;

/// The proxy arm the spike measured and rejected: the same document behind
/// a handle whose every read is one napi call.
#[napi]
pub struct PluginDocumentHandle {
    document: PluginDocument,
    scopes: Vec<(u32, Vec<facts::ScopeEntry>)>,
}

/// Open the proxy-arm handle over one SFC, computing `templateScopes`.
#[napi(js_name = "openPluginDocument")]
pub fn open_plugin_document(
    source: String,
    filename: Option<String>,
) -> Result<PluginDocumentHandle> {
    let filename = filename.unwrap_or_else(|| "anonymous.vue".to_owned());
    let document = PluginDocument::build(&source, &filename).map_err(host_error)?;
    let demand = resolve_demands("proxy", &["templateScopes".to_owned()]).map_err(host_error)?;
    let mut manager = FactManager::new(&REGISTRY);
    manager
        .compute(&document, demand)
        .map_err(|error| Error::new(Status::GenericFailure, format!("{error:?}")))?;
    let view = manager.view::<facts::JsPluginHost>();
    let scopes = view
        .get::<TemplateScopes>()
        .map(|table| {
            table
                .iter()
                .map(|(id, entries)| (*id, entries.clone()))
                .collect()
        })
        .unwrap_or_default();
    Ok(PluginDocumentHandle { document, scopes })
}

#[napi]
impl PluginDocumentHandle {
    /// How many nodes the document has (ids are `0..count`).
    #[napi]
    pub fn count(&self) -> u32 {
        self.document.nodes.len() as u32
    }

    #[napi]
    pub fn kind(&self, id: u32) -> Option<String> {
        self.node(id).map(|node| node.kind.to_owned())
    }

    /// The owning node, or -1.
    #[napi]
    pub fn parent(&self, id: u32) -> i32 {
        self.node(id)
            .and_then(|node| node.parent)
            .map_or(-1, |parent| parent as i32)
    }

    /// `name`, `value`, `alias.value`, `alias.key` or `alias.index`.
    #[napi]
    pub fn field(&self, id: u32, key: String) -> Option<String> {
        let node = self.node(id)?;
        let alias = node.alias.as_ref();
        match key.as_str() {
            "name" => node.name.clone(),
            "value" => node.value.clone(),
            "alias.value" => alias.map(|alias| alias.value.clone()),
            "alias.key" => alias.and_then(|alias| alias.key.clone()),
            "alias.index" => alias.and_then(|alias| alias.index.clone()),
            _ => None,
        }
    }

    /// The names scope `id` binds (`templateScopes`), or `null`.
    #[napi]
    pub fn scope(&self, id: u32) -> Option<Vec<ScopeEntryNapi>> {
        let (_, entries) = self.scopes.iter().find(|(scope, _)| *scope == id)?;
        let own = |entry: &facts::ScopeEntry| ScopeEntryNapi {
            name: entry.name.clone(),
            position: entry.position.to_owned(),
        };
        Some(entries.iter().map(own).collect())
    }
}

/// One `templateScopes` entry on the proxy arm.
#[napi(object)]
pub struct ScopeEntryNapi {
    pub name: String,
    pub position: String,
}

impl PluginDocumentHandle {
    fn node(&self, id: u32) -> Option<&document::PluginNode> {
        self.document.nodes.get(id as usize)
    }
}
