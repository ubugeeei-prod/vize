//! script/no-import-compiler-macros
//!
//! Disallow importing Vue compiler macros that are auto-imported.
//!
//! Vue compiler macros like `defineProps`, `defineEmits`, `defineExpose`,
//! `defineOptions`, `defineSlots`, and `withDefaults` are automatically
//! available in `<script setup>` and should not be explicitly imported.
//!
//! ## Examples
//!
//! ### Invalid
//! ```ts
//! import { defineProps, defineEmits } from 'vue'
//! import { withDefaults } from 'vue'
//!
//! const props = defineProps<Props>()
//! ```
//!
//! ### Valid
//! ```ts
//! // No import needed - compiler macros are auto-imported
//! const props = defineProps<Props>()
//! const emit = defineEmits<Emits>()
//!
//! // Regular imports are fine
//! import { ref, computed } from 'vue'
//! ```

use oxc_ast::ast::{ImportDeclarationSpecifier, Program, Statement};
use oxc_span::GetSpan;
use vize_s0::cstr;

use vize_croquis::COMPILER_MACRO_NAMES;

use crate::diagnostic::{LintDiagnostic, Severity};

use super::{ScriptLintResult, ScriptRule, ScriptRuleMeta};

static META: ScriptRuleMeta = ScriptRuleMeta {
    name: "script/no-import-compiler-macros",
    description: "Disallow importing Vue compiler macros that are auto-imported",
    default_severity: Severity::Error,
};

/// No import compiler macros rule
pub struct NoImportCompilerMacros;

impl ScriptRule for NoImportCompilerMacros {
    fn meta(&self) -> &'static ScriptRuleMeta {
        &META
    }

    fn uses_ast(&self) -> bool {
        true
    }

    fn check_program<'a>(
        &self,
        program: &'a Program<'a>,
        _source: &str,
        offset: usize,
        result: &mut ScriptLintResult,
    ) {
        for statement in &program.body {
            let Statement::ImportDeclaration(import) = statement else {
                continue;
            };
            if import.source.value != "vue" {
                continue;
            }
            for specifier in import.specifiers.iter().flatten() {
                let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier else {
                    continue;
                };
                let name = specifier.imported.name();
                if !COMPILER_MACRO_NAMES.contains(&name.as_str()) {
                    continue;
                }
                let span = specifier.imported.span();
                result.add_diagnostic(
                    LintDiagnostic::error(
                        META.name,
                        cstr!("Do not import '{}' - compiler macros are automatically available in <script setup>", name),
                        offset as u32 + span.start,
                        offset as u32 + span.end,
                    ).with_help("Remove the macro from the import statement. Compiler macros are auto-imported."),
                );
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::NoImportCompilerMacros;
    use crate::rules::script::ScriptLinter;

    fn create_linter() -> ScriptLinter {
        let mut linter = ScriptLinter::new();
        linter.add_rule(Box::new(NoImportCompilerMacros));
        linter
    }

    #[test]
    fn test_valid_no_compiler_macros() {
        let linter = create_linter();
        let result = linter.lint("import { ref, computed } from 'vue'", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_import_use_template_ref() {
        let linter = create_linter();
        let result = linter.lint("import { useTemplateRef } from 'vue'", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_invalid_import_define_props() {
        let linter = create_linter();
        let result = linter.lint("import { defineProps } from 'vue'", 0);
        assert_eq!(result.error_count, 1);
        insta::assert_debug_snapshot!(result.diagnostics);
    }

    #[test]
    fn test_invalid_import_multiple_macros() {
        let linter = create_linter();
        let result = linter.lint("import { defineProps, defineEmits } from 'vue'", 0);
        assert_eq!(result.error_count, 2);
    }

    #[test]
    fn test_valid_usage_without_import() {
        let linter = create_linter();
        let result = linter.lint("const props = defineProps<Props>()", 0);
        assert_eq!(result.error_count, 0);
    }

    #[test]
    fn test_valid_other_package() {
        let linter = create_linter();
        let result = linter.lint("import { defineProps } from 'other-package'", 0);
        assert_eq!(result.error_count, 0);
    }
    #[test]
    fn imports_are_parsed_instead_of_matching_comments_strings_or_aliases() {
        let linter = create_linter();
        for source in [
            "// import { defineProps } from 'vue'",
            "const text = \"import { defineProps } from 'vue'\";",
            "import { computed as defineProps } from 'vue';",
            "import { definePropsHelper } from 'vue';",
            "import { computed /* defineProps */ } from 'vue';",
            "import { defineProps } from 'elsewhere'; import { computed } from 'vue';",
        ] {
            assert_eq!(linter.lint(source, 0).error_count, 0, "{source}");
        }
        let source = "const emoji = '😀';\r\nimport {\r\n  computed,\r\n  defineProps as props,\r\n  defineEmits\r\n} from 'vue';";
        let result = linter.lint(source, 7);
        assert_eq!(result.error_count, 2);
        let spans: Vec<_> = result
            .diagnostics
            .iter()
            .map(|diagnostic| (diagnostic.start, diagnostic.end))
            .collect();
        let expected: Vec<_> = ["defineProps", "defineEmits"]
            .iter()
            .map(|name| {
                let start = source.find(name).unwrap() as u32 + 7;
                (start, start + name.len() as u32)
            })
            .collect();
        assert_eq!(spans, expected);
    }
}
