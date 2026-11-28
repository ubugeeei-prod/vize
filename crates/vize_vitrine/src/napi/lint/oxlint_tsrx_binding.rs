//! Original-path TSRX selection only. The authenticated JS carrier owns linting.
use super::file_collection::oxlint_html_profile as profile;
use super::oxlint_html_binding::dto::{OxlintHtmlOriginal, OxlintHtmlSource};
use napi::bindgen_prelude::{Error, Result, Status};
use napi_derive::napi;
use std::path::Path;

#[napi(object)]
pub struct OxlintTsrxOptions {
    pub cwd: String,
    pub literal_targets: Vec<String>,
    pub root_json: String,
    #[napi(ts_type = "Array<number>")]
    pub root_bytes: Vec<u8>,
    pub host_profile: String,
    pub no_ignore: bool,
    pub cli_ignore_patterns: Vec<String>,
    pub custom_ignore_filename: String,
}

#[napi(object)]
pub struct OxlintTsrxSelection {
    pub originals: Vec<OxlintHtmlOriginal>,
    pub sources: Vec<OxlintHtmlSource>,
}

/// Apply the exact locked ignore predicates to original `.tsrx` paths. Unsupported
/// host/config/filesystem domains are operational errors, never empty success.
#[napi(js_name = "selectOxlintTsrx")]
pub fn select_oxlint_tsrx(options: OxlintTsrxOptions) -> Result<OxlintTsrxSelection> {
    let host = match options.host_profile.as_str() {
        "1.78.0" => profile::HostProfile::Oxlint178,
        "1.81.0" => profile::HostProfile::Oxlint181,
        "1.86.0" => profile::HostProfile::Oxlint186,
        _ => {
            return Err(Error::new(
                Status::InvalidArg,
                "unsupported Oxlint TSRX host profile",
            ));
        }
    };
    if options.literal_targets.is_empty() {
        return Err(Error::new(
            Status::InvalidArg,
            "TSRX requires original literal targets",
        ));
    }
    let patterns = options
        .cli_ignore_patterns
        .iter()
        .map(|value| value.as_str().into())
        .collect::<Vec<_>>();
    let mut originals = Vec::new();
    let mut sources = Vec::new();
    let text = |path: &Path| {
        path.to_str()
            .map(String::from)
            .ok_or_else(|| Error::new(Status::GenericFailure, "non-UTF8 TSRX selection path"))
    };
    for target in &options.literal_targets {
        let request = profile::Request {
            cwd: Path::new(&options.cwd),
            literal_target: target,
            root_json: Path::new(&options.root_json),
            host,
            no_ignore: options.no_ignore,
            cli_ignore_patterns: &patterns,
            custom_ignore_filename: &options.custom_ignore_filename,
        };
        let selected = profile::select_tsrx(request, &options.root_bytes).map_err(|refusal| {
            Error::new(
                Status::GenericFailure,
                format!(
                    "{:?}: {}: {}",
                    refusal.kind,
                    refusal.path.display(),
                    refusal.details
                ),
            )
        })?;
        for original in selected.originals {
            originals.push(OxlintHtmlOriginal {
                path: text(&original.path)?,
                cwd_relative: text(&original.cwd_relative)?,
                origin: match original.origin {
                    profile::Origin::DirectoryDiscovery => "DirectoryDiscovery",
                    profile::Origin::ExplicitFile => "ExplicitFile",
                }
                .into(),
                bytes: original.bytes,
            });
        }
        for source in selected.sources {
            sources.push(OxlintHtmlSource {
                path: text(&source.path)?,
                role: format!("{:?}", source.role),
                bytes: source.bytes,
            });
        }
    }
    originals.sort_unstable_by(|left, right| left.path.cmp(&right.path));
    originals.dedup_by(|left, right| left.path == right.path);
    sources
        .sort_unstable_by(|left, right| (&left.path, &left.role).cmp(&(&right.path, &right.role)));
    sources.dedup_by(|left, right| left.path == right.path && left.role == right.role);
    Ok(OxlintTsrxSelection { originals, sources })
}
