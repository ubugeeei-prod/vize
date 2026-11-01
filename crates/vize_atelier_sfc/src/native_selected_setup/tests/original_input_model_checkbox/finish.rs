//! Always persist all actual attempts before the first, unqualified verdict.

use serde_json::Value;

pub(super) fn write(capture: Value, complete: bool) -> Result<(), String> {
    let serialized = serde_json::to_string_pretty(&capture).map_err(|error| format!("{error}"))?;
    let first = capture["failures"]
        .as_array()
        .and_then(|values| values.first())
        .and_then(Value::as_str)
        .map(str::to_owned);
    let persistence = (|| -> Result<(), String> {
        if let Some(path) = std::env::var_os("VIZE_NATIVE_MODEL_CHECKBOX_CAPTURE") {
            let path = std::path::Path::new(&path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| format!("{error}"))?;
            }
            std::fs::write(path, &serialized).map_err(|error| format!("{error}"))?;
        }
        Ok(())
    })();
    if !complete || persistence.is_err() {
        eprintln!("CHECKBOX_MODEL_FAILED_CAPTURE={serialized}");
    }
    if !complete {
        let first = first.unwrap_or_else(|| "incomplete original checkbox observation".into());
        return Err(match persistence {
            Ok(()) => first,
            Err(error) => format!("{first}; additional capture-persistence failure: {error}"),
        });
    }
    persistence
}
