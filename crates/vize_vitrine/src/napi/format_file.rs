//! NAPI bindings that format a source as `vize fmt --write` formats a file.

#![expect(
    clippy::disallowed_types,
    reason = "N-API values cross the boundary as std `String`s"
)]

use super::format::{
    FormatOptionsNapi, FormatResultNapi, apply_overrides, sort_imports_override,
    vue_version_override,
};
use napi::bindgen_prelude::{AsyncTask, Error, Result, Status};
use napi::{Env, Task};
use napi_derive::napi;
use std::path::Path;
use vize::format_api::{FormatConfig, format_source};

/// Where `formatFile` reads its file name, configuration and overrides.
#[napi(object)]
#[derive(Default)]
pub struct FormatFileOptionsNapi {
    /// File name whose extension selects the formatter and whose directory
    /// selects the configuration. Relative names resolve from the working
    /// directory. The file is never read or written.
    pub filename: String,
    /// Vite config file or project directory to read instead of discovering
    /// one from `filename`.
    pub config: Option<String>,
    /// Format with defaults only, without reading any configuration.
    pub no_config: Option<bool>,
    /// Options applied over the configuration, as `vize fmt` flags are.
    pub overrides: Option<FormatOptionsNapi>,
}

fn format_file(source: &str, request: FormatFileOptionsNapi) -> Result<FormatResultNapi> {
    let FormatFileOptionsNapi {
        filename,
        config,
        no_config,
        overrides,
    } = request;
    let config = match (no_config.unwrap_or(false), config.as_deref()) {
        (true, _) => FormatConfig::Defaults,
        (false, Some(config)) => FormatConfig::Path(Path::new(config)),
        (false, None) => FormatConfig::Discover,
    };
    let mut invalid = None;
    let result = format_source(source, Path::new(&filename), config, |settings| {
        let Some(overrides) = overrides else {
            return Ok(());
        };
        let applied = (|| {
            if let Some(version) = vue_version_override(&overrides)? {
                settings.vue_version = version;
            }
            if let Some(sorting) = sort_imports_override(&overrides)? {
                settings.sort_imports = sorting;
            }
            apply_overrides(&mut settings.options, overrides)
        })();
        applied.map_err(|error| {
            let message = error.reason.as_str().into();
            invalid = Some(error);
            message
        })
    });
    match (result, invalid) {
        (Ok(result), _) => Ok(FormatResultNapi {
            code: result.code.into(),
            changed: result.changed,
        }),
        (Err(_), Some(error)) => Err(error),
        (Err(message), None) => Err(Error::new(Status::GenericFailure, message.as_str())),
    }
}

/// Format `source` as the contents of `options.filename` on a worker thread,
/// with the configuration and per-file rules `vize fmt` applies to that file.
#[napi(js_name = "formatFile")]
pub fn format_file_napi(source: String, options: FormatFileOptionsNapi) -> AsyncTask<FormatFile> {
    AsyncTask::new(FormatFile {
        source,
        options: Some(options),
    })
}

/// Synchronous `formatFile`.
#[napi(js_name = "formatFileSync")]
pub fn format_file_sync_napi(
    source: String,
    options: FormatFileOptionsNapi,
) -> Result<FormatResultNapi> {
    format_file(&source, options)
}

pub struct FormatFile {
    source: String,
    options: Option<FormatFileOptionsNapi>,
}

impl Task for FormatFile {
    type Output = FormatResultNapi;
    type JsValue = FormatResultNapi;

    fn compute(&mut self) -> Result<Self::Output> {
        let options = self.options.take().unwrap_or_default();
        format_file(&self.source, options)
    }

    fn resolve(&mut self, _env: Env, output: Self::Output) -> Result<Self::JsValue> {
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::{FormatFileOptionsNapi, format_file};
    use crate::napi::format::FormatOptionsNapi;

    #[test]
    fn file_names_select_the_formatter_and_overrides_apply() {
        let format = |filename: &str, source: &str, overrides: Option<FormatOptionsNapi>| {
            format_file(
                source,
                FormatFileOptionsNapi {
                    filename: filename.into(),
                    no_config: Some(true),
                    overrides,
                    ..Default::default()
                },
            )
        };
        let script = format("gen/out.ts", "const a = {b:1}", None).expect("script");
        assert_eq!(script.code, "const a = { b: 1 };\n");
        assert!(script.changed);
        let unquoted = format(
            "gen/out.ts",
            "const a = 'b'",
            Some(FormatOptionsNapi {
                semi: Some(false),
                single_quote: Some(false),
                ..Default::default()
            }),
        )
        .expect("overridden script");
        assert_eq!(unquoted.code, "const a = \"b\"\n");
        let json = format("data.json", "{\"a\":1}", None).expect("json");
        assert_eq!(json.code, "{ \"a\": 1 }\n");

        let Err(unsupported) = format("notes.md", "# a", None) else {
            panic!("markdown is not formatted");
        };
        assert_eq!(
            unsupported.reason,
            "notes.md: YAML and Markdown formatting is not implemented; these formats are excluded from default discovery"
        );
        let Err(invalid) = format(
            "a.ts",
            "a",
            Some(FormatOptionsNapi {
                quote_props: Some("keep".into()),
                ..Default::default()
            }),
        ) else {
            panic!("an unknown quoteProps is refused");
        };
        assert_eq!(invalid.status, napi::Status::InvalidArg);
    }
}
