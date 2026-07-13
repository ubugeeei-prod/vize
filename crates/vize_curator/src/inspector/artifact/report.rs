//! Multi-source inspector report root.

use vize_atlas::{
    Compilation, Product, ProductRequest, Provider, ProviderContext, ProviderError, QueryOutcome,
    RegisterProviderError, SourceId, SourceInput, SourceInputId,
};

use super::super::payload::{
    InspectorAgentReport, InspectorPayload, InspectorSourceFile, build_agent_report_from_analyses,
};
use super::source::{
    InspectorSourceAnalysisProduct, InspectorSourceAnalysisProvider, is_inspector_source,
};

/// Complete immutable request attached to the report's anchor source.
#[derive(Clone)]
pub struct InspectorAgentRequest {
    pub payload: InspectorPayload,
    pub playground_url: vize_carton::String,
}

/// Source-scoped report request so unrelated roots are not invalidated.
pub struct InspectorAgentRequestInput;

impl SourceInput for InspectorAgentRequestInput {
    type Value = InspectorAgentRequest;

    const NAME: &'static str = "curator.inspector.agent-request";
}

/// Complete multi-source inspector agent report.
pub struct InspectorAgentReportProduct;

impl Product for InspectorAgentReportProduct {
    type Value = InspectorAgentReport;

    const NAME: &'static str = "curator.inspector.agent-report";
}

/// Aggregates only explicitly requested per-source analyses.
pub struct InspectorAgentReportProvider;

impl Provider for InspectorAgentReportProvider {
    type Product = InspectorAgentReportProduct;

    fn source_input_dependencies(&self) -> Vec<SourceInputId> {
        vec![SourceInputId::of::<InspectorAgentRequestInput>()]
    }

    fn dependency_requests(
        &self,
        context: &vize_atlas::PlanningContext<'_>,
    ) -> Vec<ProductRequest> {
        source_ids(context)
            .into_iter()
            .map(ProductRequest::for_product::<InspectorSourceAnalysisProduct>)
            .collect()
    }

    fn provide(
        &self,
        context: &mut ProviderContext<'_>,
    ) -> Result<InspectorAgentReport, ProviderError> {
        let request = context
            .source_input::<InspectorAgentRequestInput>()
            .cloned()
            .ok_or_else(|| ProviderError::message("missing inspector agent request"))?;
        let mut sources: Vec<_> = context
            .sources()
            .iter()
            .filter(|source| is_inspector_source(source.name()))
            .cloned()
            .collect();
        sources.sort_by(|left, right| {
            left.name()
                .cmp(right.name())
                .then_with(|| left.id().cmp(&right.id()))
        });
        let mut analyzed = Vec::with_capacity(sources.len());
        for source in &sources {
            let analysis = context.get_for_source::<InspectorSourceAnalysisProduct>(source.id())?;
            analyzed.push((source.clone(), analysis));
        }
        Ok(build_agent_report_from_analyses(
            request.payload,
            request.playground_url,
            &analyzed,
        ))
    }
}

fn source_ids(context: &vize_atlas::PlanningContext<'_>) -> Vec<SourceId> {
    let mut sources: Vec<_> = context
        .sources()
        .iter()
        .filter(|source| is_inspector_source(source.name()))
        .collect();
    sources.sort_by(|left, right| {
        left.name()
            .cmp(right.name())
            .then_with(|| left.id().cmp(&right.id()))
    });
    sources.into_iter().map(|source| source.id()).collect()
}

pub fn register_inspector_atlas_providers(
    compilation: &mut Compilation,
) -> Result<(), RegisterProviderError> {
    vize_atelier_sfc::register_atlas_providers(compilation)?;
    if !compilation.has_provider::<InspectorSourceAnalysisProduct>() {
        compilation.register_provider(InspectorSourceAnalysisProvider)?;
    }
    if !compilation.has_provider::<InspectorAgentReportProduct>() {
        compilation.register_provider(InspectorAgentReportProvider)?;
    }
    Ok(())
}

pub(in crate::inspector) struct InspectorReportGraph {
    compilation: Compilation,
    anchor: SourceId,
}

impl InspectorReportGraph {
    pub(in crate::inspector) fn new(
        payload: InspectorPayload,
        playground_url: vize_carton::String,
        files: &[InspectorSourceFile],
    ) -> Result<Self, vize_carton::String> {
        let mut compilation = Compilation::new();
        register_inspector_atlas_providers(&mut compilation)
            .map_err(|error| vize_carton::cstr!("register inspector providers: {error}"))?;
        let mut anchor = None;
        for file in files {
            let source = compilation
                .add_source(file.path.as_str(), file.source.as_str())
                .map_err(|error| vize_carton::cstr!("add inspector source: {error}"))?;
            anchor.get_or_insert(source);
        }
        let anchor = anchor.ok_or_else(|| vize_carton::cstr!("inspector report has no sources"))?;
        compilation
            .set_source_input::<InspectorAgentRequestInput>(
                anchor,
                InspectorAgentRequest {
                    payload,
                    playground_url,
                },
            )
            .map_err(|error| vize_carton::cstr!("set inspector request: {error}"))?;
        Ok(Self {
            compilation,
            anchor,
        })
    }

    pub(in crate::inspector) fn query(
        &mut self,
    ) -> Result<QueryOutcome<InspectorAgentReportProduct>, vize_carton::String> {
        self.compilation
            .query::<InspectorAgentReportProduct>(self.anchor)
            .map_err(|error| vize_carton::cstr!("query inspector report: {error}"))
    }
}
