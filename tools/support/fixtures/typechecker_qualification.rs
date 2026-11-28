//! Coverage is transport evidence; unavailable project inputs prevent semantic qualification.

use serde_json::Value;

pub fn typechecker_run_status(exit_code: i64, parsed: &Value) -> &'static str {
    if exit_code == 0 {
        return "ok";
    }
    let Some(files) = parsed.get("files").and_then(Value::as_array) else {
        return "findings";
    };
    for file in files {
        let path = file.get("file").and_then(Value::as_str).unwrap_or("");
        let Some(diagnostics) = file.get("diagnostics").and_then(Value::as_array) else {
            continue;
        };
        for diagnostic in diagnostics {
            let Some(text) = diagnostic.as_str() else {
                continue;
            };
            let Some(header) = text.strip_prefix("error:") else {
                continue;
            };
            let Some((position, diagnostic)) = header.split_once(" [TS") else {
                continue;
            };
            let Some((line, column)) = position.split_once(':') else {
                continue;
            };
            if line
                .parse::<u64>()
                .ok()
                .filter(|value| *value > 0)
                .is_none()
                || column
                    .parse::<u64>()
                    .ok()
                    .filter(|value| *value > 0)
                    .is_none()
            {
                continue;
            }
            let Some((code, _message)) = diagnostic.split_once("] ") else {
                continue;
            };
            let Ok(code) = code.parse::<u64>() else {
                continue;
            };
            // Deprecated and removed compiler options introduce no `any`.
            // They stay in the compared set; both checkers must agree on them.
            if matches!(code, 5101 | 5102 | 5107 | 5108) {
                continue;
            }
            // Program inputs that TypeScript could not construct make the run
            // partial. Positioned module-resolution diagnostics (TS2307/TS2792)
            // are ordinary semantic findings, though: vue-tsc can compare the
            // same authored import and its downstream diagnostics.
            if path.ends_with(".json")
                || path.ends_with(".jsonc")
                || matches!(
                    code,
                    2318 | 2688 | 5083 | 6053 | 6059 | 6305 | 6307 | 18002 | 18003
                )
            {
                return "configuration";
            }
        }
    }
    "findings"
}

#[cfg(test)]
mod tests {
    use super::typechecker_run_status;
    use serde_json::json;

    fn status(file: &str, diagnostic: &str) -> &'static str {
        typechecker_run_status(
            1,
            &json!({"files": [{"file": file, "diagnostics": [diagnostic]}]}),
        )
    }

    #[test]
    fn option_notices_do_not_block_qualification() {
        for code in [5101, 5102, 5107, 5108] {
            let diagnostic =
                format!("error:3:3 [TS{code}] Option 'downlevelIteration' has been removed.");
            assert_eq!(
                status("packages/lib/tsconfig.json", &diagnostic),
                "findings"
            );
        }
    }

    #[test]
    fn other_configuration_and_unavailable_inputs_still_block_it() {
        assert_eq!(
            status(
                "configs/build.jsonc",
                "error:1:1 [TS5023] Unknown compiler option 'missing'."
            ),
            "configuration"
        );
        for code in [2318, 2688, 5083, 6053, 6059, 6305, 6307, 18002, 18003] {
            let diagnostic = format!("error:1:1 [TS{code}] Unavailable input.");
            assert_eq!(status("src/App.vue", &diagnostic), "configuration");
        }
        for code in [2307, 2792] {
            let diagnostic = format!("error:1:1 [TS{code}] Cannot find module './x'.");
            assert_eq!(status("src/App.vue", &diagnostic), "findings");
        }
        // Mealie's pinned source imports a module it does not ship; plain tsc
        // and vue-tsc report the same authored error.
        assert_eq!(
            status(
                "frontend/app/composables/recipes/use-recipe-tools.ts",
                "error:3:28 [TS2307] Cannot find module '~/types/vuetify' or its corresponding type declarations."
            ),
            "findings"
        );
        let mixed = json!({"files": [
            {"file": "tsconfig.json", "diagnostics": ["error:1:1 [TS5102] Option 'baseUrl' has been removed."]},
            {"file": "src/App.vue", "diagnostics": ["error:1:1 [TS2307] Cannot find module './x'."]}
        ]});
        assert_eq!(typechecker_run_status(1, &mixed), "findings");
    }
}
