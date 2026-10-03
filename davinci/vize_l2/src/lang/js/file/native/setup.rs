//! Normally owned original setup syntax, attached by the selected owner itself.

use super::{
    NativeRouteState, NativeTemplateFile, NativeTemplateIssue, NativeTemplateIssueKind as Kind,
    NativeTemplateOwner,
};
use crate::file::ScriptUnitId;
use alloc::boxed::Box;
use vize_l1::embed::{
    EmbedSource,
    syntax::{NativeSyntax, ProgramOptions, parse_program_once},
};

mod bindings;
mod identity;
mod prepare;
mod receipt;
pub use receipt::{NativeSelectedSetup, NativeSetupIssue, NativeSetupIssueKind};
#[cfg(test)]
mod tests;

impl<'a> NativeTemplateOwner<'a> {
    /// Parse only this owner's actual selected setup, using its original allocator.
    /// The whole stock observation is parked before declaration admission. There
    /// is no caller syntax/source/allocator, and either setup route consumes the
    /// one selected slot. This grants no whole-SFC or target completion.
    ///
    /// ```compile_fail
    /// use vize_l1::embed::syntax::NativeSyntax;
    /// use vize_l2::lang::js::NativeTemplateOwner;
    /// fn pair<'a>(owner: &mut NativeTemplateOwner<'a>, syntax: NativeSyntax<'a>) {
    ///     owner.parse_setup_program(syntax);
    /// }
    /// ```
    /// ```compile_fail
    /// use vize_l0::Allocator;
    /// use vize_l2::lang::js::NativeTemplateOwner;
    /// fn replace<'a>(owner: &mut NativeTemplateOwner<'a>, arena: &'a Allocator) {
    ///     owner.parse_setup_program(arena);
    /// }
    /// ```
    /// The original parser arena remains borrowed by its normal owner:
    /// ```compile_fail
    /// use vize_l0::{Allocator, config::{VueDialect, VueVersion}};
    /// use vize_l1::{SurfaceParseOptions, container::{Vue, vue::DescriptorOptions}, markup::NativeTemplateComponent};
    /// use vize_l2::lang::js::NativeTemplateOwner;
    /// let arena = Allocator::default();
    /// let source = "<template></template><script setup>const x=1</script>";
    /// let descriptor = Vue.observe_descriptor(&arena, source, DescriptorOptions {
    ///     version: VueVersion::V3, dialect: VueDialect::Vue, template: SurfaceParseOptions::default(),
    /// });
    /// let selected = NativeTemplateComponent::parse_in(&arena, descriptor.admitted().unwrap()).unwrap().unwrap();
    /// let mut owner = NativeTemplateOwner::new(selected).unwrap_or_else(|_| panic!("owner"));
    /// owner.parse_setup_program().unwrap();
    /// drop(arena);
    /// let _ = owner.retained_setup();
    /// ```
    pub fn parse_setup_program(&mut self) -> Result<ScriptUnitId, NativeTemplateIssue> {
        let span = self.selected.component().block().span();
        if !matches!(self.state, NativeRouteState::Scripts) {
            return self.refuse(span, Kind::Interrupted);
        }
        if self.setup.is_some() || self.retained_setup.is_some() {
            return self.refuse(span, Kind::DuplicateProgram);
        }
        if self.selected.ordinary().is_some() {
            return self.refuse(span, Kind::UnsupportedOrdinaryScript);
        }
        if self.selected.has_styles() {
            return self.refuse(span, Kind::UnsupportedStyle);
        }
        let Some(script) = self.selected.setup() else {
            return self.refuse(span, Kind::MissingProgram);
        };
        let block = script.block();
        let lang = script.lang();
        // A caught parser or declaration-walk unwind cannot revive this slot.
        self.state = NativeRouteState::Interrupted;
        let source = match EmbedSource::authored(block.root_source(), block.span()) {
            Ok(source) => source,
            Err(error) => return self.refuse(block.span(), Kind::SetupSource(error)),
        };
        let syntax = parse_program_once(
            self.selected.component().allocator(),
            source,
            ProgramOptions::module(lang),
        );
        self.retained_setup = Some(Box::new(syntax));
        #[cfg(test)]
        tests::after_park();
        let Some(syntax) = self.retained_setup.as_deref() else {
            return self.refuse(block.span(), Kind::MissingProgram);
        };
        let Some(admitted) = syntax.admitted_program() else {
            let issue = NativeTemplateIssue {
                span: block.span(),
                kind: Kind::SetupSyntax(
                    syntax
                        .hole()
                        .unwrap_or(vize_l1::embed::syntax::EmbedHole::UnsupportedShape),
                ),
            };
            self.state = NativeRouteState::Refused(issue);
            return Err(issue);
        };
        self.state = NativeRouteState::Scripts;
        super::program::Parts {
            selected: &self.selected,
            producer: &mut self.producer,
            state: &mut self.state,
            ordinary: &mut self.ordinary,
            setup: &mut self.setup,
        }
        .attach(admitted, true)
    }

    /// Original stock syntax remains available after syntax/admission failure.
    /// This getter alone grants no completed File or setup eligibility.
    #[must_use]
    pub fn retained_setup(&self) -> Option<&NativeSyntax<'a>> {
        self.retained_setup.as_deref()
    }
}

impl<'a> NativeTemplateFile<'a> {
    /// The same normally owned Program, comments and full diagnostics after move.
    #[must_use]
    pub fn retained_setup(&self) -> Option<&NativeSyntax<'a>> {
        self.retained_setup.as_deref()
    }

    /// Join the original Program to this exact normally completed selected File.
    /// The borrowed setup route cannot mint this receipt.
    pub fn setup(&self) -> Result<NativeSelectedSetup<'_, 'a>, NativeSetupIssue> {
        receipt::checked(self)
    }
}
