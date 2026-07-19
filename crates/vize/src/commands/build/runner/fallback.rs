use std::{path::Path, sync::Mutex};

use super::super::config::{CompileError, CompileOutput, ErrorPhase};

pub(super) fn record_error(errors: &Mutex<Vec<CompileError>>, error: CompileError) {
    if let Ok(mut errors) = errors.lock() {
        errors.push(error);
    }
}

/// Print collected read/parse/compile errors grouped by phase.
pub(super) fn report_errors(errors: &[CompileError]) {
    if errors.is_empty() {
        return;
    }
    eprintln!();
    eprintln!(
        "\x1b[31m\u{2717} {} error(s) occurred:\x1b[0m",
        errors.len()
    );
    eprintln!();

    let read_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.phase == ErrorPhase::Read)
        .collect();
    let parse_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.phase == ErrorPhase::Parse)
        .collect();
    let compile_errors: Vec<_> = errors
        .iter()
        .filter(|e| e.phase == ErrorPhase::Compile)
        .collect();

    if !read_errors.is_empty() {
        eprintln!("  \x1b[31mRead errors ({}):\x1b[0m", read_errors.len());
        for err in &read_errors {
            eprintln!("    {} - {}", err.path.display(), err.error);
        }
        eprintln!();
    }

    if !parse_errors.is_empty() {
        eprintln!("  \x1b[31mParse errors ({}):\x1b[0m", parse_errors.len());
        for err in &parse_errors {
            eprintln!("    \x1b[1m{}\x1b[0m", err.path.display());
            for line in err.error.lines() {
                eprintln!("      {}", line);
            }
        }
        eprintln!();
    }

    if !compile_errors.is_empty() {
        eprintln!(
            "  \x1b[31mCompile errors ({}):\x1b[0m",
            compile_errors.len()
        );
        for err in &compile_errors {
            eprintln!("    \x1b[1m{}\x1b[0m", err.path.display());
            for line in err.error.lines() {
                eprintln!("      {}", line);
            }
        }
        eprintln!();
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
