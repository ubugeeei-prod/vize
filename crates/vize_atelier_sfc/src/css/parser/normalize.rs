//! Serialized AST normalization applied before printing.

use serde_json::Value;

/// Drops null `fileType` entries so serialized `image-set()` ASTs deserialize
/// with LightningCSS' optional file-type representation.
pub(crate) fn normalize_image_set_file_types(value: &mut Value) {
    match value {
        Value::Object(map) => {
            if map.get("fileType") == Some(&Value::Null) {
                map.remove("fileType");
            }

            for child in map.values_mut() {
                normalize_image_set_file_types(child);
            }
        }
        Value::Array(items) => {
            for child in items {
                normalize_image_set_file_types(child);
            }
        }
        _ => {}
    }
}
