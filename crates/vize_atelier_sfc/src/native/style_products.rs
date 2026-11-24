//! A stylesheet request depends on the original block descriptor and style
//! bodies. Its result can be published while a script or template is incomplete.
use super::{NativeScssSession, NativeSfcCompileError, NativeSfcCompileOptions, scope, styles};
use crate::scss::ScssProducts;
use vize_l0::String;
use vize_l1::{
    container::vue::{DescriptorIssueCode, L1Ir},
    css::StyleSyntax,
};
use vize_l4::{
    module::ScopeId,
    write::{EmitDocument, NoLinks, Recorded},
};

#[derive(Debug, Clone, Copy)]
pub struct NativeStyleOptions<'a> {
    pub filename: &'a str,
    pub scope_id: Option<&'a str>,
    pub trim: bool,
    pub source_map: bool,
}

impl<'a> From<NativeSfcCompileOptions<'a>> for NativeStyleOptions<'a> {
    fn from(options: NativeSfcCompileOptions<'a>) -> Self {
        Self {
            filename: options.filename,
            scope_id: options.scope_id,
            trim: options.style_trim,
            source_map: options.source_map,
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NativeStyleOutput {
    document: Option<EmitDocument>,
    source_map: Option<String>,
    scope_id: Option<String>,
}

impl NativeStyleOutput {
    pub fn css(&self) -> Option<&str> {
        self.document.as_ref().map(EmitDocument::as_str)
    }
    pub fn css_document(&self) -> Option<&EmitDocument> {
        self.document.as_ref()
    }
    pub fn css_source_map(&self) -> Option<&str> {
        self.source_map.as_deref()
    }
    pub fn scope_id(&self) -> Option<&str> {
        self.scope_id.as_deref()
    }
}

/// Compile styles directly from an original L1 descriptor. The owner supplies
/// every block and mapping coordinate. Prepared SCSS must belong to this source.
pub fn compile_native_styles<'p, 's: 'p>(
    owner: &L1Ir<'_>,
    options: NativeStyleOptions<'_>,
    prepared: impl IntoIterator<Item = &'p ScssProducts<'s>>,
) -> Result<NativeStyleOutput, NativeSfcCompileError> {
    emit(owner, None, options, &mut prepared.into_iter(), true)
}

impl NativeScssSession<'_, '_> {
    /// Reuse this session's original observations when only the style output is requested.
    pub fn emit_styles<'p, 's: 'p>(
        &self,
        prepared: impl IntoIterator<Item = &'p ScssProducts<'s>>,
    ) -> Result<NativeStyleOutput, NativeSfcCompileError> {
        let owner = self.observation.descriptor();
        let syntax = owner
            .admitted()
            .is_ok()
            .then(|| self.observation.style_syntax());
        emit(
            owner,
            syntax,
            self.options.into(),
            &mut prepared.into_iter(),
            self.scoped,
        )
    }
}

fn emit<'p, 's: 'p>(
    owner: &L1Ir<'_>,
    retained: Option<&[StyleSyntax<'_>]>,
    options: NativeStyleOptions<'_>,
    prepared: &mut dyn Iterator<Item = &'p ScssProducts<'s>>,
    scoped_scss: bool,
) -> Result<NativeStyleOutput, NativeSfcCompileError> {
    let descriptor = owner.admitted_styles().map_err(|refusal| {
        refusal
            .issues()
            .iter()
            .find_map(|issue| {
                let index = issue.container_index?;
                (issue.code == DescriptorIssueCode::ExternalSource
                    && owner.container().blocks.get(index)?.name == "style")
                    .then_some(NativeSfcCompileError::ExternalBlock {
                        container_index: index,
                        span: issue.span,
                    })
            })
            .unwrap_or(NativeSfcCompileError::Descriptor)
    })?;
    // Prepared SCSS has its own lowering; unscoped CSS is copied verbatim.
    // Borrow retained syntax when present, including an empty retained slice.
    let observed;
    let syntax = match retained {
        Some(syntax) => syntax,
        None => {
            observed = descriptor
                .styles()
                .filter(|style| {
                    style.attrs().iter().any(|attr| attr.name == "scoped")
                        && style.attrs().iter().all(|attr| {
                            attr.name != "lang" || styles::is_css_lang_value(attr.value)
                        })
                })
                .map(StyleSyntax::observe)
                .collect::<Vec<_>>();
            &observed
        }
    };
    let scope_id = scope::for_styles(descriptor, options.scope_id, options.filename)?;
    let scope = scope_id
        .as_deref()
        .map(ScopeId::new)
        .transpose()
        .map_err(NativeSfcCompileError::Assembly)?;
    let writer = if options.source_map {
        styles::emit::<Recorded>
    } else {
        styles::emit::<NoLinks>
    };
    let document = writer(
        descriptor,
        syntax,
        scope,
        options.trim,
        Some(prepared),
        scoped_scss,
    )?;
    let source_map = document
        .as_ref()
        .filter(|_| options.source_map)
        .map(|doc| doc.source_map(options.filename, descriptor.source()));
    Ok(NativeStyleOutput {
        document,
        source_map,
        scope_id,
    })
}
