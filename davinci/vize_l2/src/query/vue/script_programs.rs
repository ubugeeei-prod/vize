use super::{VueDemand, VueL2IrProducer, VueProgramError};
use vize_l0::Allocator;
use vize_l1::container::vue::{ScriptPrograms, ScriptRole};

impl<'a> VueL2IrProducer<'a> {
    /// Attach Vue observers to the Programs retained by this descriptor. Each
    /// present script supplies lexical facts even if no framework observer was
    /// requested for its role. Other tools can borrow the same L1 syntax.
    ///
    /// # Errors
    /// Returns the original producer error for the first unavailable Program
    /// or invalid file. A malformed ordinary script is reported first. Both
    /// roles retain their original L1 syntax and diagnostics for other tools.
    #[inline]
    pub fn from_script_programs(
        arena: &'a Allocator,
        programs: ScriptPrograms<'_, '_, 'a>,
        demand: VueDemand,
    ) -> Result<Self, VueProgramError> {
        let admitted = |role| {
            programs
                .get(role)
                .map(|syntax| {
                    syntax
                        .admitted_program()
                        .ok_or(VueProgramError::Missing(role))
                })
                .transpose()
        };
        Self::scripts(
            arena,
            programs.descriptor(),
            admitted(ScriptRole::Ordinary)?,
            admitted(ScriptRole::Setup)?,
            demand,
        )
    }
}
