//! The owned result at the public Vapor compilation boundary.

use vize_carton::String;

/// Vapor compilation result
#[derive(Debug)]
pub struct VaporCompileResult {
    /// Generated code
    pub code: String,
    /// Template strings for static parts
    pub templates: Vec<String>,
    /// Source Map v3 JSON for the generated render code.
    pub map: Option<String>,
    /// Error messages during compilation
    pub error_messages: Vec<String>,
}
