//! Explicit native SFC entry for bounded scriptless, setup and empty-option DOM.
//!
//! Original descriptor, syntax, diagnostics and partial File owners are always
//! retained. Admission and target refusals never select a legacy compiler.

use vize_l0::{
    Allocator, Span, String,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    container::vue::{DescriptorOptions, ScriptRole},
    css::StyleIssue,
    embed::Lang,
    markup::SurfaceParseOptions,
};
use vize_l1_to_l2::native_file::{NativeSfcObservation, lower_sfc_native};
use vize_l2::lang::js::{OrdinaryIssue, SetupIssue};
use vize_l3::decision::{DecisionBuildError, build_dom_file_decisions};
use vize_l4::{
    module::{
        AssemblyError, ModuleParts, RenderFunctionName, assemble,
        ordinary::OrdinaryEmitError,
        setup::{COMPONENT_BINDING, SetupEmitError},
    },
    runtime::Runtime,
    targets::dom::{DomError, emit_file},
    write::{EmitDocument, LinkSink, NoLinks, Recorded},
};

mod ordinary;
mod setup;

/// Explicit native parsing/runtime policy; unsupported values remain refusals.
#[derive(Debug, Clone, Copy)]
pub struct NativeSfcCompileOptions<'o> {
    pub descriptor: DescriptorOptions,
    pub filename: &'o str,
    pub runtime_version: &'o str,
    pub source_map: bool,
    /// Trim each emitted plain CSS block; the ordinary SFC default is false.
    pub style_trim: bool,
    /// Full `data-v-*` scope identity. None uses the ordinary filename hash.
    pub scope_id: Option<&'o str>,
}

impl Default for NativeSfcCompileOptions<'static> {
    fn default() -> Self {
        Self {
            descriptor: DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
            filename: "anonymous.vue",
            runtime_version: "3.5.35",
            source_map: false,
            style_trim: false,
            scope_id: None,
        }
    }
}

/// The exact stage that refused this original whole-file observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeSfcCompileError {
    ExternalBlock {
        container_index: usize,
        span: Span,
    },
    Descriptor,
    ScriptCompilationUnavailable {
        container_index: usize,
        span: Span,
    },
    ScriptSetup(SetupIssue),
    SetupEmission(SetupEmitError),
    ScriptOrdinary(OrdinaryIssue),
    OrdinaryEmission(OrdinaryEmitError),
    StyleCompilationUnavailable {
        container_index: usize,
        span: Span,
    },
    StyleBindSyntaxUnproven {
        container_index: usize,
        span: Span,
    },
    ScopedStyleUnavailable {
        container_index: usize,
        issue: StyleIssue,
    },
    MissingTemplate,
    Orchestration,
    Analysis(DecisionBuildError),
    Dom(DomError),
    Assembly(AssemblyError),
    #[cfg(feature = "native-sfc-scss")]
    UnusedPreparedScss {
        span: Span,
    },
    #[cfg(feature = "native-sfc-scss")]
    Scss {
        container_index: usize,
        issue: vize_l2::lang::scss::ScssRefusal,
    },
}

/// Complete emitted module and its full-file links, with an optional v3 map.
#[derive(Debug)]
pub struct NativeSfcOutput {
    document: EmitDocument,
    source_map: Option<String>,
    css: Option<EmitDocument>,
    css_source_map: Option<String>,
    scope_id: Option<String>,
}

impl NativeSfcOutput {
    #[must_use]
    pub fn scope_id(&self) -> Option<&str> {
        self.scope_id.as_deref()
    }
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
}

/// The complete original parse/File custody beside its whole output or refusal.
/// Fields and constructors are private; callers cannot pair arbitrary outputs.
///
/// ```compile_fail
/// use vize_atelier_sfc::{NativeSfcCompilation, NativeSfcCompileError};
/// use vize_l1_to_l2::native_file::NativeSfcObservation;
/// fn forge(observation: NativeSfcObservation<'_>) {
///     let _ = NativeSfcCompilation { observation, result: Err(NativeSfcCompileError::Descriptor) };
/// }
/// ```
#[derive(Debug)]
pub struct NativeSfcCompilation<'a> {
    observation: NativeSfcObservation<'a>,
    result: Result<NativeSfcOutput, NativeSfcCompileError>,
}

impl<'a> NativeSfcCompilation<'a> {
    #[must_use]
    pub fn observation(&self) -> &NativeSfcObservation<'a> {
        &self.observation
    }
    pub fn result(&self) -> Result<&NativeSfcOutput, NativeSfcCompileError> {
        self.result.as_ref().map_err(|error| *error)
    }
    pub fn into_parts(
        self,
    ) -> (
        NativeSfcObservation<'a>,
        Result<NativeSfcOutput, NativeSfcCompileError>,
    ) {
        (self.observation, self.result)
    }
}

/// Compile through the genuine admitted SFC/File, L3 and L4 owner chain.
///
/// This additive entry supports scriptless static structure and retained
/// literals, original JS/TS setup let/var/const primitive declarations plus empty
/// statements, sole ordinary JS/TS direct empty default objects, and plain CSS
/// without unproven binding syntax, plus one simple-class rule, literal class
/// list or literal `.class:empty` rule per scoped block. Broader scripts/scoped/
/// module/preprocessor styles, custom/external blocks, unsupported
/// profiles and unavailable native target semantics return typed refusals.
/// Every original observation survives, and no partial module is returned.
#[must_use]
pub fn compile_native_sfc<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: NativeSfcCompileOptions<'_>,
) -> NativeSfcCompilation<'a> {
    let observation = lower_sfc_native(allocator, source, options.descriptor);
    let result = if options.source_map {
        emit::<Recorded>(
            &observation,
            options,
            #[cfg(feature = "native-sfc-scss")]
            None,
            #[cfg(feature = "native-scoped-scss")]
            false,
        )
    } else {
        emit::<NoLinks>(
            &observation,
            options,
            #[cfg(feature = "native-sfc-scss")]
            None,
            #[cfg(feature = "native-scoped-scss")]
            false,
        )
    };
    NativeSfcCompilation {
        observation,
        result,
    }
}

fn emit<L: LinkSink>(
    observation: &NativeSfcObservation<'_>,
    options: NativeSfcCompileOptions<'_>,
    #[cfg(feature = "native-sfc-scss")] scss: Option<
        &mut dyn Iterator<Item = &crate::scss::ScssProducts<'_>>,
    >,
    #[cfg(feature = "native-scoped-scss")] scoped_scss: bool,
) -> Result<NativeSfcOutput, NativeSfcCompileError> {
    // Refusal classification reads retained block metadata, never source text
    // or an external file. It grants no descriptor or compilation authority.
    for (container_index, block) in observation
        .descriptor()
        .container()
        .blocks
        .iter()
        .enumerate()
    {
        if let Some(attribute) = block
            .attrs
            .iter()
            .find(|attribute| attribute.name.eq_ignore_ascii_case("src"))
        {
            return Err(NativeSfcCompileError::ExternalBlock {
                container_index,
                span: attribute.span,
            });
        }
    }
    let descriptor = observation
        .descriptor()
        .admitted()
        .map_err(|_| NativeSfcCompileError::Descriptor)?;
    let scope_id = scope::for_styles(descriptor, options.scope_id, options.filename)?;
    let scope = scope_id
        .as_deref()
        .map(vize_l4::module::ScopeId::new)
        .transpose()
        .map_err(NativeSfcCompileError::Assembly)?;
    let css = styles::emit::<L>(
        descriptor,
        observation.style_syntax(),
        scope,
        options.style_trim,
        #[cfg(feature = "native-sfc-scss")]
        scss,
        #[cfg(feature = "native-scoped-scss")]
        scoped_scss,
    )?;
    if let Some(script) = observation.scripts().first()
        && (observation.scripts().len() != 1
            || !(script.role() == ScriptRole::Setup
                || (script.role() == ScriptRole::Ordinary
                    && descriptor.setup().is_none()
                    && observation.admitted().is_some_and(|admitted| {
                        admitted.file().file().ordinary_empty_script().is_some()
                    })))
            || !matches!(script.lang(), Lang::Js | Lang::Ts)
            || observation.admitted().is_none()
            || !script.syntax().is_some_and(|syntax| {
                syntax
                    .admitted_program()
                    .is_some_and(|program| !program.program().body.is_empty())
            }))
    {
        return Err(NativeSfcCompileError::ScriptCompilationUnavailable {
            container_index: script.container_index(),
            span: script.block().span(),
        });
    }
    if observation.template().is_none() {
        return Err(NativeSfcCompileError::MissingTemplate);
    }
    let admitted = observation
        .admitted()
        .ok_or(NativeSfcCompileError::Orchestration)?;
    let mut parts =
        ModuleParts::for_runtime(Runtime::VueDom, options.runtime_version, COMPONENT_BINDING)
            .map_err(NativeSfcCompileError::Assembly)?;
    if observation.scripts().is_empty() {
        let analysis = build_dom_file_decisions(admitted.file().file())
            .map_err(NativeSfcCompileError::Analysis)?;
        parts.render = Some(emit_file::<L>(&analysis).map_err(NativeSfcCompileError::Dom)?);
    } else if descriptor.ordinary().is_some() {
        ordinary::emit(&admitted, &mut parts)?;
    } else {
        setup::emit(&admitted, &mut parts)?;
    }
    parts.scope_id = scope;
    parts.placement = RenderFunctionName::Render.placement();
    let document = assemble(parts)
        .map_err(NativeSfcCompileError::Assembly)?
        .into_document();
    let source_map = options
        .source_map
        .then(|| document.source_map(options.filename, observation.descriptor().source()));
    let css_source_map = css.as_ref().and_then(|document| {
        options
            .source_map
            .then(|| document.source_map(options.filename, observation.descriptor().source()))
    });
    Ok(NativeSfcOutput {
        document,
        source_map,
        css,
        css_source_map,
        scope_id,
    })
}

#[cfg(test)]
mod tests;

#[cfg(test)]
mod class_list_tests;
#[cfg(test)]
mod css_tests;
pub(crate) mod scope;
mod scoped;
#[cfg(test)]
mod scoped_empty_tests;
#[cfg(test)]
mod scoped_tests;
#[cfg(test)]
mod style_tests;
pub(crate) mod styles;

/// Compile a DOM component including unscoped inline SCSS in the supported
/// original grammar. Scripts, template and CSS maps refer to this same file.
#[cfg(feature = "native-sfc-scss")]
#[must_use]
pub fn compile_native_sfc_with_scss<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: NativeSfcCompileOptions<'_>,
) -> NativeSfcCompilation<'a> {
    let session = NativeScssSession::new(allocator, source, options);
    let result = session.emit([]);
    NativeSfcCompilation {
        observation: session.observation,
        result,
    }
}

/// A single original component observation shared by the compiler and tool passes.
#[cfg(feature = "native-sfc-scss")]
pub struct NativeScssSession<'a, 'o> {
    observation: NativeSfcObservation<'a>,
    options: NativeSfcCompileOptions<'o>,
    #[cfg(feature = "native-scoped-scss")]
    scoped: bool,
}

#[cfg(feature = "native-sfc-scss")]
impl<'a, 'o> NativeScssSession<'a, 'o> {
    pub fn new(
        allocator: &'a Allocator,
        source: &'a str,
        options: NativeSfcCompileOptions<'o>,
    ) -> Self {
        Self {
            observation: lower_sfc_native(allocator, source, options.descriptor),
            options,
            #[cfg(feature = "native-scoped-scss")]
            scoped: false,
        }
    }

    pub fn observation(&self) -> &NativeSfcObservation<'a> {
        &self.observation
    }

    /// Also emit original scoped SCSS through the admitted selector grammar.
    #[cfg(feature = "native-scoped-scss")]
    #[must_use]
    pub fn with_scoped_styles(mut self) -> Self {
        self.scoped = true;
        self
    }

    /// Reuse the supplied style passes in authored order. Other inline SCSS
    /// is lowered on demand. A foreign or unused pass cannot produce a module.
    pub fn emit<'p, 's: 'p>(
        &self,
        prepared: impl IntoIterator<Item = &'p crate::scss::ScssProducts<'s>>,
    ) -> Result<NativeSfcOutput, NativeSfcCompileError> {
        let mut styles = prepared.into_iter();
        if self.options.source_map {
            emit::<Recorded>(
                &self.observation,
                self.options,
                Some(&mut styles),
                #[cfg(feature = "native-scoped-scss")]
                self.scoped,
            )
        } else {
            emit::<NoLinks>(
                &self.observation,
                self.options,
                Some(&mut styles),
                #[cfg(feature = "native-scoped-scss")]
                self.scoped,
            )
        }
    }
}
