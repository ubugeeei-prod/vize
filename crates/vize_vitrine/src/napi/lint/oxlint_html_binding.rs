//! Public boundary for the existing one-operation original HTML owner.
use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use std::path::Path;

use super::file_collection::oxlint_html_profile as profile;
use super::oxlint_html;
mod conversion;
pub mod dto;
mod report;
use dto::{OxlintHtmlOptions, OxlintHtmlOutcome, OxlintHtmlRefusal};

/// Select retained original HTML once, lint each retained original once, and
/// perform final presentation without any additional native query or lint pass.
#[napi(js_name = "lintOxlintHtml")]
pub fn lint_oxlint_html(options: OxlintHtmlOptions) -> Result<OxlintHtmlOutcome> {
    let host = match options.host_profile.as_str() {
        "1.78.0" => profile::HostProfile::Oxlint178,
        "1.86.0" => profile::HostProfile::Oxlint186,
        _ => {
            return Err(Error::new(
                Status::InvalidArg,
                "unsupported Oxlint host profile",
            ));
        }
    };
    report::validate(&options).map_err(|message| Error::new(Status::InvalidArg, message))?;
    let patterns = options
        .cli_ignore_patterns
        .iter()
        .map(|value| value.as_str().into())
        .collect::<Vec<_>>();
    let request = profile::Request {
        cwd: Path::new(&options.cwd),
        literal_target: &options.literal_target,
        root_json: Path::new(&options.root_json),
        host,
        no_ignore: options.no_ignore,
        cli_ignore_patterns: &patterns,
        custom_ignore_filename: &options.custom_ignore_filename,
    };
    match oxlint_html::run(request, &options.root_bytes) {
        Ok(operation) => {
            let rendered = report::render(&operation, &options)
                .map_err(|message| Error::new(Status::GenericFailure, message))?;
            let completed = conversion::completed(operation, options, rendered)
                .map_err(|message| Error::new(Status::GenericFailure, message))?;
            Ok(OxlintHtmlOutcome {
                completed: Some(completed),
                refused: None,
            })
        }
        Err(refusal) => Ok(OxlintHtmlOutcome {
            completed: None,
            refused: Some(OxlintHtmlRefusal {
                kind: vize_l0::cstr!("{:?}", refusal.kind).as_str().into(),
                path: refusal
                    .path
                    .to_str()
                    .ok_or_else(|| {
                        Error::new(Status::GenericFailure, "HTML refusal has a non-UTF8 path")
                    })?
                    .into(),
                details: refusal.details.as_str().into(),
                original_bytes: refusal.original_bytes,
            }),
        }),
    }
}
