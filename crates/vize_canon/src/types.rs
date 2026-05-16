//! Type representations for Vue SFC type checking.

use vize_carton::String;

/// Type information for a value or expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeInfo {
    /// Human-readable type representation.
    pub display: String,
    /// Type kind for categorization.
    pub kind: TypeKind,
    /// Documentation if available.
    pub documentation: Option<String>,
}

impl TypeInfo {
    /// Create a new type info.
    pub fn new(display: impl Into<String>, kind: TypeKind) -> Self {
        Self {
            display: display.into(),
            kind,
            documentation: None,
        }
    }

    /// Create an unknown type.
    pub fn unknown() -> Self {
        Self::new("unknown", TypeKind::Unknown)
    }

    /// Create an any type.
    pub fn any() -> Self {
        Self::new("any", TypeKind::Any)
    }

    /// Create a string type.
    pub fn string() -> Self {
        Self::new("string", TypeKind::Primitive)
    }

    /// Create a number type.
    pub fn number() -> Self {
        Self::new("number", TypeKind::Primitive)
    }

    /// Create a boolean type.
    pub fn boolean() -> Self {
        Self::new("boolean", TypeKind::Primitive)
    }

    /// Create a void type.
    pub fn void() -> Self {
        Self::new("void", TypeKind::Void)
    }

    /// Create a null type.
    pub fn null() -> Self {
        Self::new("null", TypeKind::Null)
    }

    /// Create an undefined type.
    pub fn undefined() -> Self {
        Self::new("undefined", TypeKind::Undefined)
    }

    /// Add documentation to the type.
    pub fn with_documentation(mut self, doc: impl Into<String>) -> Self {
        self.documentation = Some(doc.into());
        self
    }

    /// Check if this is an error type (unknown).
    pub fn is_error(&self) -> bool {
        matches!(self.kind, TypeKind::Unknown)
    }
}

/// Kind of type for categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeKind {
    /// Primitive types (string, number, boolean, bigint, symbol).
    Primitive,
    /// Object types.
    Object,
    /// Array types.
    Array,
    /// Function types.
    Function,
    /// Class types.
    Class,
    /// Interface types.
    Interface,
    /// Enum types.
    Enum,
    /// Union types (A | B).
    Union,
    /// Intersection types (A & B).
    Intersection,
    /// Tuple types.
    Tuple,
    /// Literal types (specific values).
    Literal,
    /// Generic type parameters.
    TypeParameter,
    /// Ref<T> from Vue.
    Ref,
    /// Computed<T> from Vue.
    Computed,
    /// Reactive<T> from Vue.
    Reactive,
    /// Component type.
    Component,
    /// Directive type.
    Directive,
    /// Slot type.
    Slot,
    /// Event handler type.
    EventHandler,
    /// Void type.
    Void,
    /// Null type.
    Null,
    /// Undefined type.
    Undefined,
    /// Never type.
    Never,
    /// Any type.
    Any,
    /// Unknown type (error state).
    Unknown,
}

/// Completion item from type analysis.
#[derive(Debug, Clone)]
pub struct CompletionItem {
    /// Label to display.
    pub label: String,
    /// Kind of completion.
    pub kind: CompletionKind,
    /// Detail information.
    pub detail: Option<String>,
    /// Documentation.
    pub documentation: Option<String>,
    /// Text to insert (if different from label).
    pub insert_text: Option<String>,
    /// Sort priority (lower = higher priority).
    pub sort_priority: u32,
}

impl CompletionItem {
    /// Create a new completion item.
    pub fn new(label: impl Into<String>, kind: CompletionKind) -> Self {
        Self {
            label: label.into(),
            kind,
            detail: None,
            documentation: None,
            insert_text: None,
            sort_priority: 100,
        }
    }

    /// Set detail.
    pub fn with_detail(mut self, detail: impl Into<String>) -> Self {
        self.detail = Some(detail.into());
        self
    }

    /// Set documentation.
    pub fn with_documentation(mut self, doc: impl Into<String>) -> Self {
        self.documentation = Some(doc.into());
        self
    }

    /// Set insert text.
    pub fn with_insert_text(mut self, text: impl Into<String>) -> Self {
        self.insert_text = Some(text.into());
        self
    }

    /// Set sort priority.
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.sort_priority = priority;
        self
    }
}

/// Kind of completion item.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompletionKind {
    /// Variable.
    Variable,
    /// Function.
    Function,
    /// Method.
    Method,
    /// Property.
    Property,
    /// Class.
    Class,
    /// Interface.
    Interface,
    /// Enum.
    Enum,
    /// Enum member.
    EnumMember,
    /// Module.
    Module,
    /// Keyword.
    Keyword,
    /// Snippet.
    Snippet,
    /// Type.
    Type,
    /// Constant.
    Constant,
    /// Component.
    Component,
    /// Directive.
    Directive,
    /// Event.
    Event,
    /// Slot.
    Slot,
}

#[cfg(test)]
mod tests {
    use super::{CompletionItem, CompletionKind, TypeInfo, TypeKind};

    #[test]
    fn test_type_info() {
        let t = TypeInfo::string();
        assert_eq!(t.display, "string");
        assert_eq!(t.kind, TypeKind::Primitive);
    }

    #[test]
    fn test_completion_item() {
        let item = CompletionItem::new("count", CompletionKind::Variable)
            .with_detail("number")
            .with_priority(10);
        assert_eq!(item.label, "count");
        assert_eq!(item.detail.as_deref(), Some("number"));
        assert_eq!(item.sort_priority, 10);
    }
}
