//! `.tsrx` modules go through the one TSRX compile entry. Its diagnostics are
//! the file's errors and a refused module writes no output.

use std::{path::Path, time::Instant};

use vize_l0::{String, cstr, line_index::offset_to_line_col, profile};
use vize_l4::lang::tsrx::{Code, CompileOptions, Diagnostic, Target, compile};

use crate::commands::build::ScriptExtension;
use crate::commands::build::config::{CompileError, CompileOutput, ErrorPhase, FileProfile};

use super::super::capture::BuildCapture;
use super::super::profile_facts::{self, FileProfileFacts, StatsCacheStatus};
use super::super::settings::CompileFileSettings;

pub(super) fn compile_tsrx_file(
    path: &Path,
    source: &str,
    settings: &CompileFileSettings,
    file_start: Instant,
) -> Result<(CompileOutput, FileProfile, Option<BuildCapture>), CompileError> {
    let filename: String = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("anonymous.tsrx")
        .into();
    let target = if settings.ssr {
        Target::Ssr
    } else if settings.compiler_for_path(path).vapor {
        Target::Vapor
    } else {
        Target::Dom
    };
    let erase_types = matches!(settings.script_ext, ScriptExtension::Downcompile);
    let options = CompileOptions {
        filename: filename.as_str(),
        target,
        source_map: false,
        erase_types,
    };
    let compile_start = Instant::now();
    let compiled =
        profile!("l4.tsrx.compile", compile(source, &options)).map_err(|failure| CompileError {
            path: path.to_path_buf(),
            error: report(source, &failure.diagnostics),
            phase: ErrorPhase::Compile,
        })?;
    let compile_time = compile_start.elapsed();
    let profile = profile_facts::file_profile(
        path,
        FileProfileFacts {
            file_size: source.len(),
            parse_time: std::time::Duration::ZERO,
            compile_time,
            total_time: file_start.elapsed(),
            template_size: 0,
            script_size: source.len(),
            style_count: 0,
        },
        settings,
        StatsCacheStatus::NotRequested,
    );
    let output = CompileOutput {
        filename,
        code: compiled.code,
        css: None,
        errors: Vec::new(),
        warnings: Vec::new(),
        script_lang: (if erase_types { "js" } else { "ts" }).into(),
        macro_artifacts: Vec::new(),
    };
    Ok((output, profile, None))
}

/// One `line:column [code] message` line per diagnostic, 1-based.
fn report(source: &str, diagnostics: &[Diagnostic]) -> String {
    let mut lines = Vec::with_capacity(diagnostics.len());
    for diagnostic in diagnostics {
        let (line, column) = offset_to_line_col(source, diagnostic.start as usize);
        let code = match &diagnostic.code {
            Code::Syntax(code) => code.as_str(),
            Code::Lowering(_) | Code::Target(_) | Code::TypeErasure => "unsupported",
        };
        lines.push(cstr!(
            "{}:{} [{code}] {}",
            line + 1,
            column + 1,
            diagnostic.message
        ));
    }
    lines.join("\n").into()
}

#[cfg(test)]
mod tests {
    use super::{CompileOptions, Target, compile, report};

    fn refusal(source: &str, erase_types: bool) -> vize_l0::String {
        let options = CompileOptions {
            filename: "App.tsrx",
            target: Target::Dom,
            source_map: false,
            erase_types,
        };
        report(source, &compile(source, &options).unwrap_err().diagnostics)
    }

    #[test]
    fn refusals_report_each_diagnostic_at_its_line_and_column() {
        let source =
            include_str!("../../../../../../../tests/_fixtures/tsrx/errors/code-block-return.tsrx");
        let report = refusal(source, false);
        // TSRX2001 at bytes 62..74, `return null;` on line 2, column 19.
        assert_eq!(
            report.as_str(),
            "2:19 [TSRX2001] Return statements are not allowed inside TSRX templates. Move the return before the TSRX return value, or use conditional rendering instead."
        );

        let report = refusal("export const App = () => <p>hi</p>;\n", true);
        assert_eq!(
            report.as_str(),
            "1:1 [unsupported] Dom emission of TSRX modules is not supported yet\n1:1 [unsupported] type erasure of TSRX modules is not supported yet"
        );
    }
}
