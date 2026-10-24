use vize_carton::config::VueVersion;
use vize_croquis::Croquis;

use super::super::helpers::VUE_TYPE_HELPERS;
use super::super::types::{VirtualTsGenerationOptions, VirtualTsOptions, VirtualTsOutput};
use super::generate_virtual_ts_with_offsets_and_checks;
use super::spans::DEFINE_COMPONENT_HELPER;

const LEGACY_VUE_TYPE_HELPERS: &str = concat!(
    include_str!("../helpers/declarations/core-types.d.ts.txt"),
    include_str!("../helpers/declarations/vue2-types.d.ts.txt"),
    include_str!("../helpers/declarations/prop-keys.d.ts.txt"),
    include_str!("../helpers/declarations/vue2-component-and-list.d.ts.txt")
);
const LEGACY_REF_UNWRAP_HELPER: &str =
    "    type __U<T> = T extends { value: infer __V } ? __V : T;\n";
const LEGACY_EXPOSED_UNWRAP_HELPER: &str = "type __VizeShallowUnwrapRef<T> = { [K in keyof T]: T[K] extends { value: infer __V } ? __V : T[K] };\n";
/// Template-scope ref unwrapping for the modern (Vue 3) dialect when the shared
/// preamble is *not* hoisted (check server, content mapper).
///
/// The widening conditional types stay here, inside `__template()`, instead of
/// joining the module-scope preamble: `__U` is their only reference. Module-scope
/// declarations are module-local, so an unused one surfaces to the user as a
/// `TS6196` hint on their own file.
/// Check for a `value` property before referring to `vue.Ref`: when Vue types
/// are temporarily unavailable, that import can resolve to `any` and would
/// otherwise erase ordinary imported function signatures in template scope.
const MODERN_REF_UNWRAP_HELPER: &str =
    include_str!("../helpers/declarations/vue3-ref-unwrap.d.ts.txt");
/// [`MODERN_REF_UNWRAP_HELPER`] for the hoisted path: the ambient helpers file
/// (`SHARED_PREAMBLE_DTS`) is a `.d.ts` global script, so it declares the two
/// widening types once per program and never reports them unused.
///
/// `__U` itself stays per file — it is dialect-dependent. Hoisting the two
/// aliases saves 369 bytes in every generated `.vue.ts` and stops TypeScript
/// instantiating a distinct declaration per file instead of caching one
/// (#3443, #3460).
const MODERN_HOISTED_REF_UNWRAP_HELPER: &str = "    type __U<T> = T extends { value: unknown } ? T extends import('vue').Ref ? __VizeWidenTemplateRef<T['value']> : T : T;\n";
const MODERN_GENERIC_REF_UNWRAP_HELPER: &str = "    type __U<T> = T extends { value: unknown } ? T extends import('vue').Ref<any> ? T['value'] : T : T;\n";
const MODERN_EXPOSED_UNWRAP_HELPER: &str = "type __VizeShallowUnwrapRef<T> = { [K in keyof T]: T[K] extends import('vue').Ref<infer __V> ? __V : T[K] };\n";
const LEGACY_DEFINE_COMPONENT_HELPER: &str =
    include_str!("../helpers/declarations/vue2-define-component.d.ts.txt");
pub(super) const LEGACY_COMPONENT_INSTANCE_HELPER: &str =
    include_str!("../helpers/declarations/vue2-instance.d.ts.txt");

pub(super) fn needs_legacy_vue2_helpers(legacy_vue2: bool, dialect: VueVersion) -> bool {
    legacy_vue2 || matches!(dialect, VueVersion::V2 | VueVersion::V2_7)
}

pub(super) fn vue_type_helpers(legacy_vue2: bool, dialect: VueVersion) -> &'static str {
    if needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        LEGACY_VUE_TYPE_HELPERS
    } else {
        VUE_TYPE_HELPERS
    }
}

pub(super) fn ref_unwrap_helper(
    legacy_vue2: bool,
    dialect: VueVersion,
    hoist_shared_preamble: bool,
) -> &'static str {
    if needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        LEGACY_REF_UNWRAP_HELPER
    } else if hoist_shared_preamble {
        MODERN_HOISTED_REF_UNWRAP_HELPER
    } else {
        MODERN_REF_UNWRAP_HELPER
    }
}

pub(super) fn ref_unwrap_helper_for_template(
    legacy_vue2: bool,
    dialect: VueVersion,
    has_generic_param: bool,
    hoist_shared_preamble: bool,
) -> &'static str {
    if has_generic_param && !needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        MODERN_GENERIC_REF_UNWRAP_HELPER
    } else {
        ref_unwrap_helper(legacy_vue2, dialect, hoist_shared_preamble)
    }
}

pub(super) fn exposed_unwrap_helper(legacy_vue2: bool, dialect: VueVersion) -> &'static str {
    if needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        LEGACY_EXPOSED_UNWRAP_HELPER
    } else {
        MODERN_EXPOSED_UNWRAP_HELPER
    }
}

pub(super) fn define_component_helper(legacy_vue2: bool, dialect: VueVersion) -> &'static str {
    if needs_legacy_vue2_helpers(legacy_vue2, dialect) {
        LEGACY_DEFINE_COMPONENT_HELPER
    } else {
        DEFINE_COMPONENT_HELPER
    }
}

mod instance;
pub(super) use instance::{generic_instance_suffix, instance_helper, instance_suffix};

/// Generate virtual TypeScript with Vue 2.7 / Nuxt 2 compatibility enabled.
pub fn generate_virtual_ts_with_offsets_legacy_vue2(
    summary: &Croquis,
    script_content: Option<&str>,
    template_ast: Option<&vize_relief::RootNode<'_>>,
    script_offset: u32,
    template_offset: u32,
    options: &VirtualTsOptions,
) -> VirtualTsOutput {
    generate_virtual_ts_with_offsets_and_checks(
        summary,
        script_content,
        template_ast,
        script_offset,
        template_offset,
        options,
        VirtualTsGenerationOptions {
            legacy_vue2: true,
            ..Default::default()
        },
    )
}

#[cfg(test)]
mod tests {
    use std::{
        path::{Path, PathBuf},
        process::Command,
    };

    use vize_carton::{
        String, append,
        config::VueVersion,
        corsa_resolver::{CorsaResolveRequest, resolve_corsa_executable},
        cstr,
    };

    use super::vue_type_helpers;

    mod kebab_case;

    #[test]
    fn v_for_helper_preserves_member_array_aliases_without_losing_object_keys() {
        let Some(tsgo) = resolve_test_tsgo_binary() else {
            return;
        };
        let case_dir = std::env::temp_dir()
            .join(cstr!("vize-legacy-vfor-helper-{}", std::process::id()).as_str());
        let _ = std::fs::remove_dir_all(&case_dir);
        std::fs::create_dir_all(&case_dir).unwrap();
        let test_path = case_dir.join("vfor-helper.ts");
        let mut source = String::new("");
        append!(source, "{}\n\n", vue_type_helpers(true, VueVersion::V2));
        source.push_str(
            r#"interface QuestionField {
  questionFieldInputType: string;
  question: string;
  type: string;
}
interface QuestionFormat {
  id: string;
  questionField: QuestionField[];
}

declare const anySource: any;
for (const [answer, i] of __vForList(anySource.questionField)) {
  const keyText = `${i}question`;
  answer.questionFieldInputType;
  void keyText;
}

const obj: { foo: number; bar: number } = { foo: 1, bar: 2 };
for (const [value, key] of __vForList(obj)) {
  const valueCheck: number = value;
  const keyCheck: "foo" | "bar" = key;
  void valueCheck;
  void keyCheck;
}

declare const questionFormat: QuestionFormat;
for (const [answer, i] of __vForList(questionFormat.questionField)) {
  const inputType: string = answer.questionFieldInputType;
  const index: number = i;
  void inputType;
  void index;
}
"#,
        );
        std::fs::write(&test_path, source.as_bytes()).unwrap();

        let output = Command::new(tsgo)
            .args([
                "--ignoreConfig",
                "--noEmit",
                "--strict",
                "--target",
                "ES2022",
            ])
            .arg(&test_path)
            .output()
            .unwrap();
        let stdout = std::str::from_utf8(&output.stdout).unwrap();
        let stderr = std::str::from_utf8(&output.stderr).unwrap();
        assert!(
            output.status.success(),
            "legacy v-for helper should keep any/member-array aliases off the object overload while preserving object key types\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );

        let _ = std::fs::remove_dir_all(&case_dir);
    }

    fn resolve_test_tsgo_binary() -> Option<PathBuf> {
        if let Ok(path) = std::env::var("CORSA_PATH")
            && Path::new(&path).exists()
        {
            return Some(PathBuf::from(path));
        }

        let root = Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)?;
        resolve_corsa_executable(CorsaResolveRequest {
            explicit_path: None,
            project_root: Some(root),
        })
        .ok()
    }
}
