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
            Code::Lowering(_) | Code::Target(_) | Code::Emit(_) => "unsupported",
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
    use vize_atelier_sfc::compile_script::typescript::transform_typescript_to_js;

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

        let two = "export function A({ a }: { a: number }) @{\n  @switch (a) {\n    @case 1: {\n      <p>a</p>\n    }\n  }\n}\n\nexport function B({ b }: { b: number }) @{\n  <div>\n    @try {\n      <p>{b}</p>\n    } @catch (error) {\n      <p>failed</p>\n    }\n  </div>\n}\n";
        assert_eq!(
            refusal(two, false).as_str(),
            "2:3 [unsupported] Switch is not supported in TSRX modules yet\n11:5 [unsupported] Try is not supported in TSRX modules yet"
        );

        let arrow = "export const app = () => <p>hi</p>;\n";
        for erase_types in [false, true] {
            assert_eq!(
                refusal(arrow, erase_types).as_str(),
                "1:26 [unsupported] OutsideComponent is not emitted for the DOM target yet"
            );
        }
        let tab = "const tab = \"\\t\";\nexport function A() @{\n  <p>{tab}</p>\n}\n";
        assert_eq!(
            refusal(tab, true).as_str(),
            "1:1 [unsupported] type erasure of TSRX modules does not keep a tab character inside a literal yet"
        );
    }

    /// `--script-ext downcompile` erases a module as the `.vue` script erasure
    /// erases the same TypeScript, over every fixture the DOM target emits.
    #[test]
    fn erased_modules_equal_the_script_erasure_of_their_typescript_output() {
        let root =
            std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/_fixtures/tsrx");
        let mut files: Vec<_> = ["", "mount"]
            .into_iter()
            .flat_map(|dir| std::fs::read_dir(root.join(dir)).unwrap())
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "tsrx"))
            .collect();
        files.sort();
        let mut emitted = Vec::new();
        for path in files {
            let source = std::fs::read_to_string(&path).unwrap();
            let options = |erase_types| CompileOptions {
                filename: "fixture.tsrx",
                target: Target::Dom,
                source_map: false,
                erase_types,
            };
            let Ok(typescript) = compile(&source, &options(false)) else {
                continue;
            };
            let erased = compile(&source, &options(true)).unwrap();
            let name = path
                .strip_prefix(&root)
                .unwrap()
                .iter()
                .map(|part| part.to_str().unwrap())
                .collect::<Vec<_>>()
                .join("/");
            assert_eq!(
                erased.code.as_str(),
                transform_typescript_to_js(&typescript.code).as_str(),
                "{name}"
            );
            emitted.push(name);
        }
        assert_eq!(
            emitted,
            [
                "mount/attributes.tsrx",
                "mount/counter.tsrx",
                "mount/loops.tsrx",
                "mount/returns.tsrx",
                "mount/slots.tsrx",
                "mount/values.tsrx",
                "syntax-arrow-generic.tsrx",
                "syntax-asi.tsrx",
                "syntax-compare.tsrx",
                "syntax-empty-container.tsrx",
                "syntax-for-destructure.tsrx",
                "syntax-gt-text.tsrx",
                "syntax-trailing-semi-render.tsrx",
                "syntax-url-text.tsrx",
            ]
        );
    }
}
