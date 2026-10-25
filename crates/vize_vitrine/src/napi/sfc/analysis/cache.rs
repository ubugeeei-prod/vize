//! Bounded source-identity cache. No filesystem-dependent facts are retained.

use serde_json::Value;
use std::collections::VecDeque;
use std::sync::Arc;

const MAX_ENTRIES: usize = 64;
const MAX_BYTES: usize = 4 * 1024 * 1024;

struct Entry {
    filename: String,
    source: String,
    value: Arc<Value>,
    bytes: usize,
}

#[derive(Default)]
pub(super) struct AnalysisCache {
    entries: VecDeque<Entry>,
    bytes: usize,
}

impl AnalysisCache {
    pub fn get(&mut self, filename: &str, source: &str) -> Option<Arc<Value>> {
        let index = self
            .entries
            .iter()
            .position(|entry| entry.filename == filename && entry.source == source)?;
        let entry = self.entries.remove(index)?;
        let value = Arc::clone(&entry.value);
        self.entries.push_back(entry);
        Some(value)
    }

    pub fn insert(&mut self, filename: &str, source: &str, value: Value) {
        // A changed document supersedes its old source, including an oversized
        // edit which is deliberately not retained.
        self.entries.retain(|entry| {
            if entry.filename == filename {
                self.bytes = self.bytes.saturating_sub(entry.bytes);
                false
            } else {
                true
            }
        });
        let bytes = source
            .len()
            .saturating_add(filename.len())
            .saturating_add(value_bytes(&value));
        if bytes > MAX_BYTES {
            return;
        }
        while self.entries.len() >= MAX_ENTRIES || self.bytes.saturating_add(bytes) > MAX_BYTES {
            let Some(entry) = self.entries.pop_front() else {
                break;
            };
            self.bytes = self.bytes.saturating_sub(entry.bytes);
        }
        self.bytes += bytes;
        self.entries.push_back(Entry {
            filename: filename.to_owned(),
            source: source.to_owned(),
            value: Arc::new(value),
            bytes,
        });
    }
}

// Count retained payload storage conservatively, including container slots and
// object tree overhead. This avoids a second JSON serialization for accounting.
fn value_bytes(value: &Value) -> usize {
    let storage = match value {
        Value::String(value) => value.len(),
        Value::Array(values) => values.iter().map(value_bytes).sum(),
        Value::Object(values) => values
            .iter()
            .map(|(key, value)| key.len() + 128 + value_bytes(value))
            .sum(),
        _ => 0,
    };
    storage.saturating_add(std::mem::size_of::<Value>())
}

#[cfg(test)]
mod tests {
    use super::{AnalysisCache, MAX_BYTES, MAX_ENTRIES};
    use serde_json::json;

    #[test]
    fn exact_source_filename_lru_and_oversized_edits_are_bounded() {
        let mut cache = AnalysisCache::default();
        cache.insert("a.vue", "one", json!({"props":[]}));
        assert!(cache.get("a.vue", "one").is_some());
        assert!(cache.get("b.vue", "one").is_none());
        assert!(cache.get("a.vue", "two").is_none());
        cache.insert("a.vue", "two", json!({"props":[]}));
        assert!(cache.get("a.vue", "one").is_none());
        for index in 0..MAX_ENTRIES {
            cache.insert(&format!("{index}.vue"), "s", json!({}));
        }
        assert!(cache.get("a.vue", "two").is_none());
        assert_eq!(cache.entries.len(), MAX_ENTRIES);
        cache.insert("0.vue", &"x".repeat(MAX_BYTES), json!({}));
        assert!(cache.get("0.vue", "s").is_none());
        assert!(cache.bytes <= MAX_BYTES);
    }
}
