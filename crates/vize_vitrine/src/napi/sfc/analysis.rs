//! Script-only component metadata shared by native Musea consumers.
#![expect(
    clippy::disallowed_types,
    reason = "the public JSON boundary owns standard strings"
)]
#![expect(
    clippy::disallowed_methods,
    reason = "the public JSON boundary owns standard strings"
)]
#![cfg_attr(
    test,
    expect(
        clippy::disallowed_macros,
        reason = "test fixtures build authored SFC input strings"
    )
)]

mod cache;
mod defaults;
#[cfg(test)]
mod tests;

use std::sync::Mutex;
#[cfg(feature = "napi")]
use std::sync::OnceLock;
use std::time::Instant;

#[cfg(feature = "napi")]
use napi_derive::napi;
use serde::Serialize;
use serde_json::Value;
use vize_atelier_sfc::croquis::{SfcCroquisOptions, analyze_sfc_descriptor};
use vize_atelier_sfc::{SfcParseOptions, parse_sfc};

use cache::AnalysisCache;
use defaults::{DefaultInput, project_defaults};

/// Metadata analysis does not resolve external modules or execute defaults.
#[cfg_attr(feature = "napi", napi(object))]
#[derive(Default)]
pub struct SfcAnalyzeOptions {
    pub filename: Option<String>,
    /// Include native phase timings and actual work counts in the result.
    pub profile: Option<bool>,
    /// Bypass the bounded exact-source cache, for cold measurements.
    pub cache: Option<bool>,
}

#[derive(Serialize)]
struct Prop {
    name: String,
    #[serde(rename = "type")]
    prop_type: String,
    required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_value: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_source: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    unsupported_default: bool,
}

#[derive(Serialize)]
struct Analysis {
    props: Vec<Prop>,
    emits: Vec<String>,
}

#[derive(Default, Serialize)]
#[serde(rename_all = "camelCase")]
struct Profile {
    cache_hit: bool,
    descriptor_parse_calls: u32,
    analyze_calls: u32,
    default_parse_calls: u32,
    serialize_calls: u32,
    descriptor_parse_ns: u64,
    analyze_ns: u64,
    default_projection_ns: u64,
    serialize_ns: u64,
}

#[cfg(feature = "napi")]
static CACHE: OnceLock<Mutex<AnalysisCache>> = OnceLock::new();

fn elapsed(start: Option<Instant>) -> u64 {
    start.map_or(0, |start| {
        u64::try_from(start.elapsed().as_nanos()).unwrap_or(u64::MAX)
    })
}

/// Analyze authored props and emits from the shared SFC script analysis.
/// Defaults are JSON values only when statically proven; executable expressions
/// remain explicit `unsupported_default` metadata with their authored source.
#[cfg(feature = "napi")]
#[napi(js_name = "analyzeSfc")]
pub fn analyze_sfc(source: String, options: Option<SfcAnalyzeOptions>) -> napi::Result<Value> {
    let options = options.unwrap_or_default();
    analyze(
        &source,
        &options,
        CACHE.get_or_init(|| Mutex::new(AnalysisCache::default())),
    )
    .map_err(|message| napi::Error::new(napi::Status::InvalidArg, message))
}

fn analyze(
    source: &str,
    options: &SfcAnalyzeOptions,
    cache: &Mutex<AnalysisCache>,
) -> Result<Value, String> {
    let filename = options.filename.as_deref().unwrap_or("anonymous.vue");
    let profiling = options.profile.unwrap_or(false);
    let use_cache = options.cache.unwrap_or(true);
    let mut profile = Profile::default();
    let cached = use_cache
        .then(|| {
            cache
                .lock()
                .ok()
                .and_then(|mut cache| cache.get(filename, source))
        })
        .flatten();
    let mut value = if let Some(cached) = cached {
        profile.cache_hit = true;
        (*cached).clone()
    } else {
        let started = profiling.then(Instant::now);
        let descriptor = parse_sfc(
            source,
            SfcParseOptions {
                filename: filename.into(),
                ..Default::default()
            },
        )
        .map_err(|error| error.message.to_string())?;
        profile.descriptor_parse_ns = elapsed(started);
        profile.descriptor_parse_calls = 1;
        let started = profiling.then(Instant::now);
        let croquis =
            analyze_sfc_descriptor(&descriptor, None, SfcCroquisOptions::for_declaration());
        profile.analyze_ns = elapsed(started);
        profile.analyze_calls = 1;
        let started = profiling.then(Instant::now);
        let inputs: Vec<_> = croquis
            .macros
            .props()
            .iter()
            .map(|prop| DefaultInput {
                source: prop.default_value.as_deref(),
                // Vue treats a Function-typed default as a value, never a factory.
                allow_factory: !prop
                    .prop_type
                    .as_deref()
                    .is_some_and(|kind| kind.contains("=>") || kind == "Function"),
            })
            .collect();
        profile.default_parse_calls = u32::from(inputs.iter().any(|input| input.source.is_some()));
        let defaults = project_defaults(&inputs);
        let props = croquis
            .macros
            .props()
            .iter()
            .zip(defaults)
            .map(|(prop, projected)| Prop {
                name: prop.name.as_str().to_owned(),
                prop_type: prop.prop_type.as_deref().unwrap_or("unknown").to_owned(),
                required: prop.required,
                default_value: projected.value,
                default_source: projected
                    .unsupported
                    .then(|| prop.default_value.as_deref().unwrap_or_default().to_owned()),
                unsupported_default: projected.unsupported,
            })
            .collect();
        profile.default_projection_ns = elapsed(started);
        let mut emits = Vec::new();
        for emit in croquis.macros.emits() {
            if !emits.iter().any(|name| name == emit.name.as_str()) {
                emits.push(emit.name.as_str().to_owned());
            }
        }
        let started = profiling.then(Instant::now);
        let value =
            serde_json::to_value(Analysis { props, emits }).map_err(|error| error.to_string())?;
        profile.serialize_ns = elapsed(started);
        profile.serialize_calls = 1;
        if use_cache && let Ok(mut cache) = cache.lock() {
            cache.insert(filename, source, value.clone());
        }
        value
    };
    if profiling && let Some(object) = value.as_object_mut() {
        object.insert(
            "profile".into(),
            serde_json::to_value(profile).map_err(|error| error.to_string())?,
        );
    }
    Ok(value)
}
