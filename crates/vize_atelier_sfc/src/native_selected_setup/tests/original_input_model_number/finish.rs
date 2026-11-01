//! Always retain the observed packet before returning its genuine verdict.

use serde_json::Value;

pub(super) fn write(capture: Value, complete: bool) -> Result<(), String> {
    let serialized = serde_json::to_string_pretty(&capture).map_err(|error| format!("{error}"))?;
    let first_source_failure = capture["failures"]
        .as_array()
        .and_then(|rows| rows.first())
        .and_then(Value::as_str)
        .map(str::to_owned);
    let persistence = (|| -> Result<(), String> {
        if let Some(path) = std::env::var_os("VIZE_NATIVE_MODEL_NUMBER_CAPTURE") {
            let path = std::path::Path::new(&path);
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).map_err(|error| format!("{error}"))?;
            }
            std::fs::write(path, &serialized).map_err(|error| format!("{error}"))?;
        }
        Ok(())
    })();
    if !complete || persistence.is_err() {
        eprintln!("NUMBER_MODEL_FAILED_CAPTURE={serialized}");
    }
    if !complete {
        let first = first_source_failure
            .unwrap_or_else(|| "incomplete original number source observation".into());
        return Err(match persistence {
            Ok(()) => first,
            Err(error) => format!("{first}; additional capture-persistence failure: {error}"),
        });
    }
    persistence
}
