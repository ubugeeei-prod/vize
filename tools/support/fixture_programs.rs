use super::{
    require_exact_keys, require_non_empty_string, require_normalized_path, require_record,
};
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
                let file = file.as_str().ok_or_else(|| {
                    format!("invalid typechecker JSON output: programs[{index}].files[{file_index}] must be a normalized relative path")
                })?;
                require_normalized_path(
                    file,
                    &format!("programs[{index}].files[{file_index}]"),
                    "typechecker",
                )?;
            }
        }
    }
    Ok(())
}
