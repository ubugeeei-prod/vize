use serde::{Deserialize, Serialize};
use serde_json::Value;
use vize_carton::{FxHashMap, String};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignToken {
    pub value: Value,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attributes: Option<Value>,
    #[serde(rename = "$tier", skip_serializing_if = "Option::is_none")]
    pub tier: Option<String>,
    #[serde(rename = "$reference", skip_serializing_if = "Option::is_none")]
    pub reference: Option<String>,
    #[serde(rename = "$resolvedValue", skip_serializing_if = "Option::is_none")]
    pub resolved_value: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TokenCategory {
    pub name: String,
    #[serde(default)]
    pub tokens: FxHashMap<String, DesignToken>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub subcategories: Vec<TokenCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FlattenedToken {
    pub name: String,
    pub path: String,
    pub category_path: Vec<String>,
    pub value: Value,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub token_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResolvedTokens {
    pub categories: Vec<TokenCategory>,
    pub token_map: FxHashMap<String, DesignToken>,
    pub token_count: u32,
    pub primitive_count: u32,
    pub semantic_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationResult {
    pub valid: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum TokenError {
    #[error("Failed to read token path: {message}")]
    Io { message: String },
    #[error("Invalid token JSON: {message}")]
    Json { message: String },
    #[error("Expected token categories JSON array")]
    ExpectedCategories,
}

pub type TokenResult<T> = Result<T, TokenError>;
