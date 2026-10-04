//! Explicit original scriptless scoped CSS and complete SSR component output.

use crate::{
    NativeSfcCompileError, NativeSsrSfcCompileOptions,
    native::{scope, styles},
};
use vize_l0::{Allocator, String};
use vize_l1::container::vue::DescriptorOptions;
use vize_l1_to_l2::native_file::{
    NativeSelectedScopedSfcObservation, NativeSelectedSfcIssue, lower_selected_scoped_sfc_native,
};
use vize_l3::decision::ssr::{NativeSsrBuildError, build_native_scoped_ssr_file_decisions};
use vize_l4::{
    module::{
        AssemblyError, ModuleParts, RenderPlacement, RenderProperty, ScopeId, assemble,
        setup::COMPONENT_BINDING,
    },
    runtime::Runtime,
    targets::ssr::{SsrError, emit_scoped_template},
    write::{EmitDocument, LinkSink, NoLinks, Recorded},
};

/// Scope identity is a checked compiler option, never a lower source fact.
#[derive(Debug, Clone, Copy)]
pub struct NativeScopedSsrSfcCompileOptions<'o> {
    pub descriptor: DescriptorOptions,
    pub filename: &'o str,
    pub runtime_version: &'o str,
    pub source_map: bool,
    pub style_trim: bool,
    pub scope_id: Option<&'o str>,
}
impl Default for NativeScopedSsrSfcCompileOptions<'static> {
    fn default() -> Self {
        let ssr = NativeSsrSfcCompileOptions::default();
        Self {
            descriptor: ssr.descriptor,
            filename: ssr.filename,
            runtime_version: ssr.runtime_version,
            source_map: false,
            style_trim: false,
            scope_id: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeScopedSsrSfcCompileError {
    Lowering(NativeSelectedSfcIssue),
    Custody,
    Analysis(NativeSsrBuildError),
    Ssr(SsrError),
    Assembly(AssemblyError),
    Style(NativeSfcCompileError),
}

/// Complete SSR module and real optional CSS/maps from the same whole source.
#[derive(Debug)]
pub struct NativeScopedSsrSfcOutput {
    document: EmitDocument,
    source_map: Option<String>,
    css: Option<EmitDocument>,
    css_source_map: Option<String>,
    scope_id: String,
}
impl NativeScopedSsrSfcOutput {
    #[must_use]
    pub fn code(&self) -> &str {
        self.document.as_str()
    }
    #[must_use]
    pub fn document(&self) -> &EmitDocument {
        &self.document
    }
    #[must_use]
    pub fn source_map(&self) -> Option<&str> {
        self.source_map.as_deref()
    }
    #[must_use]
    pub fn css(&self) -> Option<&str> {
        self.css.as_ref().map(EmitDocument::as_str)
    }
    #[must_use]
    pub fn css_document(&self) -> Option<&EmitDocument> {
        self.css.as_ref()
    }
    #[must_use]
    pub fn css_source_map(&self) -> Option<&str> {
        self.css_source_map.as_deref()
    }
    #[must_use]
    pub fn scope_id(&self) -> &str {
        &self.scope_id
    }
}

/// Normal Descriptor/CSS/parser/selected File beside all output or typed refusal.
///
/// ```compile_fail
/// use vize_atelier_sfc::{NativeScopedSsrSfcCompilation, NativeScopedSsrSfcCompileError};
/// use vize_l1_to_l2::native_file::NativeSelectedScopedSfcObservation;
/// fn forge(observation: NativeSelectedScopedSfcObservation<'_>) {
///     let _ = NativeScopedSsrSfcCompilation { observation, result: Err(NativeScopedSsrSfcCompileError::Custody) };
/// }
/// ```
#[derive(Debug)]
pub struct NativeScopedSsrSfcCompilation<'a> {
    observation: NativeSelectedScopedSfcObservation<'a>,
    result: Result<NativeScopedSsrSfcOutput, NativeScopedSsrSfcCompileError>,
}
impl<'a> NativeScopedSsrSfcCompilation<'a> {
    #[must_use]
    pub fn observation(&self) -> &NativeSelectedScopedSfcObservation<'a> {
        &self.observation
    }
    pub fn result(&self) -> Result<&NativeScopedSsrSfcOutput, NativeScopedSsrSfcCompileError> {
        self.result.as_ref().map_err(|error| *error)
    }
    pub fn into_parts(
        self,
    ) -> (
        NativeSelectedScopedSfcObservation<'a>,
        Result<NativeScopedSsrSfcOutput, NativeScopedSsrSfcCompileError>,
    ) {
        (self.observation, self.result)
    }
}

/// Additive one-bare-scoped-CSS literal `.class:empty` original scriptless family.
/// Existing lower/SSR semantic refusals and authored class/style gates apply.
/// Same validated scope feeds CSS, every existing SSR opening and module metadata.
/// No DOM compiler, second parser/AST walk, legacy fallback or default migration.
#[must_use]
pub fn compile_native_scoped_ssr_sfc<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: NativeScopedSsrSfcCompileOptions<'_>,
) -> NativeScopedSsrSfcCompilation<'a> {
    let observation = lower_selected_scoped_sfc_native(allocator, source, options.descriptor);
    let result = if options.source_map {
        emit::<Recorded>(&observation, options)
    } else {
        emit::<NoLinks>(&observation, options)
    };
    NativeScopedSsrSfcCompilation {
        observation,
        result,
    }
}

fn emit<L: LinkSink>(
    observation: &NativeSelectedScopedSfcObservation<'_>,
    options: NativeScopedSsrSfcCompileOptions<'_>,
) -> Result<NativeScopedSsrSfcOutput, NativeScopedSsrSfcCompileError> {
    use NativeScopedSsrSfcCompileError as Error;
    if let Some(issue) = observation.original().issues().first() {
        return Err(Error::Lowering(*issue));
    }
    let view = observation
        .admitted()
        .ok_or(Error::Custody)?
        .into_scoped_template_view();
    let scope_id = scope::for_styles(view.descriptor(), options.scope_id, options.filename)
        .map_err(Error::Style)?
        .ok_or(Error::Custody)?;
    let scope = ScopeId::new(&scope_id).map_err(Error::Assembly)?;
    let css = styles::emit::<L>(
        view.descriptor(),
        core::slice::from_ref(view.style_syntax()),
        Some(scope),
        options.style_trim,
    )
    .map_err(Error::Style)?;
    let receipt = build_native_scoped_ssr_file_decisions(view).map_err(Error::Analysis)?;
    let mut parts = ModuleParts::for_runtime(
        Runtime::VueServerRenderer,
        options.runtime_version,
        COMPONENT_BINDING,
    )
    .map_err(Error::Assembly)?;
    parts.render = Some(emit_scoped_template::<L>(&receipt, scope).map_err(Error::Ssr)?);
    parts.scope_id = Some(scope);
    parts.placement = RenderPlacement::Function {
        binding: "ssrRender",
        property: RenderProperty::SsrRender,
    };
    let document = assemble(parts).map_err(Error::Assembly)?.into_document();
    let source = observation.original().descriptor().source();
    let source_map = options
        .source_map
        .then(|| document.source_map(options.filename, source));
    let css_source_map = css.as_ref().and_then(|css| {
        options
            .source_map
            .then(|| css.source_map(options.filename, source))
    });
    Ok(NativeScopedSsrSfcOutput {
        document,
        source_map,
        css,
        css_source_map,
        scope_id,
    })
}
