//! Stateless Atlas graph shared by analysis-oriented FFI hosts.

#[cfg(feature = "wasm")]
use vize_atelier_sfc::SfcRenderModuleArtifact;
#[cfg(any(feature = "wasm", test))]
use vize_atelier_sfc::SfcRenderModuleProduct;
use vize_atelier_sfc::{
    SfcCompileProduct, SfcCompileResult, SfcCroquisMode, SfcCroquisSettings, SfcDescriptorArtifact,
    SfcDescriptorProduct,
};
use vize_atlas::{Compilation, CompilationSnapshot, Shared, SourceId};
use vize_canon::{
    SfcTypeCheckOptions, SfcTypeCheckProduct, SfcTypeCheckRequest, SfcTypeCheckResult,
    install_sfc_typecheck_request, register_sfc_typecheck_provider,
};
use vize_carton::config::VueVersion;
use vize_carton::{FxHashMap, String, cstr};
use vize_croquis::{CroquisDocument, CroquisDocumentProduct};

/// Parse one FFI `vueVersion` option without silently rounding or falling back.
pub(crate) fn resolve_vue_version(value: Option<&str>) -> Result<VueVersion, String> {
    value.map_or(Ok(VueVersion::V3), |value| {
        VueVersion::from_config_str(value).map_err(|error| cstr!("invalid vueVersion: {error}"))
    })
}

/// Descriptor and complete module produced by one configured Atlas session.
pub(crate) struct SfcCompileArtifacts {
    descriptor: Shared<SfcDescriptorArtifact>,
    compiled: Result<Shared<SfcCompileResult>, String>,
    #[cfg(feature = "wasm")]
    render: Option<Result<Shared<SfcRenderModuleArtifact>, String>>,
    #[cfg(test)]
    descriptor_executions: u64,
    #[cfg(test)]
    descriptor_cache_hits: u64,
    #[cfg(test)]
    fallback_observations: usize,
    #[cfg(test)]
    compile_depends_on_dialect: bool,
    #[cfg(test)]
    render_cache_hit: bool,
}

impl SfcCompileArtifacts {
    pub(crate) fn descriptor_artifact(&self) -> &SfcDescriptorArtifact {
        &self.descriptor
    }

    pub(crate) fn compiled(&self) -> Result<&SfcCompileResult, &str> {
        self.compiled.as_deref().map_err(String::as_str)
    }

    #[cfg(feature = "wasm")]
    pub(crate) fn render(&self) -> Result<Option<&SfcRenderModuleArtifact>, &str> {
        self.render
            .as_ref()
            .map(|render| render.as_deref().map_err(String::as_str))
            .transpose()
    }
}

/// Query descriptor metadata and the host-ready SFC module from one session.
///
/// Callers must install every typed input before taking `snapshot`. The
/// complete module is the root plan. Descriptor and render metadata are then
/// cache-only reads from that plan, including when a malformed descriptor made
/// the root fail.
pub(crate) fn query_sfc_compile(
    snapshot: &CompilationSnapshot,
    source: SourceId,
) -> Result<SfcCompileArtifacts, String> {
    let mut session = snapshot.query_session();
    let (compiled, _fallback_observations, _compile_depends_on_dialect) = match session
        .query::<SfcCompileProduct>(
        source,
    ) {
        Ok(outcome) => {
            #[cfg(test)]
            let fallback_observations = outcome
                .execution()
                .observations()
                .iter()
                .filter(|observation| observation.kind() == vize_atlas::ObservationKind::Fallback)
                .count();
            #[cfg(not(test))]
            let fallback_observations = 0;
            let compile_depends_on_dialect = outcome
                .plan()
                .input_dependencies(vize_atlas::ProductId::of::<SfcCompileProduct>())
                .is_some_and(|inputs| {
                    inputs.contains(&vize_atlas::InputId::of::<vize_relief::VueDialectInput>())
                });
            (
                Ok(outcome.shared()),
                fallback_observations,
                compile_depends_on_dialect,
            )
        }
        Err(error) => (Err(cstr!("{error}")), 0, false),
    };
    let descriptor = session
        .query::<SfcDescriptorProduct>(source)
        .map_err(|error| cstr!("{error}"))?
        .shared();
    if let Some(error) = descriptor.diagnostic()
        && compiled.is_ok()
    {
        return Err(error.message.clone());
    }
    #[cfg(test)]
    let mut render_cache_hit = false;
    #[cfg(any(feature = "wasm", test))]
    let render = compiled
        .as_ref()
        .ok()
        .and_then(|_| descriptor.descriptor())
        .and_then(|descriptor| {
            descriptor.template.as_ref().map(|_| {
                session
                    .query::<SfcRenderModuleProduct>(source)
                    .map(|outcome| {
                        #[cfg(test)]
                        {
                            render_cache_hit =
                                outcome.status() == vize_atlas::ProductStatus::CacheHit;
                        }
                        outcome.shared()
                    })
                    .map_err(|error| cstr!("{error}"))
            })
        });
    #[cfg(all(test, not(feature = "wasm")))]
    let _ = render;
    #[cfg(test)]
    let descriptor_counters = session.counters().for_product::<SfcDescriptorProduct>();
    Ok(SfcCompileArtifacts {
        descriptor,
        compiled,
        #[cfg(feature = "wasm")]
        render,
        #[cfg(test)]
        descriptor_executions: descriptor_counters.executions(),
        #[cfg(test)]
        descriptor_cache_hits: descriptor_counters.cache_hits(),
        #[cfg(test)]
        fallback_observations: _fallback_observations,
        #[cfg(test)]
        compile_depends_on_dialect: _compile_depends_on_dialect,
        #[cfg(test)]
        render_cache_hit,
    })
}

pub(crate) struct SfcAnalysisArtifacts {
    descriptor: Shared<SfcDescriptorArtifact>,
    document: Shared<CroquisDocument>,
}

impl SfcAnalysisArtifacts {
    pub(crate) fn descriptor(&self) -> &vize_atelier_sfc::SfcDescriptor<'static> {
        self.descriptor
            .descriptor()
            .expect("artifact graph rejects malformed descriptors")
    }

    pub(crate) fn document(&self) -> &CroquisDocument {
        &self.document
    }
}

pub(crate) struct SfcAnalysisGraph {
    snapshot: CompilationSnapshot,
    sources: FxHashMap<String, SourceId>,
}

impl SfcAnalysisGraph {
    pub(crate) fn new<'a>(
        sources: impl IntoIterator<Item = (&'a str, &'a str)>,
        mode: SfcCroquisMode,
    ) -> Result<Self, String> {
        let mut compilation = Compilation::new();
        vize_atelier_sfc::register_atlas_providers(&mut compilation)
            .map_err(|error| cstr!("{error}"))?;
        let mut source_ids = FxHashMap::default();
        let mut settings = SfcCroquisSettings::new(mode);
        for (name, text) in sources {
            let source = compilation
                .add_source(name, text)
                .map_err(|error| cstr!("{error}"))?;
            settings.insert(source, mode);
            source_ids.insert(name.into(), source);
        }
        settings
            .install(&mut compilation)
            .map_err(|error| cstr!("{error}"))?;
        Ok(Self {
            snapshot: compilation.snapshot(),
            sources: source_ids,
        })
    }

    pub(crate) fn query(&self, name: &str) -> Result<SfcAnalysisArtifacts, String> {
        let source = self
            .sources
            .get(name)
            .copied()
            .ok_or_else(|| cstr!("SFC analysis source is not registered: {name}"))?;
        let mut session = self.snapshot.query_session();
        let descriptor = session
            .query::<SfcDescriptorProduct>(source)
            .map_err(|error| cstr!("{error}"))?;
        if let Some(error) = descriptor.value().diagnostic() {
            return Err(error.message.clone());
        }
        let document = session
            .query::<CroquisDocumentProduct>(source)
            .map_err(|error| cstr!("{error}"))?;
        Ok(SfcAnalysisArtifacts {
            descriptor: descriptor.shared(),
            document: document.shared(),
        })
    }

    #[cfg(test)]
    fn source(&self, name: &str) -> SourceId {
        self.sources[name]
    }
}

/// Stateless or batch-owned graph for the public AST-based typecheck host API.
pub(crate) struct SfcTypeCheckGraph {
    snapshot: CompilationSnapshot,
    sources: FxHashMap<String, SourceId>,
}

impl SfcTypeCheckGraph {
    pub(crate) fn new(
        sources: Vec<(String, String, SfcTypeCheckOptions)>,
        mode: SfcCroquisMode,
    ) -> Result<Self, String> {
        let mut compilation = Compilation::new();
        vize_atelier_sfc::register_atlas_providers(&mut compilation)
            .map_err(|error| cstr!("{error}"))?;
        register_sfc_typecheck_provider(&mut compilation).map_err(|error| cstr!("{error}"))?;
        let mut source_ids = FxHashMap::default();
        for (name, text, options) in sources {
            let source_name: String = if name.ends_with(".vue") {
                name.as_str().into()
            } else {
                cstr!("{name}.vue")
            };
            let source = compilation
                .add_source(source_name.as_str(), text)
                .map_err(|error| cstr!("{error}"))?;
            install_sfc_typecheck_request(
                &mut compilation,
                source,
                SfcTypeCheckRequest::new(options, mode),
            )
            .map_err(|error| cstr!("{error}"))?;
            source_ids.insert(name, source);
        }
        Ok(Self {
            snapshot: compilation.snapshot(),
            sources: source_ids,
        })
    }

    pub(crate) fn query(&self, name: &str) -> Result<SfcTypeCheckResult, String> {
        let source = self
            .sources
            .get(name)
            .copied()
            .ok_or_else(|| cstr!("SFC typecheck source is not registered: {name}"))?;
        self.snapshot
            .query_session()
            .query::<SfcTypeCheckProduct>(source)
            .map(|outcome| outcome.value().clone())
            .map_err(|error| cstr!("{error}"))
    }
}

#[cfg(test)]
#[path = "artifact_graph/tests.rs"]
mod tests;
