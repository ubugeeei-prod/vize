//! Raw public formatter observations for the shared differential corpus.
//! This fixture executable does not provide a native or production route.

#![expect(
    clippy::disallowed_types,
    reason = "stdin transport requires an owned UTF-8 standard String"
)]

use std::io::{self, Read, Write};
use std::process::ExitCode;
use vize_glyph::{
    FormatError, FormatOptions, VueVersion, format_json, format_jsonc, format_script, format_sfc,
    format_sfc_with_vue_version, format_style, format_template, format_template_with_vue_version,
};
use vize_l0::{ToCompactString, cstr};

fn invalid(message: impl Into<std::string::String>) -> FormatError {
    io::Error::new(io::ErrorKind::InvalidInput, message.into()).into()
}

fn configured_options(
    args: &[std::string::String],
) -> Result<(FormatOptions, bool, Option<VueVersion>), FormatError> {
    let mut options = FormatOptions::default();
    let mut single_pass = false;
    let mut expect_error = false;
    let mut configured = false;
    let mut vue_version = None;
    let mut remaining = args;
    while let Some((flag, tail)) = remaining.split_first() {
        match flag.as_str() {
            "--vue-version" if vue_version.is_none() => {
                let Some((version, tail)) = tail.split_first() else {
                    return Err(invalid("missing Vue version"));
                };
                vue_version = Some(match version.as_str() {
                    "2" => VueVersion::V2,
                    "2.7" => VueVersion::V2_7,
                    "3" => VueVersion::V3,
                    _ => return Err(invalid("unsupported Vue version")),
                });
                remaining = tail;
                continue;
            }
            "--options" if !configured => {
                let Some((json, tail)) = tail.split_first() else {
                    return Err(invalid("missing options JSON"));
                };
                let value: serde_json::Value = serde_json::from_str(json)
                    .map_err(|error| invalid(error.to_compact_string()))?;
                let Some(object) = value.as_object() else {
                    return Err(invalid("formatter options must be an object"));
                };
                options = serde_json::from_value(value.clone())
                    .map_err(|error| invalid(error.to_compact_string()))?;
                let actual = serde_json::to_value(&options)
                    .map_err(|error| invalid(error.to_compact_string()))?;
                for (key, value) in object {
                    if actual.get(key) != Some(value) {
                        return Err(invalid(cstr!("unknown or noncanonical option: {key}")));
                    }
                }
                configured = true;
                remaining = tail;
                continue;
            }
            "--legacy-single-pass" if !single_pass => single_pass = true,
            "--expect-error" if !expect_error => expect_error = true,
            _ => return Err(invalid("unsupported or duplicate observer option")),
        }
        remaining = tail;
    }
    options.skip_script_stabilization = single_pass;
    Ok((options, expect_error, vue_version))
}

fn error_kind(error: &FormatError) -> &'static str {
    match error {
        FormatError::ParseError(_) => "ParseError",
        FormatError::ScriptParseError(_) => "ScriptParseError",
        FormatError::ScriptFormatError(_) => "ScriptFormatError",
        FormatError::TemplateParseError(_) => "TemplateParseError",
        FormatError::TemplateFormatError(_) => "TemplateFormatError",
        FormatError::StyleFormatError(_) => "StyleFormatError",
        FormatError::JsonFormatError(_) => "JsonFormatError",
        FormatError::IoError(_) => "IoError",
    }
}

fn observe() -> Result<ExitCode, FormatError> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let Some((api, flags)) = args.split_first() else {
        return Err(invalid("expected API"));
    };
    let api = api.as_str();
    let (options, expect_error, vue_version) = configured_options(flags)?;
    if !matches!(
        api,
        "--script"
            | "--sfc"
            | "--style"
            | "--template"
            | "--json"
            | "--jsonc"
            | "--defaults"
            | "--options-json"
    ) || (options.skip_script_stabilization
        && !matches!(api, "--script" | "--sfc" | "--defaults" | "--options-json"))
        || (expect_error && matches!(api, "--defaults" | "--options-json"))
        || (vue_version.is_some()
            && !matches!(
                api,
                "--sfc" | "--template" | "--defaults" | "--options-json"
            ))
    {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "unsupported API/profile").into());
    }
    if api == "--defaults" {
        writeln!(io::stdout().lock(), "{options:#?}")?;
        return Ok(ExitCode::SUCCESS);
    }
    if api == "--options-json" {
        let mut value =
            serde_json::to_value(&options).map_err(|error| invalid(error.to_compact_string()))?;
        let Some(object) = value.as_object_mut() else {
            return Err(invalid("serialized formatter options must be an object"));
        };
        object.insert(
            "skipScriptStabilization".into(),
            options.skip_script_stabilization.into(),
        );
        if let Some(version) = vue_version {
            object.insert("vueVersion".into(), version.as_str().into());
        }
        writeln!(io::stdout().lock(), "{value}")?;
        return Ok(ExitCode::SUCCESS);
    }
    let mut source = std::string::String::new();
    io::stdin().read_to_string(&mut source)?;
    let observed = match api {
        "--script" => format_script(&source, &options),
        "--style" => format_style(&source, &options),
        "--template" => vue_version.map_or_else(
            || format_template(&source, &options),
            |version| format_template_with_vue_version(&source, &options, version),
        ),
        "--json" => format_json(&source, &options),
        "--jsonc" => format_jsonc(&source, &options),
        "--sfc" => vue_version
            .map_or_else(
                || format_sfc(&source, &options),
                |version| format_sfc_with_vue_version(&source, &options, version),
            )
            .and_then(|result| {
                if !expect_error {
                    writeln!(io::stderr().lock(), "changed={}", result.changed)?;
                }
                Ok(result.code)
            }),
        _ => {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "unsupported API").into());
        }
    };
    if expect_error {
        let error = observed
            .err()
            .ok_or_else(|| invalid("expected a real formatter error"))?;
        writeln!(io::stdout().lock(), "{error:#?}")?;
        writeln!(io::stderr().lock(), "error={}", error_kind(&error))?;
        return Ok(ExitCode::FAILURE);
    }
    let output = observed?;
    io::stdout().lock().write_all(output.as_bytes())?;
    Ok(ExitCode::SUCCESS)
}

fn main() -> Result<ExitCode, FormatError> {
    observe()
}
