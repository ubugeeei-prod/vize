use super::{require_exact_keys, require_non_empty_string, require_record};
use serde_json::Value;

pub(super) fn validate(output: &Value) -> Result<(), String> {
    if let Some(programs) = output.get("programs").and_then(Value::as_array) {
        for (index, program) in programs.iter().enumerate() {
            let object = require_record(program, &format!("programs[{index}]"), "typechecker")?;
            let expected_keys = if program.get("tsconfig").is_some_and(Value::is_string) {
                &["compilerOptions", "files", "root", "tsconfig"][..]
            } else {
                &["files", "root"][..]
            };
            require_exact_keys(
                object,
                expected_keys,
                &format!("programs[{index}]"),
                "typechecker",
            )?;
            require_non_empty_string(
                &program["root"],
                &format!("programs[{index}].root"),
                "typechecker",
            )?;
            if object.contains_key("tsconfig") {
                require_non_empty_string(
                    &program["tsconfig"],
                    &format!("programs[{index}].tsconfig"),
                    "typechecker",
                )?;
            }
            if object.contains_key("compilerOptions") {
                require_record(
                    &program["compilerOptions"],
                    &format!("programs[{index}].compilerOptions"),
                    "typechecker",
                )?;
            }
            let program_files = program["files"].as_array().ok_or_else(|| {
                format!("invalid typechecker JSON output: programs[{index}].files must be an array")
            })?;
            for (file_index, file) in program_files.iter().enumerate() {
                require_program_path(file, &format!("programs[{index}].files[{file_index}]"))?;
            }
        }
    }
    Ok(())
}

fn require_program_path(value: &Value, label: &str) -> Result<(), String> {
    if value.as_str().is_some_and(is_normalized_program_path) {
        return Ok(());
    }
    Err(format!(
        "invalid typechecker JSON output: {label} must be a normalized program path"
    ))
}

// Program metadata includes compilerOptions.types resolved from ancestor node_modules.
// Unlike diagnostic and fixture-input paths, these members can be outside the cwd.
fn is_normalized_program_path(value: &str) -> bool {
    if value.is_empty() || value.contains(['\\', '\0']) {
        return false;
    }
    let drive = value
        .as_bytes()
        .first()
        .is_some_and(u8::is_ascii_alphabetic)
        && value.as_bytes().get(1) == Some(&b':');
    let components = if drive {
        if value.as_bytes().get(2) != Some(&b'/') {
            return false;
        }
        &value[3..]
    } else if let Some(unc) = value.strip_prefix("//") {
        if unc.split('/').count() < 2 {
            return false;
        }
        unc
    } else {
        value.strip_prefix('/').unwrap_or(value)
    };
    components
        .split('/')
        .all(|segment| !segment.is_empty() && segment != "." && segment != "..")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> Value {
        serde_json::from_str(include_str!(
            "../../tests/tooling/fixtures/typechecker-external-program.json"
        ))
        .unwrap()
    }

    #[test]
    fn external_members_preserve_complete_program_and_checked_input_evidence() {
        let output = fixture();
        let original = output.clone();
        let inputs = vec![output["files"][0]["file"].as_str().unwrap().to_string()];
        let coverage = super::super::validate_typechecker_output(
            &json!({"id": "inertia"}),
            &output,
            0,
            Some(&inputs),
            &inputs,
        )
        .unwrap();
        let mut relative_only = original.clone();
        relative_only["programs"][0]["files"] = json!(inputs);
        assert_eq!(
            coverage,
            super::super::validate_typechecker_output(
                &json!({"id": "inertia"}),
                &relative_only,
                0,
                Some(&inputs),
                &inputs,
            )
            .unwrap()
        );
        assert_eq!(output, original);
    }

    #[test]
    fn program_members_accept_normalized_paths_on_all_hosts() {
        for path in [
            "src/App.vue",
            "node_modules/vue/index.d.ts",
            "/home/runner/node_modules/vite/client.d.ts",
            "C:/repo/node_modules/vite/client.d.ts",
            "//server/share/node_modules/vite/client.d.ts",
        ] {
            let mut output = fixture();
            output["programs"][0]["files"] = json!([path]);
            assert_eq!(validate(&output), Ok(()), "{path}");
        }
    }

    #[test]
    fn malformed_program_members_and_absolute_diagnostic_files_remain_rejected() {
        for path in [
            json!(""),
            json!("../types.d.ts"),
            json!("/repo/../types.d.ts"),
            json!("C:/repo/./types.d.ts"),
            json!("//server//types.d.ts"),
            json!("./types.d.ts"),
            json!("/repo//types.d.ts"),
            json!("C:types.d.ts"),
            json!("C:\\repo\\types.d.ts"),
            json!("/repo/types.d.ts\0"),
            json!("/"),
            json!("C:/"),
            json!("//server"),
            json!(null),
            json!(42),
        ] {
            let mut output = fixture();
            output["programs"][0]["files"] = json!([path]);
            assert_eq!(validate(&output), Err("invalid typechecker JSON output: programs[0].files[0] must be a normalized program path".to_string()));
        }
        let mut output = fixture();
        output["files"][0]["file"] = json!("/repo/src/App.vue");
        assert_eq!(
            super::super::validate_typechecker_output(
                &json!({"id": "inertia"}),
                &output,
                0,
                None,
                &[],
            ),
            Err(
                "invalid typechecker JSON output: files[0].file must be a normalized relative path"
                    .to_string()
            )
        );
    }
}
