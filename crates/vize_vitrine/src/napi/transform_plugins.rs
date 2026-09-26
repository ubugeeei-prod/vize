//! Sync JS callbacks over real L2 batches, before canonical compilation.
#![expect(
    clippy::disallowed_types,
    reason = "N-API serialized boundary uses std strings"
)]
#![expect(
    clippy::disallowed_methods,
    reason = "N-API serialized boundary uses owned strings"
)]
#![expect(clippy::disallowed_macros, reason = "napi derive uses std formatting")]
use super::plugin_sdk::PluginCacheInputNapi;
use crate::{
    CompileResult,
    plugin_transform::{self, Identity},
};
use napi::Env;
use napi::bindgen_prelude::{Error, FunctionRef, Result, Status};
use napi_derive::napi;

#[napi(object, object_to_js = false)]
pub struct TransformPluginNapi {
    pub name: String,
    pub version: String,
    pub fingerprint: String,
    pub cache_inputs: Option<Vec<PluginCacheInputNapi>>,
    pub run: FunctionRef<String, String>,
}

#[napi(object)]
#[derive(Default)]
pub struct TransformCompileOptionsNapi {
    pub filename: Option<String>,
    pub source_map: Option<bool>,
    pub hoist_static: Option<bool>,
    pub cache: Option<bool>,
    pub cache_dir: Option<String>,
}

#[napi(object)]
pub struct TransformPluginCostNapi {
    pub name: String,
    pub content_key: String,
    pub nodes: u32,
    pub edits: u32,
    pub cached: bool,
    pub elapsed_ns: f64,
    pub js_ns: f64,
}

#[napi(object)]
pub struct TransformCompileOutputNapi {
    pub result: CompileResult,
    pub plugins: Vec<TransformPluginCostNapi>,
}

/// Compile a template using bounded, deterministic edits to the native L2
/// artifact. Unsupported edits and incompatible canonical output are errors.
#[napi(js_name = "compileWithTransformPlugins")]
pub fn compile_with_transform_plugins(
    env: Env,
    template: String,
    plugins: Vec<TransformPluginNapi>,
    options: Option<TransformCompileOptionsNapi>,
) -> Result<TransformCompileOutputNapi> {
    let options = options.unwrap_or_default();
    let filename = options.filename.unwrap_or_else(|| "template.vue".into());
    let identities: Vec<Identity> = plugins
        .iter()
        .map(|plugin| Identity {
            name: plugin.name.clone(),
            version: plugin.version.clone(),
            fingerprint: plugin.fingerprint.clone(),
            cache_inputs: plugin.cache_inputs.as_ref().map(|inputs| {
                inputs
                    .iter()
                    .map(|input| (input.name.clone(), input.value.clone()))
                    .collect()
            }),
        })
        .collect();
    let output = plugin_transform::compile(
        &template,
        &filename,
        &identities,
        plugin_transform::CompileOptions {
            source_map: options.source_map == Some(true),
            hoist_static: options.hoist_static == Some(true),
            cache: options.cache == Some(true),
            cache_dir: options.cache_dir.as_deref().map(std::path::Path::new),
        },
        |index, batch| {
            let plugin = plugins
                .get(index)
                .ok_or_else(|| "missing transform callback".to_owned())?;
            plugin
                .run
                .borrow_back(&env)
                .and_then(|run| run.call(batch))
                .map_err(|error| error.to_string())
        },
    )
    .map_err(|error| Error::new(Status::InvalidArg, error))?;
    let map = output
        .result
        .map
        .map(|map| serde_json::from_str(map.as_str()))
        .transpose()
        .map_err(|error| Error::new(Status::GenericFailure, error.to_string()))?;
    Ok(TransformCompileOutputNapi {
        result: CompileResult {
            code: output.result.code.to_string(),
            preamble: output.result.preamble.to_string(),
            ast: serde_json::json!({}),
            map,
            helpers: Vec::new(),
            templates: None,
        },
        plugins: output
            .costs
            .into_iter()
            .map(|cost| TransformPluginCostNapi {
                name: cost.name,
                content_key: cost.content_key,
                nodes: cost.nodes,
                edits: cost.edits,
                cached: cost.cached,
                elapsed_ns: cost.elapsed_ns,
                js_ns: cost.js_ns,
            })
            .collect(),
    })
}
