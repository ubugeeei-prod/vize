use corsa_core::fast::CompactString;
use serde::{Deserialize, Serialize};

/// File or URI identifier accepted by Corsa API endpoints.
///
/// Most endpoints accept either an on-disk file name or a URI. The enum keeps
/// both forms explicit while still serializing to the wire shape that Corsa
/// expects.
///
/// # Examples
///
/// ```
/// use corsa_client::DocumentIdentifier;
///
/// let file = DocumentIdentifier::from("/workspace/main.ts");
/// assert_eq!(file.as_wire_value(), "/workspace/main.ts");
/// ```
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(untagged)]
pub enum DocumentIdentifier {
    /// Plain file path form used by most filesystem-backed requests.
    FileName(CompactString),
    /// URI form used by LSP-style or virtual-document workflows.
    Uri {
        /// URI string sent to the Corsa endpoint.
        uri: CompactString,
    },
}

impl DocumentIdentifier {
    /// Converts the identifier into the string form used on the wire.
    pub fn as_wire_value(&self) -> CompactString {
        match self {
            Self::FileName(path) => path.clone(),
            Self::Uri { uri } => uri.clone(),
        }
    }
}

impl From<&str> for DocumentIdentifier {
    fn from(value: &str) -> Self {
        Self::FileName(CompactString::from(value))
    }
}

impl From<String> for DocumentIdentifier {
    fn from(value: String) -> Self {
        Self::FileName(CompactString::from(value))
    }
}

/// A UTF-16 position inside a document.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct DocumentPosition {
    /// Document that owns the position.
    pub document: DocumentIdentifier,
    /// Offset expressed in UTF-16 code units, matching TypeScript/LSP APIs.
    pub position: u32,
}
