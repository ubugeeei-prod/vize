//! Session-owned evidence for the latest completion response of each live document.

use serde_json::Value;
use vize_l0::{FxHashMap, String};

#[derive(Default)]
pub(in crate::lsp_client::editor_lsp) struct IssuedCompletions {
    generation: Option<u64>,
    documents: FxHashMap<String, Vec<Value>>,
}

impl IssuedCompletions {
    pub(in crate::lsp_client::editor_lsp) fn invalidate(&mut self) {
        self.documents.clear();
        self.generation = None;
    }

    pub(super) fn record(&mut self, generation: u64, uri: &str, response: Option<&Value>) {
        if self.generation != Some(generation) {
            self.invalidate();
            self.generation = Some(generation);
        }
        self.documents.remove(uri);
        let Some(response) = response else { return };
        let (items, defaults) = match response {
            Value::Array(items) => (items, None),
            Value::Object(list) => {
                let Some(items) = list.get("items").and_then(Value::as_array) else {
                    return;
                };
                (items, list.get("itemDefaults").and_then(Value::as_object))
            }
            _ => return,
        };
        let issued = items
            .iter()
            .filter_map(|raw| {
                let mut item = raw.as_object()?.clone();
                // Match Maestro's existing defaults materialization; leave the response untouched.
                if let Some(defaults) = defaults {
                    for key in ["data", "insertTextFormat"] {
                        if let Some(value) = defaults.get(key) {
                            item.entry(key).or_insert_with(|| value.clone());
                        }
                    }
                }
                item.get("data").filter(|value| !value.is_null())?;
                Some(Value::Object(item))
            })
            .collect::<Vec<_>>();
        if !issued.is_empty() {
            self.documents.insert(uri.into(), issued);
        }
    }

    pub(super) fn allows(&self, generation: u64, uri: &str, item: &Value) -> bool {
        self.generation == Some(generation)
            && self
                .documents
                .get(uri)
                .is_some_and(|items| items.contains(item))
    }
}

#[cfg(test)]
mod tests;
