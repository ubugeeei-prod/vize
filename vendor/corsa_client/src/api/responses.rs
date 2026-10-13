use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::{NodeHandle, ProjectHandle, SignatureHandle, SymbolHandle, TypeHandle};

/// Response returned by the `initialize` endpoint.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeResponse {
    /// Whether the server treats file names as case sensitive.
    pub use_case_sensitive_file_names: bool,
    /// Current working directory used by the worker.
    pub current_directory: String,
}

/// Parsed `tsconfig` metadata returned by `parseConfigFile`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfigResponse {
    /// Compiler options after Corsa normalization and inheritance resolution.
    pub options: Value,
    /// Files that belong to the parsed config according to Corsa.
    pub file_names: Vec<String>,
}

/// Project descriptor returned by endpoints that resolve a project handle.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectResponse {
    /// Opaque handle used by follow-up project-scoped requests.
    pub id: ProjectHandle,
    /// Absolute or workspace-relative `tsconfig` path that defines the project.
    pub config_file_name: String,
    /// Raw compiler options associated with this project.
    pub compiler_options: Value,
    /// Root files that seed the project graph.
    pub root_files: Vec<String>,
}

/// Symbol metadata returned by Corsa.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SymbolResponse {
    /// Opaque handle used to re-query the symbol later.
    pub id: SymbolHandle,
    /// Symbol display name.
    pub name: String,
    /// TypeScript `SymbolFlags` bitset.
    pub flags: u32,
    /// TypeScript `CheckFlags` bitset for checker-specific symbol state.
    pub check_flags: u32,
    /// Declaration nodes associated with the symbol.
    #[serde(default)]
    pub declarations: Vec<NodeHandle>,
    /// Preferred declaration node when Corsa exposes one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value_declaration: Option<NodeHandle>,
}

/// Type metadata returned by checker-oriented endpoints.
///
/// This structure intentionally mirrors upstream TypeScript concepts closely so
/// advanced consumers can recover rich relationships without re-querying every
/// detail immediately.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TypeResponse {
    /// Opaque handle used to reference this type in later requests.
    pub id: TypeHandle,
    /// TypeScript `TypeFlags` bitset.
    pub flags: u32,
    /// TypeScript `ObjectFlags` bitset when the type is an object type.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_flags: Option<u32>,
    /// Serialized literal-like value payload when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<Value>,
    /// Target type for instantiated or mapped types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<TypeHandle>,
    /// Generic type parameters declared directly on the type.
    #[serde(default)]
    pub type_parameters: Vec<TypeHandle>,
    /// Generic type parameters captured from outer declarations.
    #[serde(default)]
    pub outer_type_parameters: Vec<TypeHandle>,
    /// Type parameters introduced locally while resolving the type.
    #[serde(default)]
    pub local_type_parameters: Vec<TypeHandle>,
    /// Tuple/element flags for tuple-like types.
    #[serde(default)]
    pub element_flags: Vec<u32>,
    /// Fixed tuple length when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fixed_length: Option<i32>,
    /// Whether the type is marked readonly in a tuple/object-like context.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readonly: Option<bool>,
    /// Object type referenced by wrapper types such as indexed accesses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub object_type: Option<TypeHandle>,
    /// Index type referenced by wrapper types such as indexed accesses.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index_type: Option<TypeHandle>,
    /// Checker "check" type associated with conditional or indexed forms.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check_type: Option<TypeHandle>,
    /// Type used on the `extends` side of conditional relationships.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extends_type: Option<TypeHandle>,
    /// Base type for inheritance-like relationships.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_type: Option<TypeHandle>,
    /// Substitution constraint for substituted type variables.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subst_constraint: Option<TypeHandle>,
    /// Human-readable type renderings produced by Corsa.
    ///
    /// Many higher-level integrations can use this directly instead of
    /// round-tripping through another text-rendering endpoint.
    #[serde(default)]
    pub texts: Vec<String>,
    /// Symbol associated with the type when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol: Option<SymbolHandle>,
}

/// Call signature metadata returned by checker-oriented endpoints.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignatureResponse {
    /// Opaque handle used to reference the signature later.
    pub id: SignatureHandle,
    /// TypeScript `SignatureFlags` bitset.
    pub flags: u32,
    /// Declaration node that produced the signature, if exposed.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declaration: Option<NodeHandle>,
    /// Generic type parameters declared on the signature.
    #[serde(default)]
    pub type_parameters: Vec<TypeHandle>,
    /// Parameter symbols in declaration order.
    #[serde(default)]
    pub parameters: Vec<SymbolHandle>,
    /// Full parameter symbol metadata in declaration order when available.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameter_symbols: Vec<SymbolResponse>,
    /// Rendered parameter type texts aligned with [`Self::parameters`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub parameter_type_texts: Vec<Vec<String>>,
    /// `this` parameter symbol when explicitly modeled.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub this_parameter: Option<SymbolHandle>,
    /// Full `this` parameter symbol metadata when available.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub this_parameter_symbol: Option<SymbolResponse>,
    /// Rendered `this` parameter type texts when available.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub this_parameter_type_texts: Vec<String>,
    /// Target signature for instantiated or wrapped signatures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<SignatureHandle>,
    /// Rendered default type argument texts aligned with [`Self::type_parameters`].
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_parameter_default_texts: Vec<String>,
}

/// Type predicate metadata such as `value is Foo`.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TypePredicateResponse {
    /// TypeScript internal predicate kind tag.
    pub kind: i32,
    /// Parameter index targeted by the predicate.
    pub parameter_index: i32,
    /// Parameter name, when the predicate refers to a named parameter.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parameter_name: Option<String>,
    /// Narrowed type described by the predicate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<TypeResponse>,
}

/// Index signature information returned by checker queries.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexInfo {
    /// Type of the index key.
    pub key_type: TypeResponse,
    /// Value type produced by indexing with [`Self::key_type`].
    pub value_type: TypeResponse,
    /// Whether the index signature is readonly.
    #[serde(default)]
    pub is_readonly: bool,
}

/// A single JSDoc tag attached to a symbol.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JsDocTagInfo {
    /// Tag name without the leading `@`, such as `deprecated` or `param`.
    pub name: String,
    /// Rendered tag text, when the tag carries any.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text: String,
}

/// Handles for the checker's intrinsic well-known symbols.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WellKnownSymbolsResponse {
    /// The checker's `unknown` symbol.
    pub unknown: SymbolHandle,
    /// The checker's `undefined` symbol.
    pub undefined: SymbolHandle,
    /// The checker's `arguments` symbol.
    pub arguments: SymbolHandle,
}

/// Program-stored metadata about a single source file.
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFileMetadata {
    /// Whether the file is part of the default library (`lib.*.d.ts`).
    #[serde(default)]
    pub is_default_library: bool,
    /// Whether the file came from an external library such as `node_modules`.
    #[serde(default)]
    pub is_from_external_library: bool,
    /// `type` field of the nearest `package.json`, when one applies.
    #[serde(default)]
    pub package_json_type: String,
    /// Directory of the nearest `package.json`, when one applies.
    #[serde(default)]
    pub package_json_directory: String,
    /// Module resolution mode the program implied for this file.
    #[serde(default)]
    pub implied_node_format: i32,
}
