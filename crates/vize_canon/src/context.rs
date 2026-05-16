//! Type context for Vue SFC type checking.
//!
//! The TypeContext holds all the type information extracted from
//! the script block that is available in the template.

use vize_carton::FxHashMap;
use vize_carton::String;

use crate::types::TypeInfo;

/// Type context for a Vue SFC.
///
/// Contains all type information needed to type-check the template.
#[derive(Debug, Clone, Default)]
pub struct TypeContext {
    /// Bindings available in the template (from script setup or setup()).
    pub bindings: FxHashMap<String, Binding>,
    /// Imported components.
    pub components: FxHashMap<String, ComponentInfo>,
    /// Props defined in the component.
    pub props: Vec<Prop>,
    /// Emits defined in the component.
    pub emits: Vec<Emit>,
    /// Slots defined in the component.
    pub slots: Vec<Slot>,
    /// Global properties (e.g., $router, $store).
    pub globals: FxHashMap<String, TypeInfo>,
}

impl TypeContext {
    /// Create a new empty type context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a binding to the context.
    pub fn add_binding(&mut self, name: impl Into<String>, binding: Binding) {
        self.bindings.insert(name.into(), binding);
    }

    /// Get a binding by name.
    pub fn get_binding(&self, name: &str) -> Option<&Binding> {
        self.bindings.get(name)
    }

    /// Add a component to the context.
    pub fn add_component(&mut self, name: impl Into<String>, info: ComponentInfo) {
        self.components.insert(name.into(), info);
    }

    /// Get a component by name.
    pub fn get_component(&self, name: &str) -> Option<&ComponentInfo> {
        self.components.get(name)
    }

    /// Add a prop definition.
    pub fn add_prop(&mut self, prop: Prop) {
        self.props.push(prop);
    }

    /// Add an emit definition.
    pub fn add_emit(&mut self, emit: Emit) {
        self.emits.push(emit);
    }

    /// Add a slot definition.
    pub fn add_slot(&mut self, slot: Slot) {
        self.slots.push(slot);
    }

    /// Check if a binding exists.
    pub fn has_binding(&self, name: &str) -> bool {
        self.bindings.contains_key(name)
    }

    /// Check if a component is registered.
    pub fn has_component(&self, name: &str) -> bool {
        self.components.contains_key(name)
    }
}

/// A binding available in the template.
#[derive(Debug, Clone)]
pub struct Binding {
    /// Name of the binding.
    pub name: String,
    /// Type information.
    pub type_info: TypeInfo,
    /// Kind of binding.
    pub kind: BindingKind,
    /// Source location (byte offset) in script.
    pub source_offset: Option<u32>,
}

impl Binding {
    /// Create a new binding.
    pub fn new(name: impl Into<String>, type_info: TypeInfo, kind: BindingKind) -> Self {
        Self {
            name: name.into(),
            type_info,
            kind,
            source_offset: None,
        }
    }

    /// Set the source offset.
    pub fn with_offset(mut self, offset: u32) -> Self {
        self.source_offset = Some(offset);
        self
    }
}

/// Kind of binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BindingKind {
    /// const declaration.
    Const,
    /// let declaration.
    Let,
    /// var declaration (legacy).
    Var,
    /// Function declaration.
    Function,
    /// Class declaration.
    Class,
    /// Import binding.
    Import,
    /// Destructured binding.
    Destructure,
    /// Ref from Vue.
    Ref,
    /// Computed from Vue.
    Computed,
    /// Reactive from Vue.
    Reactive,
    /// Prop binding.
    Prop,
    /// Setup return value.
    SetupReturn,
}

/// Information about an imported component.
#[derive(Debug, Clone)]
pub struct ComponentInfo {
    /// Component name.
    pub name: String,
    /// Import path.
    pub import_path: Option<String>,
    /// Props the component accepts.
    pub props: Vec<Prop>,
    /// Events the component emits.
    pub emits: Vec<Emit>,
    /// Slots the component provides.
    pub slots: Vec<Slot>,
    /// Whether this is a built-in component.
    pub is_builtin: bool,
}

impl ComponentInfo {
    /// Create a new component info.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            import_path: None,
            props: Vec::new(),
            emits: Vec::new(),
            slots: Vec::new(),
            is_builtin: false,
        }
    }

    /// Set the import path.
    pub fn with_import_path(mut self, path: impl Into<String>) -> Self {
        self.import_path = Some(path.into());
        self
    }

    /// Mark as built-in.
    pub fn builtin(mut self) -> Self {
        self.is_builtin = true;
        self
    }
}

/// Import information.
#[derive(Debug, Clone)]
pub struct Import {
    /// Local name.
    pub local: String,
    /// Imported name (may differ from local for renamed imports).
    pub imported: String,
    /// Module path.
    pub module: String,
    /// Type information for the import.
    pub type_info: TypeInfo,
    /// Whether this is a type-only import.
    pub is_type_only: bool,
}

impl Import {
    /// Create a new import.
    pub fn new(local: impl Into<String>, module: impl Into<String>) -> Self {
        let local = local.into();
        Self {
            imported: local.clone(),
            local,
            module: module.into(),
            type_info: TypeInfo::unknown(),
            is_type_only: false,
        }
    }

    /// Set the imported name (for renamed imports).
    pub fn with_imported(mut self, imported: impl Into<String>) -> Self {
        self.imported = imported.into();
        self
    }

    /// Set type information.
    pub fn with_type(mut self, type_info: TypeInfo) -> Self {
        self.type_info = type_info;
        self
    }

    /// Mark as type-only import.
    pub fn type_only(mut self) -> Self {
        self.is_type_only = true;
        self
    }
}

/// A prop definition.
#[derive(Debug, Clone)]
pub struct Prop {
    /// Prop name.
    pub name: String,
    /// Type information.
    pub type_info: TypeInfo,
    /// Whether the prop is required.
    pub required: bool,
    /// Default value expression (as string).
    pub default: Option<String>,
    /// Validator expression (as string).
    pub validator: Option<String>,
}

impl Prop {
    /// Create a new prop.
    pub fn new(name: impl Into<String>, type_info: TypeInfo) -> Self {
        Self {
            name: name.into(),
            type_info,
            required: false,
            default: None,
            validator: None,
        }
    }

    /// Mark as required.
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Set default value.
    pub fn with_default(mut self, default: impl Into<String>) -> Self {
        self.default = Some(default.into());
        self
    }
}

/// An emit definition.
#[derive(Debug, Clone)]
pub struct Emit {
    /// Event name.
    pub name: String,
    /// Payload type information.
    pub payload_type: Option<TypeInfo>,
}

impl Emit {
    /// Create a new emit.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            payload_type: None,
        }
    }

    /// Set payload type.
    pub fn with_payload(mut self, type_info: TypeInfo) -> Self {
        self.payload_type = Some(type_info);
        self
    }
}

/// A slot definition.
#[derive(Debug, Clone)]
pub struct Slot {
    /// Slot name (empty string for default slot).
    pub name: String,
    /// Slot props type.
    pub props_type: Option<TypeInfo>,
}

impl Slot {
    /// Create a new slot.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            props_type: None,
        }
    }

    /// Create the default slot.
    pub fn default_slot() -> Self {
        Self::new("")
    }

    /// Set props type.
    pub fn with_props(mut self, type_info: TypeInfo) -> Self {
        self.props_type = Some(type_info);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::{Binding, BindingKind, Prop, TypeContext};
    use crate::types::{TypeInfo, TypeKind};

    #[test]
    fn test_type_context() {
        let mut ctx = TypeContext::new();

        ctx.add_binding(
            "count",
            Binding::new(
                "count",
                TypeInfo::new("Ref<number>", TypeKind::Ref),
                BindingKind::Ref,
            ),
        );

        assert!(ctx.has_binding("count"));
        assert!(!ctx.has_binding("unknown"));
    }

    #[test]
    fn test_binding() {
        let binding = Binding::new("foo", TypeInfo::string(), BindingKind::Const).with_offset(100);

        assert_eq!(binding.name, "foo");
        assert_eq!(binding.source_offset, Some(100));
    }

    #[test]
    fn test_prop() {
        let prop = Prop::new("message", TypeInfo::string())
            .required()
            .with_default("\"hello\"");

        assert!(prop.required);
        assert_eq!(prop.default.as_deref(), Some("\"hello\""));
    }
}
