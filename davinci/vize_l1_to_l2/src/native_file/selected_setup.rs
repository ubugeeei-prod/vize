//! Whole original descriptor custody with one genuinely owned setup Program.

use super::{
    NativeSelectedSfcObservation,
    selected::{self, SelectedMode},
};
use vize_l0::Allocator;
use vize_l1::{container::vue::DescriptorOptions, embed::Lang, markup::NativeTemplateGrammar};
use vize_l2::lang::js::{NativeSelectedSetup, NativeTemplateView};

#[cfg(test)]
mod tests;

/// Normal original Descriptor, selected File and whole setup syntax stay in the
/// same sealed owner. The scriptless observation remains a separate admission.
///
/// ```compile_fail
/// use vize_l1_to_l2::native_file::{NativeSelectedSfcObservation, NativeSelectedSetupSfcObservation};
/// fn pair(original: NativeSelectedSfcObservation<'_>) {
///     let _ = NativeSelectedSetupSfcObservation { original };
/// }
/// ```
/// ```compile_fail
/// use vize_l1_to_l2::native_file::NativeSelectedSetupSfcObservation;
/// fn alter(owner: &mut NativeSelectedSetupSfcObservation<'_>) { owner.original.template.take(); }
/// ```
pub struct NativeSelectedSetupSfcObservation<'a> {
    original: NativeSelectedSfcObservation<'a>,
}
impl core::fmt::Debug for NativeSelectedSetupSfcObservation<'_> {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        formatter
            .debug_struct("NativeSelectedSetupSfcObservation")
            .field("original", &self.original)
            .finish()
    }
}
impl<'a> NativeSelectedSetupSfcObservation<'a> {
    /// Full original descriptor and every retained template/program refusal.
    #[must_use]
    pub fn original(&self) -> &NativeSelectedSfcObservation<'a> {
        &self.original
    }

    /// Join this whole original descriptor to its normal selected File/setup.
    /// No source, Program, File, unit or role is provided by the caller.
    #[must_use]
    pub fn admitted(&self) -> Option<NativeSelectedSetupSfc<'_, 'a>> {
        if !self.original.issues().is_empty() {
            return None;
        }
        let descriptor = self.original.descriptor().admitted().ok()?;
        if descriptor.ordinary().is_some() || descriptor.styles().len() != 0 {
            return None;
        }
        let script = descriptor.setup()?;
        let template = descriptor.template()?;
        let original = self.original.template()?;
        let selected = original.selected();
        let block = selected.component().block();
        let setup = original.setup().ok()?;
        let view = original.view().ok()?;
        let file = view.file()?;
        let expected = match descriptor.template_lang() {
            Lang::Js => NativeTemplateGrammar::JavaScriptModule,
            Lang::Ts => NativeTemplateGrammar::TypeScriptModule,
        };
        if selected.ordinary().is_some()
            || selected.has_styles()
            || selected.grammar() != expected
            || selected.template_index() != template.container_index()
            || block.span() != template.block().span()
            || !core::ptr::eq(block.source(), template.block().source())
            || !core::ptr::eq(block.root_source(), descriptor.source())
            || !core::ptr::eq(file.artifact().source(), descriptor.source())
            || !core::ptr::eq(setup.file(), file)
            || !core::ptr::eq(setup.owner(), original)
            || setup.selected().container_index() != script.container_index()
            || setup.selected().block().span() != script.block().span()
            || setup.selected().lang() != script.lang()
            || setup.selected().role() != script.role()
            || !core::ptr::eq(setup.selected().block().source(), script.block().source())
            || !core::ptr::eq(setup.syntax().source().authored_root(), descriptor.source())
            || !core::ptr::eq(setup.program().source(), script.block().source())
            || setup.unit().index() as usize != script.container_index()
            || setup.unit_record().span != script.block().span()
        {
            return None;
        }
        Some(NativeSelectedSetupSfc {
            observation: self,
            template: view,
            setup,
        })
    }
}

/// The original whole envelope remains borrowed while both genuine views live.
/// This provides no runtime read spelling, setup emitter or product default.
///
/// ```compile_fail
/// use vize_l1_to_l2::native_file::NativeSelectedSetupSfcObservation;
/// fn discard(owner: NativeSelectedSetupSfcObservation<'_>) {
///     let view = owner.admitted().unwrap();
///     drop(owner);
///     let _ = view.into_template_view();
/// }
/// ```
/// ```compile_fail
/// use vize_l1_to_l2::native_file::NativeSelectedSetupSfc;
/// fn copy(view: NativeSelectedSetupSfc<'_, '_>) { let _ = view.clone(); }
/// ```
/// ```compile_fail
/// use vize_l1_to_l2::native_file::NativeSelectedSetupSfc;
/// fn forge() { let _ = NativeSelectedSetupSfc { setup: true }; }
/// ```
/// ```compile_fail
/// use vize_l1_to_l2::native_file::NativeSelectedSetupSfcObservation;
/// use vize_l2::lang::js::NativeSelectedSetup;
/// fn separate<'a>(owner: NativeSelectedSetupSfcObservation<'a>) -> NativeSelectedSetup<'a, 'a> {
///     owner.original().template().unwrap().setup().unwrap()
/// }
/// ```
pub struct NativeSelectedSetupSfc<'owner, 'arena> {
    observation: &'owner NativeSelectedSetupSfcObservation<'arena>,
    template: NativeTemplateView<'owner, 'arena>,
    setup: NativeSelectedSetup<'owner, 'arena>,
}
impl<'owner, 'arena> NativeSelectedSetupSfc<'owner, 'arena> {
    #[must_use]
    pub fn observation(&self) -> &'owner NativeSelectedSetupSfcObservation<'arena> {
        self.observation
    }
    #[must_use]
    pub fn setup(&self) -> &NativeSelectedSetup<'owner, 'arena> {
        &self.setup
    }
    #[must_use]
    pub fn into_template_view(self) -> NativeTemplateView<'owner, 'arena> {
        self.template
    }
}

/// Observe the descriptor once, parse this original setup and template once,
/// gate real setup eligibility before the original root cursor, then retain the
/// whole original owners through its normal end or typed refusal. Other sibling
/// scripts/styles/custom/external/profile families are never omitted.
#[must_use]
pub fn lower_selected_setup_sfc_native<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    options: DescriptorOptions,
) -> NativeSelectedSetupSfcObservation<'a> {
    NativeSelectedSetupSfcObservation {
        original: selected::observe(allocator, source, options, SelectedMode::Setup),
    }
}
