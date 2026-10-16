use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// UTF-8 byte range used by Oxlint-compatible rule diagnostics and fixes.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TextRange {
    /// Inclusive byte offset where the range starts.
    pub start: u32,
    /// Exclusive byte offset where the range ends.
    pub end: u32,
}

impl TextRange {
    /// Creates a range from inclusive `start` and exclusive `end` offsets.
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    /// Returns `true` when the range contains no bytes.
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }

    /// Returns `true` when `start` does not exceed `end`.
    pub const fn is_valid(self) -> bool {
        self.start <= self.end
    }
}

/// Serializable AST/type facts passed from the Oxlint JS plugin boundary into
/// Rust-authored rules.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintNode {
    /// ESTree or TypeScript-ESTree node kind.
    pub kind: String,
    /// Source byte range occupied by the node.
    pub range: TextRange,
    /// Optional source text for the node when the host already has it cheaply.
    #[serde(default)]
    pub text: Option<String>,
    /// TypeScript-rendered type texts associated with the node.
    #[serde(default)]
    pub type_texts: Vec<String>,
    /// Property names visible on the node's type.
    #[serde(default)]
    pub property_names: Vec<String>,
    /// Scalar node fields keyed by host-specific field name.
    #[serde(default)]
    pub fields: BTreeMap<String, Value>,
    /// Single child nodes keyed by field name.
    #[serde(default)]
    pub children: BTreeMap<String, LintNode>,
    /// Child node lists keyed by field name.
    #[serde(default)]
    pub child_lists: BTreeMap<String, Vec<LintNode>>,
}

impl LintNode {
    /// Returns a named child node.
    pub fn child(&self, key: &str) -> Option<&Self> {
        self.children.get(key)
    }

    /// Returns a named list of child nodes.
    pub fn child_list(&self, key: &str) -> Option<&[Self]> {
        self.child_lists.get(key).map(Vec::as_slice)
    }

    /// Returns a named scalar field as a string.
    pub fn field_str(&self, key: &str) -> Option<&str> {
        self.fields.get(key).and_then(Value::as_str)
    }

    /// Returns a named scalar field as a boolean.
    pub fn field_bool(&self, key: &str) -> Option<bool> {
        self.fields.get(key).and_then(Value::as_bool)
    }

    /// Returns a named scalar field as an `f64`.
    pub fn field_f64(&self, key: &str) -> Option<f64> {
        self.fields.get(key).and_then(Value::as_f64)
    }

    /// Returns a named scalar field as a string, accepting primitive values.
    pub fn field_stringish(&self, key: &str) -> Option<String> {
        self.fields.get(key).and_then(|value| match value {
            Value::String(value) => Some(value.clone()),
            Value::Number(value) => Some(value.to_string()),
            Value::Bool(value) => Some(value.to_string()),
            _ => None,
        })
    }
}

/// Text edit that can repair or rewrite a lint diagnostic range.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintFix {
    /// Source byte range replaced by this fix.
    pub range: TextRange,
    /// Text inserted in place of [`Self::range`].
    pub replacement_text: String,
}

impl LintFix {
    /// Creates a fix that replaces `range` with `replacement_text`.
    pub fn replace_range(range: TextRange, replacement_text: impl Into<String>) -> Self {
        Self {
            range,
            replacement_text: replacement_text.into(),
        }
    }

    /// Creates a fix that removes `range`.
    pub fn remove_range(range: TextRange) -> Self {
        Self::replace_range(range, "")
    }
}

/// User-facing suggestion attached to a lint diagnostic.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintSuggestion {
    /// Message ID describing the suggested change.
    pub message_id: String,
    /// Rendered message text for the suggestion.
    pub message: String,
    /// Ordered fixes that implement the suggestion.
    pub fixes: Vec<LintFix>,
}

/// Serializable lint diagnostic returned to the host adapter.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LintDiagnostic {
    /// Stable rule name that produced the diagnostic.
    pub rule_name: String,
    /// Stable rule-local message identifier.
    pub message_id: String,
    /// Rendered user-facing diagnostic message.
    pub message: String,
    /// Source byte range reported by the rule.
    pub range: TextRange,
    /// Optional suggestions for automated repair.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub suggestions: Vec<LintSuggestion>,
}

/// Static message catalog entry for a Rust-authored lint rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RuleMessage {
    /// Stable rule-local message identifier.
    pub id: &'static str,
    /// User-facing fallback message text.
    pub description: &'static str,
}

/// Depth range for metadata the JavaScript host should attach to native nodes.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NodeMetadataDepth {
    /// Minimum recursive node depth, where the listener node is depth 0.
    pub min_depth: u8,
    /// Maximum recursive node depth, inclusive.
    pub max_depth: u8,
}

impl NodeMetadataDepth {
    /// Includes metadata at a single depth.
    pub const fn exact(depth: u8) -> Self {
        Self {
            min_depth: depth,
            max_depth: depth,
        }
    }

    /// Includes metadata from the listener node through `max_depth`.
    pub const fn through(max_depth: u8) -> Self {
        Self {
            min_depth: 0,
            max_depth,
        }
    }

    /// Includes metadata for the provided inclusive depth range.
    pub const fn range(min_depth: u8, max_depth: u8) -> Self {
        Self {
            min_depth,
            max_depth,
        }
    }
}

/// Host-side metadata requirements for a Rust-authored lint rule.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleBridgeRequirements {
    /// Maximum AST recursion depth the host should serialize.
    pub max_depth: u8,
    /// Depths where rendered type text should be attached.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_texts: Option<NodeMetadataDepth>,
    /// Depths where property names should be attached.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_names: Option<NodeMetadataDepth>,
    /// Depths where source text should be attached.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub text: Option<NodeMetadataDepth>,
    /// Whether the host should resolve checker symbol facts (for example
    /// deprecation) for the visited node.
    #[serde(skip_serializing_if = "core::ops::Not::not")]
    pub symbol_facts: bool,
}

impl RuleBridgeRequirements {
    /// Default recursion depth used by the JavaScript host bridge.
    pub const DEFAULT_MAX_DEPTH: u8 = 4;

    /// Creates requirements for syntax-only rules.
    pub const fn syntax_only(max_depth: u8) -> Self {
        Self {
            max_depth,
            type_texts: None,
            property_names: None,
            text: None,
            symbol_facts: false,
        }
    }

    /// Creates requirements that attach type text in an inclusive depth range.
    pub const fn type_texts(max_depth: u8, type_texts: NodeMetadataDepth) -> Self {
        Self {
            max_depth,
            type_texts: Some(type_texts),
            property_names: None,
            text: None,
            symbol_facts: false,
        }
    }

    /// Creates requirements that attach type text and property names together.
    pub const fn type_texts_and_properties(max_depth: u8, metadata: NodeMetadataDepth) -> Self {
        Self {
            max_depth,
            type_texts: Some(metadata),
            property_names: Some(metadata),
            text: None,
            symbol_facts: false,
        }
    }

    /// Adds source text metadata to existing requirements.
    pub const fn with_text(mut self, text: NodeMetadataDepth) -> Self {
        self.text = Some(text);
        self
    }

    /// Requests checker symbol facts (for example deprecation) from the host.
    pub const fn with_symbol_facts(mut self) -> Self {
        self.symbol_facts = true;
        self
    }

    /// Creates the default requirements for rules that only declare whether
    /// they need type text.
    pub const fn default_for_type_texts(requires_type_texts: bool) -> Self {
        if requires_type_texts {
            Self::type_texts_and_properties(
                Self::DEFAULT_MAX_DEPTH,
                NodeMetadataDepth::through(Self::DEFAULT_MAX_DEPTH),
            )
        } else {
            Self::syntax_only(Self::DEFAULT_MAX_DEPTH)
        }
    }
}

/// Serializable metadata that describes one Rust-authored lint rule.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuleMeta {
    /// Stable rule name.
    pub name: String,
    /// Short documentation sentence for generated rule docs.
    pub docs_description: String,
    /// Message catalog keyed by message ID.
    pub messages: BTreeMap<String, String>,
    /// Whether the rule can emit suggestions.
    pub has_suggestions: bool,
    /// AST node kinds the rule listens to.
    pub listeners: Vec<String>,
    /// Whether the host should include rendered TypeScript type text.
    pub requires_type_texts: bool,
    /// Native bridge metadata requirements owned by the Rust rule registry.
    pub bridge: RuleBridgeRequirements,
}
