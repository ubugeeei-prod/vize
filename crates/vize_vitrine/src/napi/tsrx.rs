//! NAPI binding for TSRX modules (#6858, #13140).
//!
//! `compileTsrx` runs the one TSRX compile entry,
//! [`vize_l4::lang::tsrx::compile`], which `vize build` also uses: a `.tsrx`
//! source string in, the compiled module or every diagnostic out. A refused
//! module has no code, so a bundler never receives partial output.
//!
//! FFI boundary code: uses std types for JavaScript interop.
#![expect(
    clippy::disallowed_types,
    reason = "N-API values cross the boundary as std `String`s"
)]
#![expect(
    clippy::disallowed_macros,
    reason = "N-API values cross the boundary as std `String`s"
)]

#[cfg(feature = "napi")]
use napi_derive::napi;
use vize_l0::line_index::offset_to_line_col;
use vize_l4::lang::tsrx::{Code, CompileOptions, Diagnostic, Target, compile};

/// Options for [`compile_tsrx`].
#[cfg_attr(feature = "napi", napi(object))]
#[derive(Default)]
pub struct TsrxCompileOptionsNapi {
    /// The module's file name, recorded in its source map.
    pub filename: Option<String>,
    /// `"dom"` (default), `"ssr"` or `"vapor"`.
    pub target: Option<String>,
    /// Produce a v3 source map in the result's `map`.
    pub source_map: Option<bool>,
    /// Erase TypeScript so `code` is JavaScript, as the `.vue` whole-script
    /// erasure prints it. When `false` (default), host code is copied verbatim
    /// and `code` is TypeScript without JSX.
    pub erase_types: Option<bool>,
}

/// One diagnostic of a refused module.
#[cfg_attr(feature = "napi", napi(object))]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TsrxDiagnosticNapi {
    /// The reference compiler's code for a syntax or template-rule error
    /// (`TSRX2001`, or a shared `TS1012`); `unsupported` for a construct,
    /// target or option that is not supported yet.
    pub code: String,
    pub message: String,
    /// UTF-8 byte offsets into the source.
    pub start: u32,
    pub end: u32,
    /// 1-based line of `start`.
    pub line: u32,
    /// 0-based UTF-16 column of `start`, as Rollup and Vite locations count.
    pub column: u32,
}

/// Result of [`compile_tsrx`].
#[cfg_attr(feature = "napi", napi(object))]
pub struct TsrxCompileResultNapi {
    /// The compiled module, absent when `errors` is non-empty.
    pub code: Option<String>,
    /// v3 source map (JSON) for `code`, when requested.
    pub map: Option<String>,
    /// Every diagnostic of a refused module; empty when it compiled.
    pub errors: Vec<TsrxDiagnosticNapi>,
}

#[cfg(feature = "napi")]
#[napi(js_name = "compileTsrx")]
pub fn compile_tsrx(
    source: String,
    options: Option<TsrxCompileOptionsNapi>,
) -> napi::Result<TsrxCompileResultNapi> {
    // Refusals are data (`errors`), so the result is always `Ok`; the work
    // lives in `compile_tsrx_impl` so tests run without the N-API runtime.
    Ok(compile_tsrx_impl(&source, options.unwrap_or_default()))
}

fn compile_tsrx_impl(source: &str, options: TsrxCompileOptionsNapi) -> TsrxCompileResultNapi {
    let target = match options.target.as_deref() {
        None | Some("dom") => Target::Dom,
        Some("ssr") => Target::Ssr,
        Some("vapor") => Target::Vapor,
        Some(other) => {
            let message =
                format!("unknown TSRX target {other:?}; expected \"dom\", \"ssr\" or \"vapor\"");
            return TsrxCompileResultNapi {
                code: None,
                map: None,
                errors: vec![TsrxDiagnosticNapi {
                    code: "unsupported".to_owned(),
                    message,
                    start: 0,
                    end: 0,
                    line: 1,
                    column: 0,
                }],
            };
        }
    };
    let filename = options.filename.as_deref().unwrap_or("anonymous.tsrx");
    let compile_options = CompileOptions {
        filename,
        target,
        source_map: options.source_map.unwrap_or(false),
        erase_types: options.erase_types.unwrap_or(false),
    };
    match compile(source, &compile_options) {
        Ok(compiled) => TsrxCompileResultNapi {
            code: Some(compiled.code.as_str().to_owned()),
            map: compiled.map.map(|map| map.as_str().to_owned()),
            errors: Vec::new(),
        },
        Err(failure) => TsrxCompileResultNapi {
            code: None,
            map: None,
            errors: failure
                .diagnostics
                .iter()
                .map(|diagnostic| napi_diagnostic(source, diagnostic))
                .collect(),
        },
    }
}

fn napi_diagnostic(source: &str, diagnostic: &Diagnostic) -> TsrxDiagnosticNapi {
    let (line, column) = offset_to_line_col(source, diagnostic.start as usize);
    let code = match &diagnostic.code {
        Code::Syntax(code) => code.as_str(),
        Code::Lowering(_) | Code::Target(_) | Code::Emit(_) => "unsupported",
    };
    TsrxDiagnosticNapi {
        code: code.to_owned(),
        message: diagnostic.message.as_str().to_owned(),
        start: diagnostic.start,
        end: diagnostic.end,
        line: line + 1,
        column,
    }
}

#[cfg(test)]
mod tests;
