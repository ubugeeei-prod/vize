//! Shared prop type information.

use vize_carton::String;

/// Prop type information
#[derive(Debug, Clone)]
pub struct PropTypeInfo {
    /// JavaScript type constructor name (String, Number, Boolean, Array, Object, Function)
    pub js_type: String,
    /// Original TypeScript type (for PropType<T> usage)
    pub ts_type: Option<String>,
    /// Whether the prop is optional
    pub optional: bool,
    /// Whether the prop accepts null at runtime
    pub nullable: bool,
    /// Whether an unresolvable union member was dropped beside `Boolean` or
    /// `Function`; Vue then emits `skipCheck: true`.
    pub skip_check: bool,
}
