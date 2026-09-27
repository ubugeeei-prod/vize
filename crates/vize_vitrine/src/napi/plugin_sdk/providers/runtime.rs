//! Sync JS provider execution on the calling thread, once per demanded closure.
use napi::Env;
use napi::bindgen_prelude::{Error, FunctionRef, Result, Status};
use napi_derive::napi;
use serde_json::{Map, Value};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Instant;
use vize_davinci::fact::FactManager;

use super::super::PluginCacheInputNapi;
use super::super::batch::{BuiltBatch, PluginSpec};
use super::super::document::PluginDocument;
use super::super::plugin_cache::{PluginCacheInput, content_key};
use super::cache::cache;
use super::{ProviderCatalog, ProviderError, ProviderOutput, ProviderSpec, custom_batch, refusal};
use vize_davinci::key::source_block_key;

#[napi(object, object_to_js = false)]
pub struct JsFactProviderNapi {
    pub name: String,
    pub version: String,
    pub fingerprint: String,
    pub visit: Option<Vec<String>>,
    pub demands: Option<Vec<String>>,
    pub provides: Vec<String>,
    pub cache_inputs: Option<Vec<PluginCacheInputNapi>>,
    pub run: FunctionRef<String, String>,
}

#[napi(object)]
#[derive(Clone)]
pub struct ProviderCostNapi {
    pub name: String,
    pub version: String,
    pub content_key: String,
    pub provided_groups: u32,
    pub nodes: u32,
    pub batch_bytes: u32,
    pub elapsed_ns: f64,
    pub js_ns: f64,
    pub audited: bool,
    pub cached: bool,
}

pub struct ProviderBatch {
    pub built: BuiltBatch,
    pub cache_inputs: Vec<(String, String)>,
}

struct Provided {
    values: Map<String, Value>,
    stamp: String,
}

pub struct ProviderHost {
    providers: Vec<JsFactProviderNapi>,
    catalog: ProviderCatalog,
    results: BTreeMap<usize, Provided>,
    costs: Vec<ProviderCostNapi>,
    use_cache: bool,
    cache_dir: Option<PathBuf>,
}

fn error(error: ProviderError) -> Error {
    Error::new(Status::InvalidArg, error.to_string())
}

fn spec(provider: &JsFactProviderNapi) -> PluginSpec<'_> {
    PluginSpec {
        name: &provider.name,
        version: &provider.version,
        fingerprint: &provider.fingerprint,
        visit: provider.visit.as_deref(),
        demands: provider.demands.as_deref().unwrap_or_default(),
    }
}

fn inputs(provider: &JsFactProviderNapi) -> Vec<PluginCacheInput<'_>> {
    provider
        .cache_inputs
        .as_deref()
        .unwrap_or_default()
        .iter()
        .map(|input| PluginCacheInput {
            name: &input.name,
            value: &input.value,
        })
        .collect()
}

impl ProviderHost {
    pub fn new(providers: Vec<JsFactProviderNapi>) -> Result<Self> {
        let inputs: Vec<_> = providers.iter().map(inputs).collect();
        let specs: Vec<_> = providers
            .iter()
            .zip(&inputs)
            .map(|(provider, inputs)| ProviderSpec {
                plugin: spec(provider),
                provides: &provider.provides,
                inputs_declared: provider.cache_inputs.is_some(),
                inputs,
            })
            .collect();
        let catalog = ProviderCatalog::new(&specs).map_err(error)?;
        Ok(Self {
            providers,
            catalog,
            results: BTreeMap::new(),
            costs: Vec::new(),
            use_cache: false,
            cache_dir: None,
        })
    }

    pub fn configure_cache(&mut self, use_cache: bool, cache_dir: Option<&Path>) {
        self.use_cache = use_cache;
        self.cache_dir = cache_dir.map(Path::to_path_buf);
    }

    pub fn batch(
        &mut self,
        env: &Env,
        document: &PluginDocument,
        spec: &PluginSpec<'_>,
        manager: &mut FactManager<'_, PluginDocument>,
    ) -> Result<ProviderBatch> {
        for index in self.catalog.demanded(spec).map_err(error)? {
            if !self.results.contains_key(&index) {
                self.execute(index, env, document, manager)?;
            }
        }
        let (custom, cache_inputs) = self.declared(spec.demands);
        let built = custom_batch(document, spec, manager, &custom).map_err(error)?;
        Ok(ProviderBatch {
            built,
            cache_inputs,
        })
    }

    pub fn costs(&self) -> Vec<ProviderCostNapi> {
        self.costs.clone()
    }

    fn declared(&self, demands: &[String]) -> (Map<String, Value>, Vec<(String, String)>) {
        let mut facts = Map::new();
        let mut inputs = Vec::new();
        for name in demands {
            if let Some(provided) = self
                .results
                .values()
                .find(|result| result.values.contains_key(name))
                && let Some(value) = provided.values.get(name)
            {
                facts.insert(name.clone(), value.clone());
                inputs.push((format!("@vize/fact:{name}"), provided.stamp.clone()));
            }
        }
        inputs.sort_unstable();
        inputs.dedup();
        (facts, inputs)
    }

    fn execute(
        &mut self,
        index: usize,
        env: &Env,
        document: &PluginDocument,
        manager: &mut FactManager<'_, PluginDocument>,
    ) -> Result<()> {
        let started = Instant::now();
        let provider = self
            .providers
            .get(index)
            .ok_or_else(|| error(refusal("host", "invalid provider index")))?;
        let spec = spec(provider);
        let (custom, stamps) = self.declared(spec.demands);
        let built = custom_batch(document, &spec, manager, &custom).map_err(error)?;
        let provides = serde_json::to_string(&provider.provides)
            .map_err(|failure| error(refusal(spec.name, &failure.to_string())))?;
        let mut inputs = inputs(provider);
        let batch_key = source_block_key("plugin-fact-batch", &[], &built.json).to_string();
        inputs.push(PluginCacheInput {
            name: "@vize/provider-batch",
            value: &batch_key,
        });
        inputs.push(PluginCacheInput {
            name: "@vize/provider-provides",
            value: &provides,
        });
        inputs.extend(
            stamps
                .iter()
                .map(|(name, value)| PluginCacheInput { name, value }),
        );
        let key = content_key(&document.source, &document.filename, &spec, &inputs)
            .ok_or_else(|| error(refusal(spec.name, "provider inputs could not be keyed")))?;
        let bytes = built.json.len() as u32;
        let hit = self
            .use_cache
            .then(|| {
                cache().lock().ok()?.get(
                    &key,
                    spec.name,
                    &provider.provides,
                    self.cache_dir.as_deref(),
                )
            })
            .flatten();
        let (output, js_ns, cached) = match hit {
            Some(output) => (output, 0.0, true),
            None => {
                let run = provider.run.borrow_back(env)?;
                let called = Instant::now();
                let first = run.call(built.json.clone())?;
                let second = run.call(built.json)?;
                let js_ns = called.elapsed().as_nanos() as f64;
                let output = ProviderOutput::audit(spec.name, &provider.provides, &first, &second)
                    .map_err(error)?;
                if self.use_cache
                    && let Ok(mut cache) = cache().lock()
                {
                    cache.put(&key, &output, self.cache_dir.as_deref());
                }
                (output, js_ns, false)
            }
        };
        let stamp = serde_json::to_string(&(&key, &output.result_key))
            .map_err(|failure| error(refusal(spec.name, &failure.to_string())))?;
        self.costs.push(ProviderCostNapi {
            name: provider.name.clone(),
            version: provider.version.clone(),
            content_key: key,
            provided_groups: output.values.len() as u32,
            nodes: if cached { 0 } else { built.nodes },
            batch_bytes: if cached { 0 } else { bytes },
            elapsed_ns: started.elapsed().as_nanos() as f64,
            js_ns,
            audited: !cached,
            cached,
        });
        self.results.insert(
            index,
            Provided {
                values: output.values,
                stamp,
            },
        );
        Ok(())
    }
}

#[cfg(test)]
mod migration;
