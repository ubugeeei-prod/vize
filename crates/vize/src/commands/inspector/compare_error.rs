//! Error formatting for inspector compare's official compiler subprocess.

use vize_carton::{String, cstr};

pub(super) fn official_compiler_error_message(stderr: &str) -> String {
    if is_missing_vue3_compiler(stderr) {
        return cstr!(
            "--format compare currently requires Vue 3 `@vue/compiler-sfc` / `vue/compiler-sfc`.\n  Vue 2 / Nuxt 2 projects are not supported by compare yet; use `--format json` or `--format agent` for inspector payloads without running the official compiler."
        );
    }

    cstr!(
        "--format compare requires @vue/compiler-sfc in the current project or Vize workspace dev dependencies.\n{stderr}"
    )
}

fn is_missing_vue3_compiler(stderr: &str) -> bool {
    let lower = stderr.to_ascii_lowercase();
    lower.contains("compiler-sfc")
        && (lower.contains("err_module_not_found") || lower.contains("cannot find module"))
}

#[cfg(test)]
mod tests {
    use super::official_compiler_error_message;

    #[test]
    fn hides_node_stack_for_missing_vue_compiler_sfc() {
        let message = official_compiler_error_message(
            "node:internal/modules/esm/resolve:271\n\
             Error [ERR_MODULE_NOT_FOUND]: Cannot find module '/app/node_modules/vue/compiler-sfc'",
        );

        assert!(message.contains("currently requires Vue 3"));
        assert!(message.contains("Vue 2 / Nuxt 2 projects are not supported"));
        assert!(!message.contains("node:internal/modules"));
        assert!(!message.contains("ERR_MODULE_NOT_FOUND"));
    }
}
