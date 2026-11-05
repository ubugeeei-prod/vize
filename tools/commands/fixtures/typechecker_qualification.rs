//! Configuration-only typechecker runs are not semantic findings.

use serde_json::Value;

pub(super) fn typechecker_run_status(exit_code: i64, parsed: &Value) -> &'static str {
    if exit_code == 0 {
        return "ok";
    }
    let Some(files) = parsed.get("files").and_then(Value::as_array) else {
        return "findings";
    };
    let mut saw_diagnostic = false;
    for file in files {
        let path = file.get("file").and_then(Value::as_str).unwrap_or("");
        let Some(diagnostics) = file.get("diagnostics").and_then(Value::as_array) else {
            continue;
        };
        for diagnostic in diagnostics {
            let Some(text) = diagnostic.as_str() else {
                return "findings";
            };
            saw_diagnostic = true;
            if !path.ends_with("tsconfig.json")
                || !(text.contains("TS2688") || text.contains("TS6053"))
            {
                return "findings";
            }
        }
    }
    if saw_diagnostic {
        "configuration"
    } else {
        "findings"
    }
}
