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
            // Missing modules/generated declarations can introduce `any` into
            // later checks. An authored error elsewhere cannot qualify this run.
            if path.ends_with(".json")
                || path.ends_with(".jsonc")
                || matches!(
                    code,
                    2307 | 2318 | 2688 | 2792 | 5083 | 6053 | 6059 | 6305 | 6307 | 18002 | 18003
                )
            {
                return "configuration";
            }
        }
    }
    "findings"
}
