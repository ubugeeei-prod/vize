//! Script compile context.
//!
//! Holds all state during script compilation.
//! Uses OXC for proper AST-based parsing instead of regex.

mod external_types;
mod helpers;
mod parse;
mod props;

use crate::types::{BindingMetadata, BindingType};
use vize_carton::{CompactString, String, ToCompactString};
use vize_croquis::analysis::Croquis;
use vize_croquis::macros::{EmitDefinition, ModelDefinition, PropDefinition};

use super::ScriptSetupMacros;

/// Script compile context - holds all state during compilation
#[derive(Debug)]
pub struct ScriptCompileContext {
    /// Source content
    pub source: String,

    /// Binding metadata
    pub bindings: BindingMetadata,

    /// Extracted macros
    pub macros: ScriptSetupMacros,

    /// Whether defineProps was called
    pub has_define_props_call: bool,

    /// Whether defineEmits was called
    pub has_define_emits_call: bool,

    /// Whether defineExpose was called
    pub has_define_expose_call: bool,

    /// Whether defineOptions was called
    pub has_define_options_call: bool,

    /// Whether defineSlots was called
    pub has_define_slots_call: bool,

    /// Whether defineModel was called
    pub has_define_model_call: bool,

    // --- Emits related fields ---
    /// Runtime declaration for emits (the argument passed to defineEmits)
    pub emits_runtime_decl: Option<String>,

    /// Type declaration for emits (the type parameter)
    pub emits_type_decl: Option<String>,

    /// The variable name emits is assigned to (e.g., "emit")
    pub emit_decl_id: Option<String>,

    /// TypeScript interface definitions (name -> body)
    /// Used to resolve type references in defineProps<InterfaceName>()
    pub interfaces: vize_carton::FxHashMap<String, String>,

    /// TypeScript type alias definitions (name -> body)
    /// Used to resolve type references in defineProps<TypeName>()
    pub type_aliases: vize_carton::FxHashMap<String, String>,
}

impl ScriptCompileContext {
    /// Create a new context
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_compact_string(),
            bindings: BindingMetadata::default(),
            macros: ScriptSetupMacros::default(),
            has_define_props_call: false,
            has_define_emits_call: false,
            has_define_expose_call: false,
            has_define_options_call: false,
            has_define_slots_call: false,
            has_define_model_call: false,
            emits_runtime_decl: None,
            emits_type_decl: None,
            emit_decl_id: None,
            interfaces: vize_carton::FxHashMap::default(),
            type_aliases: vize_carton::FxHashMap::default(),
        }
    }

    /// Analyze script setup and extract bindings
    pub fn analyze(&mut self) {
        // Temporarily take ownership of source to avoid borrow conflicts
        let source = std::mem::take(&mut self.source);
        self.parse_with_oxc(&source);
        self.source = source;
        // ScriptCompileContext is always used for <script setup>
        self.bindings.is_script_setup = true;
    }

    /// Convert to an Croquis for use in transforms and linting.
    ///
    /// This bridges the atelier script context to the shared croquis analysis format.
    pub fn to_analysis_summary(&self) -> Croquis {
        let mut summary = Croquis::new();

        // Convert bindings
        summary.bindings.is_script_setup = true;
        for (name, binding_type) in &self.bindings.bindings {
            summary.bindings.add(name.as_str(), *binding_type);
        }

        // Convert props aliases
        for (local, key) in &self.bindings.props_aliases {
            summary
                .bindings
                .props_aliases
                .insert(CompactString::new(local), CompactString::new(key));
        }

        // Convert props from macros
        if let Some(ref props_call) = self.macros.define_props {
            for (name, binding_type) in &self.bindings.bindings {
                if matches!(binding_type, BindingType::Props) {
                    summary.macros.add_prop(PropDefinition {
                        name: CompactString::new(name),
                        required: false, // We don't track this in the current implementation
                        prop_type: None,
                        default_value: props_call.binding_name.clone().map(CompactString::new),
                    });
                }
            }
        }

        // Convert emits
        if let Some(ref emits_call) = self.macros.define_emits {
            // Parse emits from the macro call args if available
            let trimmed = emits_call.args.trim();
            if trimmed.starts_with('[') && trimmed.ends_with(']') {
                // Array syntax: ['click', 'update']
                let inner = &trimmed[1..trimmed.len() - 1];
                for part in inner.split(',') {
                    let part = part.trim();
                    if (part.starts_with('\'') && part.ends_with('\''))
                        || (part.starts_with('"') && part.ends_with('"'))
                    {
                        let name = &part[1..part.len() - 1];
                        summary.macros.add_emit(EmitDefinition {
                            name: CompactString::new(name),
                            payload_type: None,
                        });
                    }
                }
            }
        }

        // Convert models
        for model_call in &self.macros.define_models {
            if let Some(ref binding_name) = model_call.binding_name {
                // Extract model name from args if present
                let args = model_call.args.trim();
                let name = if args.starts_with('\'') || args.starts_with('"') {
                    let quote = args.as_bytes()[0];
                    if let Some(end) = args[1..].find(|c: char| c as u8 == quote) {
                        CompactString::new(&args[1..=end])
                    } else {
                        CompactString::new("modelValue")
                    }
                } else {
                    CompactString::new("modelValue")
                };

                summary.macros.add_model(ModelDefinition {
                    name: name.clone(),
                    local_name: CompactString::new(binding_name),
                    model_type: None,
                    required: false,
                    default_value: None,
                });
            }
        }

        summary
    }

    /// Extract all macros from the source
    pub fn extract_all_macros(&mut self) {
        let source = std::mem::take(&mut self.source);
        self.parse_with_oxc(&source);
        self.source = source;
    }
}

#[cfg(test)]
mod tests {
    use super::ScriptCompileContext;
    use crate::types::BindingType;
    use vize_carton::ToCompactString;

    #[test]
    fn test_context_analyze() {
        let content = r#"
const msg = ref('hello')
const count = ref(0)
let name = 'world'
const double = computed(() => count.value * 2)
function increment() { count.value++ }
"#;
        let mut ctx = ScriptCompileContext::new(content);
        ctx.analyze();

        assert_eq!(
            ctx.bindings.bindings.get("msg"),
            Some(&BindingType::SetupRef)
        );
        assert_eq!(
            ctx.bindings.bindings.get("count"),
            Some(&BindingType::SetupRef)
        );
        assert_eq!(
            ctx.bindings.bindings.get("name"),
            Some(&BindingType::SetupLet)
        );
        assert_eq!(
            ctx.bindings.bindings.get("increment"),
            Some(&BindingType::SetupConst)
        );
    }

    #[test]
    fn test_extract_define_props_typed() {
        let content = r#"const props = defineProps<{ msg: string }>()"#;
        let mut ctx = ScriptCompileContext::new(content);
        ctx.analyze();

        assert!(ctx.has_define_props_call);
        assert!(ctx.macros.define_props.is_some());
        let props_call = ctx.macros.define_props.unwrap();
        assert_eq!(
            props_call.type_args,
            Some("{ msg: string }".to_compact_string())
        );
    }

    #[test]
    fn test_extract_define_emits_typed() {
        let content = r#"const emit = defineEmits<{ (e: 'click'): void }>()"#;
        let mut ctx = ScriptCompileContext::new(content);
        ctx.analyze();

        assert!(ctx.has_define_emits_call);
        assert!(ctx.macros.define_emits.is_some());
    }

    #[test]
    fn test_extract_with_defaults() {
        let content =
            r#"const props = withDefaults(defineProps<{ msg?: string }>(), { msg: 'hello' })"#;
        let mut ctx = ScriptCompileContext::new(content);
        ctx.analyze();

        assert!(ctx.has_define_props_call);
        assert!(ctx.macros.with_defaults.is_some());
    }

    #[test]
    fn test_props_destructure() {
        let content = r#"const { foo, bar } = defineProps<{ foo: string, bar: number }>()"#;
        let mut ctx = ScriptCompileContext::new(content);
        ctx.analyze();

        assert!(ctx.macros.props_destructure.is_some());
        let destructure = ctx.macros.props_destructure.as_ref().unwrap();
        assert_eq!(destructure.bindings.len(), 2);
        assert!(destructure.bindings.contains_key("foo"));
        assert!(destructure.bindings.contains_key("bar"));
    }

    #[test]
    fn test_props_destructure_with_alias() {
        let content =
            r#"const { foo: myFoo, bar = 123 } = defineProps<{ foo: string, bar?: number }>()"#;
        let mut ctx = ScriptCompileContext::new(content);
        ctx.analyze();

        assert!(ctx.macros.props_destructure.is_some());
        let destructure = ctx.macros.props_destructure.as_ref().unwrap();

        // Check that bindings use the key as the map key
        assert!(destructure.bindings.contains_key("foo"));
        assert!(destructure.bindings.contains_key("bar"));

        // Check local names
        assert_eq!(destructure.bindings.get("foo").unwrap().local, "myFoo");
        assert_eq!(destructure.bindings.get("bar").unwrap().local, "bar");

        // Check default value
        assert!(destructure.bindings.get("bar").unwrap().default.is_some());
    }

    #[test]
    fn test_define_props_with_interface_reference() {
        let content = r#"
interface Props {
    msg: string
    count?: number
}
const props = defineProps<Props>()
"#;
        let mut ctx = ScriptCompileContext::new(content);
        ctx.analyze();

        // Check interface was captured
        assert!(ctx.interfaces.contains_key("Props"));

        // Check props were extracted from interface
        assert!(ctx.has_define_props_call);
        assert_eq!(ctx.bindings.bindings.get("msg"), Some(&BindingType::Props));
        assert_eq!(
            ctx.bindings.bindings.get("count"),
            Some(&BindingType::Props)
        );
    }

    #[test]
    fn test_define_props_with_type_alias_reference() {
        let content = r#"
type Props = {
    foo: string
    bar: number
}
const props = defineProps<Props>()
"#;
        let mut ctx = ScriptCompileContext::new(content);
        ctx.analyze();

        // Check type alias was captured
        assert!(ctx.type_aliases.contains_key("Props"));

        // Check props were extracted from type alias
        assert!(ctx.has_define_props_call);
        assert_eq!(ctx.bindings.bindings.get("foo"), Some(&BindingType::Props));
        assert_eq!(ctx.bindings.bindings.get("bar"), Some(&BindingType::Props));
    }

    #[test]
    fn test_define_props_with_exported_type_alias() {
        let content = r#"
export type MenuItemProps = {
    id: string
    label: string
    routeName: string
    disabled?: boolean
}
const { label, disabled, routeName } = defineProps<MenuItemProps>()
"#;
        let mut ctx = ScriptCompileContext::new(content);
        ctx.analyze();

        // Check exported type alias was captured
        assert!(
            ctx.type_aliases.contains_key("MenuItemProps"),
            "export type alias should be collected"
        );

        // Check props were extracted from exported type alias
        assert!(ctx.has_define_props_call);
        assert_eq!(
            ctx.bindings.bindings.get("label"),
            Some(&BindingType::Props)
        );
        assert_eq!(
            ctx.bindings.bindings.get("disabled"),
            Some(&BindingType::Props)
        );
        assert_eq!(
            ctx.bindings.bindings.get("routeName"),
            Some(&BindingType::Props)
        );
    }

    #[test]
    fn test_define_props_with_exported_interface() {
        let content = r#"
export interface Props {
    msg: string
    count?: number
}
const props = defineProps<Props>()
"#;
        let mut ctx = ScriptCompileContext::new(content);
        ctx.analyze();

        // Check exported interface was captured
        assert!(
            ctx.interfaces.contains_key("Props"),
            "export interface should be collected"
        );

        // Check props were extracted from exported interface
        assert!(ctx.has_define_props_call);
        assert_eq!(ctx.bindings.bindings.get("msg"), Some(&BindingType::Props));
        assert_eq!(
            ctx.bindings.bindings.get("count"),
            Some(&BindingType::Props)
        );
    }

    #[test]
    fn test_with_defaults_with_interface() {
        let content = r#"
interface Props {
    msg?: string
    count?: number
}
const props = withDefaults(defineProps<Props>(), {
    msg: 'hello',
    count: 0
})
"#;
        let mut ctx = ScriptCompileContext::new(content);
        ctx.analyze();

        assert!(ctx.has_define_props_call);
        assert!(ctx.macros.with_defaults.is_some());
        assert_eq!(ctx.bindings.bindings.get("msg"), Some(&BindingType::Props));
        assert_eq!(
            ctx.bindings.bindings.get("count"),
            Some(&BindingType::Props)
        );
    }
}
