//! Virtual TypeScript code generation for Vue SFC type checking.
//!
//! Generates TypeScript code from Vue SFC components that can be fed
//! to Corsa for type checking. This enables full TypeScript support
//! for template expressions, props, emits, and other Vue features.
//!
//! ## Scope Hierarchy
//!
//! ```text
//! ~mod (module scope)
//!     │
//!     ├── imports (import { ref } from 'vue')
//!     │
//!     └── function __setup<T>() {     // setup scope (function)
//!             │
//!             ├── defineProps         // compiler macros (setup-only, NOT declare)
//!             ├── defineEmits
//!             ├── defineExpose
//!             │
//!             ├── script content      // user's setup code
//!             │
//!             └── function __template() {  // template scope
//!                     │
//!                     └── expressions
//!                 }
//!         }
//! ```
//!
//! ## Key Design Principles
//!
//! 1. **Setup as Function**: The setup scope is expressed as a generic function,
//!    supporting `<script setup generic="T">` syntax.
//!
//! 2. **Scoped Compiler Macros**: `defineProps`, `defineEmits`, etc. are defined
//!    as actual functions (NOT `declare`) inside the setup function, making them
//!    truly scoped and unavailable outside.
//!
//! 3. **Template Inherits Setup**: Template scope is nested inside setup,
//!    with access to all setup bindings.
//!
//! 4. **Uses Croquis ScopeChain**: Leverages the full scope analysis from croquis
//!    including generic parameters, binding types, and scope hierarchy.

mod generator;
mod types;

use std::path::Path;

use vize_carton::profile;
use vize_relief::ast::RootNode;

use crate::analysis::BindingMetadata;
use crate::import_resolver::ImportResolver;
use crate::script_parser::ScriptParseResult;

// Re-exports
pub use generator::VirtualTsGenerator;
pub use types::{
    DiagnosticSeverity, GenerationDiagnostic, ResolvedImport, VirtualTsConfig, VirtualTsOutput,
};

/// Convenience function to generate virtual TypeScript from a full SFC.
pub fn generate_virtual_ts(
    script_content: Option<&str>,
    template_ast: Option<&RootNode>,
    bindings: &BindingMetadata,
    import_resolver: Option<ImportResolver>,
    from_file: Option<&Path>,
    template_offset: u32,
) -> VirtualTsOutput {
    let mut gen = VirtualTsGenerator::new();
    if let Some(resolver) = import_resolver {
        gen = gen.with_import_resolver(resolver);
    }

    // Generate script output first if present
    let script_output = profile!(
        "croquis.virtual_ts.script",
        script_content.map(|s| gen.generate_script_setup(s, bindings, from_file))
    );
    let has_script = script_output.is_some();

    // Generate template output
    let template_output = profile!(
        "croquis.virtual_ts.template",
        template_ast.map(|ast| gen.generate_template(ast, bindings, template_offset, !has_script))
    );

    // Combine outputs
    profile!(
        "croquis.virtual_ts.combine",
        match (script_output, template_output) {
            (Some(mut script), Some(template)) => {
                script.content.push('\n');
                script.content.push_str(&template.content);

                let script_len = script.content.len() as u32;
                for mut mapping in template.source_map.mappings().iter().cloned() {
                    mapping.generated.start += script_len;
                    mapping.generated.end += script_len;
                    script.source_map.add(mapping);
                }

                script.diagnostics.extend(template.diagnostics);
                script
            }
            (Some(script), None) => script,
            (None, Some(template)) => template,
            (None, None) => VirtualTsOutput::default(),
        }
    )
}

/// Generate virtual TypeScript using croquis analysis.
///
/// This is the preferred entry point that uses full scope analysis.
pub fn generate_virtual_ts_with_croquis(
    script_content: &str,
    parse_result: &ScriptParseResult,
    template_ast: Option<&RootNode>,
    config: &VirtualTsConfig,
    import_resolver: Option<ImportResolver>,
    from_file: Option<&Path>,
) -> VirtualTsOutput {
    let mut gen = VirtualTsGenerator::new();
    if let Some(resolver) = import_resolver {
        gen = gen.with_import_resolver(resolver);
    }

    profile!(
        "croquis.virtual_ts.from_croquis",
        gen.generate_from_croquis(
            script_content,
            parse_result,
            template_ast,
            config,
            from_file,
        )
    )
}

#[cfg(test)]
mod tests {
    use super::{VirtualTsConfig, VirtualTsGenerator};
    use crate::analysis::BindingMetadata;
    use crate::script_parser::parse_script_setup;
    use vize_carton::CompactString;
    use vize_relief::BindingType;

    #[test]
    fn test_generate_script_setup() {
        let script = r#"
const msg = ref('Hello');
const count = ref(0);
"#;
        let mut bindings = BindingMetadata::default();
        bindings.add("msg", BindingType::SetupRef);
        bindings.add("count", BindingType::SetupRef);

        let mut gen = VirtualTsGenerator::new();
        let output = gen.generate_script_setup(script, &bindings, None);

        insta::assert_snapshot!(output.content.as_str());
    }

    #[test]
    fn test_generate_with_croquis() {
        let script = r#"
import { ref } from 'vue'
const props = defineProps<{ name: string }>()
const count = ref(0)
"#;
        let parse_result = parse_script_setup(script);
        let config = VirtualTsConfig {
            generic: Some(CompactString::new("T extends string")),
            is_async: false,
            script_offset: 0,
            template_offset: 0,
        };

        let mut gen = VirtualTsGenerator::new();
        let output = gen.generate_from_croquis(script, &parse_result, None, &config, None);

        insta::assert_snapshot!(output.content.as_str());
    }

    #[test]
    fn test_generate_template() {
        let source = r#"<div>{{ message }}</div>"#;
        let allocator = vize_carton::Bump::new();
        let (ast, _) = vize_armature::parse(&allocator, source);

        let mut bindings = BindingMetadata::default();
        bindings.add("message", BindingType::SetupRef);

        let mut gen = VirtualTsGenerator::new();
        let output = gen.generate_template(&ast, &bindings, 0, true);

        assert!(!output.source_map.is_empty());
        insta::assert_snapshot!(output.content.as_str());
    }

    #[test]
    fn test_compiler_macros_are_scoped() {
        let script = r#"
const props = defineProps<{ msg: string }>()
"#;
        let mut bindings = BindingMetadata::default();
        bindings.add("props", BindingType::SetupConst);

        let mut gen = VirtualTsGenerator::new();
        let output = gen.generate_script_setup(script, &bindings, None);

        insta::assert_snapshot!(output.content.as_str());
    }

    #[test]
    fn test_extracts_generic_from_scope_chain() {
        // This test would need a way to set generic in the scope chain
        // For now, we test that the config generic is used
        let script = "const x = 1;";
        let parse_result = parse_script_setup(script);
        let config = VirtualTsConfig {
            generic: Some(CompactString::new("T, U extends T")),
            is_async: true,
            script_offset: 0,
            template_offset: 0,
        };

        let mut gen = VirtualTsGenerator::new();
        let output = gen.generate_from_croquis(script, &parse_result, None, &config, None);

        insta::assert_snapshot!(output.content.as_str());
    }

    // === Snapshot tests ===

    #[test]
    fn test_snapshot_script_setup_output() {
        let script = r#"
import { ref, computed } from 'vue'

const props = defineProps<{
  title: string
  count?: number
}>()

const emit = defineEmits<{
  (e: 'update', value: number): void
  (e: 'close'): void
}>()

const localCount = ref(props.count ?? 0)
const doubled = computed(() => localCount.value * 2)

function increment() {
  localCount.value++
  emit('update', localCount.value)
}
"#;
        let parse_result = parse_script_setup(script);
        let config = VirtualTsConfig::default();

        let mut gen = VirtualTsGenerator::new();
        let output = gen.generate_from_croquis(script, &parse_result, None, &config, None);

        insta::assert_snapshot!(output.content);
    }

    #[test]
    fn test_snapshot_generic_setup() {
        let script = r#"
const props = defineProps<{
  items: T[]
  selected?: T
}>()

const emit = defineEmits<{
  (e: 'select', item: T): void
}>()
"#;
        let parse_result = parse_script_setup(script);
        let config = VirtualTsConfig {
            generic: Some(CompactString::new("T extends { id: string }")),
            is_async: false,
            script_offset: 0,
            template_offset: 0,
        };

        let mut gen = VirtualTsGenerator::new();
        let output = gen.generate_from_croquis(script, &parse_result, None, &config, None);

        insta::assert_snapshot!(output.content);
    }

    #[test]
    fn test_snapshot_async_setup() {
        let script = r#"
const data = await fetchData()
const processed = computed(() => data.map(d => d.name))
"#;
        let parse_result = parse_script_setup(script);
        let config = VirtualTsConfig {
            generic: None,
            is_async: true,
            script_offset: 0,
            template_offset: 0,
        };

        let mut gen = VirtualTsGenerator::new();
        let output = gen.generate_from_croquis(script, &parse_result, None, &config, None);

        insta::assert_snapshot!(output.content);
    }

    #[test]
    fn test_snapshot_template_with_v_for() {
        let template_source = r#"<ul><li v-for="(item, index) in items" :key="item.id">{{ item.name }} - {{ index }}</li></ul>"#;
        let allocator = vize_carton::Bump::new();
        let (ast, _) = vize_armature::parse(&allocator, template_source);

        let mut bindings = BindingMetadata::default();
        bindings.add("items", BindingType::SetupRef);

        let mut gen = VirtualTsGenerator::new();
        let output = gen.generate_template(&ast, &bindings, 0, true);

        insta::assert_snapshot!(output.content);
    }

    #[test]
    fn test_snapshot_template_with_v_if() {
        let template_source = r#"<div><span v-if="isVisible">Visible</span><span v-else-if="isAlternate">Alternate</span><span v-else>Default</span></div>"#;
        let allocator = vize_carton::Bump::new();
        let (ast, _) = vize_armature::parse(&allocator, template_source);

        let mut bindings = BindingMetadata::default();
        bindings.add("isVisible", BindingType::SetupRef);
        bindings.add("isAlternate", BindingType::SetupRef);

        let mut gen = VirtualTsGenerator::new();
        let output = gen.generate_template(&ast, &bindings, 0, true);

        insta::assert_snapshot!(output.content);
    }

    #[test]
    fn test_snapshot_full_sfc() {
        let script = r#"
import { ref } from 'vue'
const count = ref(0)
const increment = () => count.value++
"#;
        let template_source = r#"<button @click="increment">{{ count }}</button>"#;

        let parse_result = parse_script_setup(script);
        let allocator = vize_carton::Bump::new();
        let (template_ast, _) = vize_armature::parse(&allocator, template_source);

        let config = VirtualTsConfig {
            script_offset: 0,
            template_offset: 100,
            ..Default::default()
        };

        let mut gen = VirtualTsGenerator::new();
        let output =
            gen.generate_from_croquis(script, &parse_result, Some(&template_ast), &config, None);

        insta::assert_snapshot!(output.content);
    }
}
