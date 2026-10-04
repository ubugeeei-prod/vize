//! Normal local ownership; short semantic views never replace their origins.

use vize_l0::{
    Allocator, SourceRoot,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{
        Vue,
        vue::{DescriptorObservation, DescriptorOptions},
    },
    embed::{
        EmbedSource, Lang,
        syntax::{NativeSyntax, ProgramOptions, parse_program_once},
    },
    markup::NativeTemplateComponent,
};
use vize_l2::{
    file::FileArtifact,
    lang::js::{FileProducer, ProgramInput, ProgramScope, VueSetup},
};

use super::{NativeSfcLintRefusal as Refusal, envelope};

/// Actual original Descriptor, selected Component, Program parse and script File.
/// All constructors are private to the configured SFC host; no detached raw
/// parts, caller-selected span/profile or fabricated template File can join it.
///
/// ```compile_fail
/// use vize_patina::native::sfc::NativeSfcLintOwner;
/// let owner = NativeSfcLintOwner {};
/// ```
/// Custody cannot be duplicated or detached from the original source/arena:
/// ```compile_fail
/// use vize_patina::native::sfc::NativeSfcLintOwner;
/// fn cloneable<T: Clone>() {}
/// cloneable::<NativeSfcLintOwner<'static>>();
/// ```
/// ```compile_fail
/// use vize_patina::native::sfc::NativeSfcLintOwner;
/// fn escape_source<'a>(owner: &NativeSfcLintOwner<'a>) -> &'static str {
///     owner.source()
/// }
/// ```
/// ```compile_fail
/// use vize_l1::markup::NativeTemplateComponent;
/// use vize_patina::native::sfc::NativeSfcLintOwner;
/// fn escape_arena<'a>(owner: &NativeSfcLintOwner<'a>)
///     -> &'static NativeTemplateComponent<'static> {
///     owner.template()
/// }
/// ```
pub struct NativeSfcLintOwner<'a> {
    descriptor: DescriptorObservation<'a>,
    template: NativeTemplateComponent<'a>,
    syntax: NativeSyntax<'a>,
    file: FileArtifact<'a>,
}

impl<'a> NativeSfcLintOwner<'a> {
    pub(super) fn parse_in(allocator: &'a Allocator, source: &'a str) -> Result<Self, Refusal> {
        SourceRoot::new(source).map_err(Refusal::Source)?;
        let descriptor = Vue.observe_descriptor(
            allocator,
            source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        envelope::check(&descriptor)?;
        let admitted = descriptor.admitted().map_err(|_| Refusal::SourceMismatch)?;
        let script = admitted.setup().ok_or(Refusal::SourceMismatch)?;
        let block = script.block();
        let syntax = parse_program_once(
            allocator,
            EmbedSource::authored(source, block.span()).map_err(Refusal::Embed)?,
            ProgramOptions::module(Lang::Ts),
        );
        if let Some(comment) = syntax.comments().next() {
            return Err(Refusal::ScriptComment {
                span: comment.authored_span().map_err(Refusal::Embed)?,
            });
        }
        let program = syntax.admitted_program().ok_or(Refusal::ProgramSyntax {
            span: block.span(),
            hole: syntax.hole(),
        })?;
        let mut producer = FileProducer::new(allocator, source).map_err(Refusal::File)?;
        let input = ProgramInput::checked(program, block, script.container_index())
            .map_err(Refusal::Program)?;
        producer
            .program(input, ProgramScope::Nested)
            .map_err(Refusal::Program)?;
        let file = producer
            .finish()
            .map_err(|error| Refusal::File(error.artifact().error))?;
        if !file.is_complete() {
            return Err(Refusal::FileIssues {
                issues: file.issues().to_vec(),
                interruptions: file.interrupted_programs().collect(),
            });
        }
        let template = NativeTemplateComponent::parse_in(allocator, admitted)
            .map_err(Refusal::Parse)?
            .ok_or(Refusal::SourceMismatch)?;
        super::super::admission::selected_component(&template).map_err(Refusal::Template)?;
        let owner = Self {
            descriptor,
            template,
            syntax,
            file,
        };
        owner.setup()?;
        Ok(owner)
    }

    #[must_use]
    pub fn source(&self) -> &'a str {
        self.descriptor.source()
    }
    #[must_use]
    pub fn descriptor(&self) -> &DescriptorObservation<'a> {
        &self.descriptor
    }
    #[must_use]
    pub fn template(&self) -> &NativeTemplateComponent<'a> {
        &self.template
    }
    #[must_use]
    pub fn syntax(&self) -> &NativeSyntax<'a> {
        &self.syntax
    }
    /// Script-only File; this is never native template/DOM/ref authority.
    #[must_use]
    pub fn file(&self) -> &FileArtifact<'a> {
        &self.file
    }

    /// Reborrow only this owner's original selections and actual Program/File.
    pub fn setup(&self) -> Result<NativeSfcSetup<'_, 'a>, Refusal> {
        let descriptor = self
            .descriptor
            .admitted()
            .map_err(|_| Refusal::SourceMismatch)?;
        let script = descriptor.setup().ok_or(Refusal::SourceMismatch)?;
        let selected = self.template.setup().ok_or(Refusal::SourceMismatch)?;
        let template = descriptor.template().ok_or(Refusal::SourceMismatch)?;
        if !core::ptr::eq(
            self.source(),
            self.template.component().block().root_source(),
        ) || template.container_index() != self.template.template_index()
            || template.block().span() != self.template.component().block().span()
            || !core::ptr::eq(
                template.block().source(),
                self.template.component().block().source(),
            )
            || selected.container_index() != script.container_index()
            || selected.block().span() != script.block().span()
            || !core::ptr::eq(selected.block().source(), script.block().source())
            || selected.role() != script.role()
            || selected.lang() != script.lang()
        {
            return Err(Refusal::SourceMismatch);
        }
        let program = self
            .syntax
            .admitted_program()
            .ok_or(Refusal::ProgramSyntax {
                span: script.block().span(),
                hole: self.syntax.hole(),
            })?;
        let setup = VueSetup::checked(&self.file, script, program).map_err(Refusal::Setup)?;
        Ok(NativeSfcSetup { owner: self, setup })
    }
}

/// One short capability joins every original owner, rather than a same-buffer
/// raw VueSetup or standalone Program supplied by a caller.
///
/// ```compile_fail
/// use vize_patina::native::sfc::NativeSfcSetup;
/// let view = NativeSfcSetup {};
/// ```
/// ```compile_fail
/// use vize_patina::native::sfc::NativeSfcLintOwner;
/// fn discard<'a>(owner: NativeSfcLintOwner<'a>) {
///     let setup = owner.setup().unwrap();
///     drop(owner);
///     let _ = setup.semantic();
/// }
/// ```
pub struct NativeSfcSetup<'o, 'a> {
    owner: &'o NativeSfcLintOwner<'a>,
    setup: VueSetup<'o, 'o, 'o, 'a>,
}

impl<'o, 'a> NativeSfcSetup<'o, 'a> {
    #[must_use]
    pub fn owner(&self) -> &'o NativeSfcLintOwner<'a> {
        self.owner
    }
    #[must_use]
    pub fn semantic(&self) -> &VueSetup<'o, 'o, 'o, 'a> {
        &self.setup
    }
}
