//! One materialized file's projected text, owned by the native payload that
//! carries it to Corsa. Catalog leaves, query surfaces and native batches hold
//! this one allocation instead of a copy each.
#![expect(
    clippy::disallowed_types,
    reason = "immutable payload bytes are shared with native batches"
)]

use std::fmt;
use std::ops::Deref;
use std::path::Path;
use std::sync::Arc;

use vize_carton::String;

/// `(uri, code)`, the exact native document of one materialized path.
#[derive(Clone)]
pub struct MaterializedCode(Arc<(String, String)>);

impl MaterializedCode {
    pub(crate) fn new(materialized_path: &Path, code: String) -> Self {
        Self(Arc::new((
            crate::file_uri::path_to_file_uri(materialized_path),
            code,
        )))
    }

    pub fn as_str(&self) -> &str {
        self.0.1.as_str()
    }

    /// The payload a native batch shares without the leaf's mappings.
    pub(crate) fn native_document(&self) -> &Arc<(String, String)> {
        &self.0
    }
}

impl Deref for MaterializedCode {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for MaterializedCode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq for MaterializedCode {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.as_str() == other.as_str()
    }
}

impl Eq for MaterializedCode {}

impl PartialEq<str> for MaterializedCode {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for MaterializedCode {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for MaterializedCode {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl fmt::Debug for MaterializedCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), formatter)
    }
}

impl serde::Serialize for MaterializedCode {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

impl fmt::Display for MaterializedCode {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_native_payload_is_the_code_allocation() {
        let code = MaterializedCode::new(Path::new("/root/A.vue.ts"), "export {};".into());
        let payload = code.native_document();
        assert_eq!(
            payload.0,
            crate::file_uri::path_to_file_uri(Path::new("/root/A.vue.ts"))
        );
        assert!(std::ptr::eq(payload.1.as_str(), code.as_str()));
        assert_eq!(code, "export {};");
    }
}
