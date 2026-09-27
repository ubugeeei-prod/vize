//! Raw public formatter observations for the shared differential corpus.
//! This fixture executable does not provide a native or production route.

#![expect(
    clippy::disallowed_types,
    reason = "stdin transport requires an owned UTF-8 standard String"
)]

use std::io::{self, Read, Write};
use vize_glyph::{
    FormatError, FormatOptions, format_script, format_sfc, format_style, format_template,
};

fn observe() -> Result<(), FormatError> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let (api, single_pass) = match args.as_slice() {
        [api] => (api.as_str(), false),
        [api, profile] if profile == "--legacy-single-pass" => (api.as_str(), true),
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "expected API and optional legacy profile",
            )
            .into());
        }
    };
    if !matches!(
        api,
        "--script" | "--sfc" | "--style" | "--template" | "--defaults"
    ) || (single_pass && !matches!(api, "--script" | "--sfc" | "--defaults"))
    {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "unsupported API/profile").into());
    }
    let options = FormatOptions {
        skip_script_stabilization: single_pass,
        ..FormatOptions::default()
    };
    if api == "--defaults" {
        writeln!(io::stdout().lock(), "{options:#?}")?;
        return Ok(());
    }
    let mut source = std::string::String::new();
    io::stdin().read_to_string(&mut source)?;
    let output = match api {
        "--script" => format_script(&source, &options)?,
        "--style" => format_style(&source, &options)?,
        "--template" => format_template(&source, &options)?,
        "--sfc" => {
            let result = format_sfc(&source, &options)?;
            writeln!(io::stderr().lock(), "changed={}", result.changed)?;
            result.code
        }
        _ => {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "unsupported API").into());
        }
    };
    io::stdout().lock().write_all(output.as_bytes())?;
    Ok(())
}

fn main() -> Result<(), FormatError> {
    observe()
}
