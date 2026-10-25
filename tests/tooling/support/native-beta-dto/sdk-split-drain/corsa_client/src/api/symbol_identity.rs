//! Symbol identity for the [`DerivedSnapshots`](super::ApiDialect::DerivedSnapshots) dialect.
//!
//! TypeScript 7.0 identified a symbol by one number. TypeScript 7.1 identifies
//! it by a *reference* that names its owner: binder-produced symbols belong to
//! the source file that declares them, and everything the checker synthesizes
//! belongs to a snapshot. Responses also stopped repeating that reference where
//! a symbol is merely mentioned — a type's `symbol`, a signature's
//! `parameters` — and send a compact `{ id, file }` pair instead.
//!
//! `corsa-bind` keeps [`SymbolHandle`](super::SymbolHandle) opaque across that
//! change. On this dialect a handle is the compact pair, rendered as one
//! canonical string, and the client supplies the rest of the reference when
//! the handle is sent back:
//!
//! | owner    | handle                  | completed with |
//! | -------- | ----------------------- | -------------- |
//! | file     | `{"file":"142","id":6}` | the file's descriptor, remembered from the first full reference that carried it |
//! | snapshot | `{"id":9,"snapshot":3}` | the project the request itself names |
//!
//! That keeps three properties the rest of the workspace relies on:
//!
//! - a handle round-trips through every binding as a short plain string
//! - a symbol has one handle, whether a response carried it or only mentioned
//!   it, so handles stay usable as map keys
//! - adapting a response never costs a request
//!
//! It also keeps what a mention means. On TypeScript 7.0 a symbol id embedded
//! in another response only resolves once the runtime has handed that symbol
//! out in full; here a mention only resolves once its owner is known the same
//! way. Either way the failure is a stale handle, which
//! [`ApiClient::is_stale_handle_error`](crate::ApiClient::is_stale_handle_error)
//! recognizes, and the dedicated endpoint (`getSymbolOfType`,
//! `getParametersOfSignature`, …) is how a caller asks for the symbol itself.
//!
//! Only the fields the typed API exposes as handles are adapted: a symbol's
//! own `reference`, a type's `symbol`, and a signature's `parameters` and
//! `thisParameter`. A symbol's `parent` and `exportSymbol` and a type's
//! `aliasSymbol` pass through as upstream sent them; they only reach callers
//! of the raw endpoints, who get upstream's shape there by design.

use corsa_core::fast::{CompactString, FastMap, compact_format};
use parking_lot::Mutex;
use serde_json::{Map, Value, json};

use crate::{CorsaError, Result};

/// Source file descriptors remembered per generation; see [`FileMemo`].
const FILE_MEMO_GENERATION: usize = 32_768;

/// `SymbolOwnerKind` values upstream puts on the wire.
const OWNER_KIND_FILE: u64 = 0;
const OWNER_KIND_SNAPSHOT: u64 = 1;

/// Marks the error for a handle whose owner this connection has not seen.
///
/// Shared with the stale-handle classifier so the two cannot drift apart.
pub(crate) const UNKNOWN_SYMBOL_OWNER: &str = "symbol owner is unknown to this connection";

/// Symbol owners one connection has learned about.
#[derive(Default)]
pub(crate) struct SymbolIdentity {
    files: Mutex<FileMemo>,
}

/// Source file node id to the descriptor upstream resolves that file by.
///
/// A node id names one parsed version of one file, so entries never change,
/// but a long-lived connection keeps meeting new versions. Entries live in two
/// generations: a lookup that finds one in the old generation moves it to the
/// young one, and the old generation is dropped whenever the young one fills.
/// A descriptor therefore survives for as long as it is used, and one that goes
/// unused belongs to a file version no live snapshot is likely to still hold.
struct FileMemo {
    young: FastMap<CompactString, Value>,
    old: FastMap<CompactString, Value>,
    /// Entries the young generation holds before the old one is dropped.
    generation: usize,
}

impl Default for FileMemo {
    fn default() -> Self {
        Self::with_generation(FILE_MEMO_GENERATION)
    }
}

impl FileMemo {
    fn with_generation(generation: usize) -> Self {
        Self {
            young: FastMap::default(),
            old: FastMap::default(),
            generation,
        }
    }

    fn remember(&mut self, node_id: &str, descriptor: &Map<String, Value>) {
        if self.young.contains_key(node_id) {
            return;
        }
        let descriptor = self
            .old
            .remove(node_id)
            .unwrap_or_else(|| Value::Object(descriptor.clone()));
        self.insert(CompactString::from(node_id), descriptor);
    }

    fn get(&mut self, node_id: &str) -> Option<Value> {
        if let Some(descriptor) = self.young.get(node_id) {
            return Some(descriptor.clone());
        }
        let descriptor = self.old.remove(node_id)?;
        self.insert(CompactString::from(node_id), descriptor.clone());
        Some(descriptor)
    }

    fn insert(&mut self, node_id: CompactString, descriptor: Value) {
        if self.young.len() >= self.generation {
            self.old = std::mem::take(&mut self.young);
        }
        self.young.insert(node_id, descriptor);
    }
}

/// Snapshot and project a request addressed, as far as its params say.
#[derive(Clone, Debug, Default)]
pub(crate) struct RequestScope {
    snapshot: Option<u64>,
    project: Option<CompactString>,
}

impl RequestScope {
    /// Reads the scope out of request params.
    pub(crate) fn from_params(params: &Value) -> Self {
        Self {
            snapshot: params.get("snapshot").and_then(handle_number),
            project: params
                .get("project")
                .and_then(Value::as_str)
                .filter(|project| !project.is_empty())
                .map(CompactString::from),
        }
    }
}

impl SymbolIdentity {
    /// Rewrites `response` so every symbol the typed API exposes is a handle.
    pub(crate) fn adopt(&self, response: &mut Value, scope: &RequestScope) {
        match response {
            Value::Array(items) => {
                for item in items {
                    self.adopt(item, scope);
                }
            }
            Value::Object(fields) => {
                self.adopt_object(fields, scope);
                for value in fields.values_mut() {
                    self.adopt(value, scope);
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        }
    }

    fn adopt_object(&self, fields: &mut Map<String, Value>, scope: &RequestScope) {
        if let Some(handle) = fields
            .get("reference")
            .and_then(full_reference)
            .and_then(|reference| self.adopt_reference(reference, scope))
        {
            fields.remove("reference");
            fields.insert("id".into(), Value::String(handle));
            return;
        }
        if !is_checker_object(fields) {
            return;
        }
        for key in ["symbol", "thisParameter"] {
            if let Some(handle) = fields
                .get(key)
                .and_then(|value| mention_handle(value, scope))
            {
                fields.insert(key.into(), Value::String(handle));
            }
        }
        let Some(Value::Array(parameters)) = fields.get_mut("parameters") else {
            return;
        };
        let handles = parameters
            .iter()
            .map(|parameter| mention_handle(parameter, scope))
            .collect::<Option<Vec<_>>>();
        if let Some(handles) = handles {
            *parameters = handles.into_iter().map(Value::String).collect();
        }
    }

    /// Renders the handle for a full reference, remembering a file owner.
    ///
    /// A reference that does not carry what its owner kind requires is left
    /// for the caller to see as upstream sent it.
    fn adopt_reference(
        &self,
        reference: &Map<String, Value>,
        scope: &RequestScope,
    ) -> Option<String> {
        let id = reference.get("id").and_then(Value::as_u64)?;
        match reference.get("kind").and_then(Value::as_u64)? {
            OWNER_KIND_FILE => {
                let descriptor = reference.get("file").and_then(Value::as_object)?;
                let node_id = descriptor.get("nodeId").and_then(Value::as_str)?;
                self.files.lock().remember(node_id, descriptor);
                Some(file_handle(id, node_id))
            }
            // Falling back to the request keeps this the handle a mention of
            // the same symbol renders to.
            OWNER_KIND_SNAPSHOT => Some(snapshot_handle(
                id,
                reference
                    .get("snapshot")
                    .and_then(Value::as_u64)
                    .or(scope.snapshot),
            )),
            _ => None,
        }
    }

    /// Turns the bare ids of `getWellKnownSymbols` into handles.
    ///
    /// That endpoint registers the checker's singleton symbols in the snapshot
    /// it was asked about and answers with their ids alone.
    pub(crate) fn adopt_well_known_symbols(response: &mut Value, scope: &RequestScope) {
        let Value::Object(fields) = response else {
            return;
        };
        for value in fields.values_mut() {
            if let Some(id) = value.as_u64() {
                *value = Value::String(snapshot_handle(id, scope.snapshot));
            }
        }
    }

    /// Replaces symbol handles in request params with wire references.
    pub(crate) fn encode_params(&self, params: &mut Value) -> Result<()> {
        let scope = RequestScope::from_params(params);
        self.encode_value(params, &scope)
    }

    fn encode_value(&self, value: &mut Value, scope: &RequestScope) -> Result<()> {
        match value {
            Value::Array(items) => {
                for item in items {
                    self.encode_value(item, scope)?;
                }
            }
            Value::Object(fields) => {
                for (key, value) in fields {
                    match key.as_str() {
                        "symbol" => self.encode_symbol(value, scope)?,
                        "symbols" => {
                            if let Value::Array(items) = value {
                                for item in items {
                                    self.encode_symbol(item, scope)?;
                                }
                            }
                        }
                        // Batched sub-requests address their own snapshot.
                        "params" => {
                            let scope = RequestScope::from_params(value);
                            self.encode_value(value, &scope)?;
                        }
                        _ => self.encode_value(value, scope)?,
                    }
                }
            }
            Value::Null | Value::Bool(_) | Value::Number(_) | Value::String(_) => {}
        }
        Ok(())
    }

    fn encode_symbol(&self, value: &mut Value, scope: &RequestScope) -> Result<()> {
        let Value::String(handle) = value else {
            return Ok(());
        };
        // Handles from other dialects are numbers or prefixed ids.
        if !handle.starts_with('{') {
            return Ok(());
        }
        let invalid = || CorsaError::InvalidHandle(CompactString::from(handle.as_str()));
        let parsed: Value = serde_json::from_str(handle).map_err(|_| invalid())?;
        let id = parsed
            .get("id")
            .and_then(Value::as_u64)
            .ok_or_else(invalid)?;
        let reference = match parsed.get("file") {
            Some(Value::String(node_id)) => {
                let descriptor = self.files.lock().get(node_id).ok_or_else(|| {
                    CorsaError::Protocol(compact_format(format_args!(
                        "symbol handle {handle} not found: {UNKNOWN_SYMBOL_OWNER}"
                    )))
                })?;
                json!({ "kind": OWNER_KIND_FILE, "file": descriptor, "id": id })
            }
            Some(_) => return Err(invalid()),
            None => json!({
                "kind": OWNER_KIND_SNAPSHOT,
                "snapshot": parsed
                    .get("snapshot")
                    .and_then(Value::as_u64)
                    .or(scope.snapshot)
                    .ok_or_else(invalid)?,
                // The runtime wants the project a checker-owned symbol is
                // looked up in. Left empty, it rejects the reference itself.
                "project": scope.project.as_deref().unwrap_or_default(),
                "id": id,
            }),
        };
        *value = reference;
        Ok(())
    }
}

/// Recognizes a type or signature response, which both carry numeric `id` and `flags`.
fn is_checker_object(fields: &Map<String, Value>) -> bool {
    let is_number = |key| fields.get(key).is_some_and(Value::is_u64);
    is_number("id") && is_number("flags")
}

/// Recognizes a full `SymbolReference`: an object with a numeric `kind` and `id`.
fn full_reference(value: &Value) -> Option<&Map<String, Value>> {
    let reference = value.as_object()?;
    let kind = reference.get("kind").and_then(Value::as_u64)?;
    reference.get("id").and_then(Value::as_u64)?;
    matches!(kind, OWNER_KIND_FILE | OWNER_KIND_SNAPSHOT).then_some(reference)
}

/// Renders the handle for a `CompactSymbolReference`: `{ id }` or `{ id, file }`
/// and nothing else.
fn mention_handle(value: &Value, scope: &RequestScope) -> Option<String> {
    let fields = value.as_object()?;
    let id = fields.get("id").and_then(Value::as_u64)?;
    match fields.get("file") {
        Some(Value::String(node_id)) if fields.len() == 2 => Some(file_handle(id, node_id)),
        None if fields.len() == 1 => Some(snapshot_handle(id, scope.snapshot)),
        Some(_) | None => None,
    }
}

fn file_handle(id: u64, node_id: &str) -> String {
    format!("{{\"file\":{},\"id\":{id}}}", Value::String(node_id.into()))
}

fn snapshot_handle(id: u64, snapshot: Option<u64>) -> String {
    match snapshot {
        Some(snapshot) => format!("{{\"id\":{id},\"snapshot\":{snapshot}}}"),
        None => format!("{{\"id\":{id}}}"),
    }
}

/// Reads a handle that the wire carries as a number but callers may hold as text.
fn handle_number(value: &Value) -> Option<u64> {
    match value {
        Value::Number(number) => number.as_u64(),
        Value::String(text) => text.parse().ok(),
        Value::Null | Value::Bool(_) | Value::Array(_) | Value::Object(_) => None,
    }
}

#[cfg(test)]
#[path = "symbol_identity_tests.rs"]
mod tests;
