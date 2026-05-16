//! Shared types for bindings.
//!
//! These types are FFI boundary types used by both NAPI and WASM builds,
//! so they use `std::string::String` for JavaScript interop compatibility.

#![allow(clippy::disallowed_types)]

use serde::{Deserialize, Serialize};

#[cfg(feature = "napi")]
use napi_derive::napi;

/// Compiler options for bindings
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[cfg_attr(feature = "napi", napi(object))]
#[serde(rename_all = "camelCase")]
pub struct CompilerOptions {
    /// Output mode: "module" or "function"
    #[serde(default)]
    pub mode: Option<String>,
    /// Whether to prefix identifiers
    #[serde(default)]
    pub prefix_identifiers: Option<bool>,
    /// Whether to hoist static nodes
    #[serde(default)]
    pub hoist_static: Option<bool>,
    /// Whether to cache event handlers
    #[serde(default)]
    pub cache_handlers: Option<bool>,
    /// Scope ID for scoped CSS
    #[serde(default)]
    pub scope_id: Option<String>,
    /// Whether in SSR mode
    #[serde(default)]
    pub ssr: Option<bool>,
    /// Whether to generate source map
    #[serde(default)]
    pub source_map: Option<bool>,
    /// Filename for source map
    #[serde(default)]
    pub filename: Option<String>,
    /// Output mode: "vdom" or "vapor"
    #[serde(default)]
    pub output_mode: Option<String>,
    /// Whether the template contains TypeScript
    #[serde(default)]
    pub is_ts: Option<bool>,
    /// Whether the template targets a custom renderer instead of the DOM.
    #[serde(default)]
    pub custom_renderer: Option<bool>,
    /// Script extension handling: "preserve" (keep TypeScript) or "downcompile" (transpile to JS)
    /// Defaults to "downcompile"
    #[serde(default)]
    pub script_ext: Option<String>,
}

/// Compile result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "napi", napi(object))]
#[serde(rename_all = "camelCase")]
pub struct CompileResult {
    /// Generated code
    pub code: String,
    /// Preamble code (imports)
    pub preamble: String,
    /// AST (serialized as JSON)
    pub ast: serde_json::Value,
    /// Source map
    #[serde(skip_serializing_if = "Option::is_none")]
    pub map: Option<serde_json::Value>,
    /// Used helpers
    pub helpers: Vec<String>,
    /// Template strings for Vapor mode static parts
    #[serde(skip_serializing_if = "Option::is_none")]
    pub templates: Option<Vec<String>>,
}

/// Compile error
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Source location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loc: Option<SourceLocation>,
}

/// Source location for errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub start: Position,
    pub end: Position,
    pub source: String,
}

/// Position in source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub offset: u32,
    pub line: u32,
    pub column: u32,
}
