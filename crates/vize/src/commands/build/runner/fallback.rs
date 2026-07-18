use std::{path::Path, sync::Mutex};

use super::super::config::{CompileError, CompileOutput};

pub(super) fn record_error(errors: &Mutex<Vec<CompileError>>, error: CompileError) {
    if let Ok(mut errors) = errors.lock() {
        errors.push(error);
    }
}

pub(super) fn fallback_output(source: &Path, error: &CompileError) -> CompileOutput {
    let filename = source
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("anonymous.vue")
        .into();
    CompileOutput {
        filename,
        code: "".into(),
        css: None,
        errors: vec![error.error.clone()],
        warnings: Vec::new(),
        script_lang: "js".into(),
        macro_artifacts: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{CompileError, CompileOutput, fallback_output};
    use crate::commands::build::config::ErrorPhase;

    #[test]
    fn fallback_output_preserves_compile_errors_in_json_shape() {
        let output: CompileOutput = fallback_output(
            PathBuf::from("src/Broken.vue").as_path(),
            &CompileError {
                path: PathBuf::from("src/Broken.vue"),
                error: "semantic compile failure".into(),
                phase: ErrorPhase::Compile,
            },
        );

        assert_eq!(output.filename, "Broken.vue");
        assert!(output.code.is_empty());
        assert!(output.css.is_none());
        assert_eq!(output.errors, ["semantic compile failure"]);
        assert!(output.warnings.is_empty());
        assert_eq!(output.script_lang, "js");
        assert!(output.macro_artifacts.is_empty());
    }
}
