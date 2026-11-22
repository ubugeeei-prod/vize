//! An imported component request uses the original descriptor, Program and L2 operations.
use super::{NativeSfcIssueKind, NativeSfcObservation};
use smallvec::SmallVec;
use vize_l2::query::{ComponentImportIssue, ImportedComponentSite, Model};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComponentImportRefusal {
    SourceAssembly,
    Component(ComponentImportIssue),
}

/// Static tag sites and their imported declarations, borrowed from one source revision.
pub struct SetupComponentImports<'o, 'a> {
    source: &'a str,
    sites: SmallVec<[ImportedComponentSite<'o, 'a>; 2]>,
}

impl<'o, 'a> SetupComponentImports<'o, 'a> {
    /// Original file text shared by tag spans and import declarations.
    #[must_use]
    pub fn source(&self) -> &'a str {
        self.source
    }

    /// Authored opening tags in source order.
    #[must_use]
    pub fn sites(&self) -> &[ImportedComponentSite<'o, 'a>] {
        &self.sites
    }

    /// Locate an opening tag's name using a file-absolute UTF-8 byte offset.
    #[must_use]
    pub fn at_offset(&self, offset: u32) -> Option<&ImportedComponentSite<'o, 'a>> {
        let index = self
            .sites
            .partition_point(|site| site.name_span().end <= offset);
        self.sites
            .get(index)
            .filter(|site| site.name_span().start <= offset)
    }
}

impl<'a> NativeSfcObservation<'a> {
    /// Resolve every authored inline component to a direct setup value import.
    /// The product borrows the original declarations and is available only when
    /// the entire request succeeds. General native File admission is independent.
    pub fn setup_component_imports(
        &self,
    ) -> Result<SetupComponentImports<'_, 'a>, ComponentImportRefusal> {
        let source = ComponentImportRefusal::SourceAssembly;
        let descriptor = self.descriptor().admitted().map_err(|_| source)?;
        let setup = descriptor.setup().ok_or(source)?;
        let template = descriptor.template().ok_or(source)?;
        let [script] = self.scripts() else {
            return Err(source);
        };
        if descriptor.ordinary().is_some()
            || script.container_index() != setup.container_index()
            || script.role() != setup.role()
            || script.lang() != setup.lang()
            || script.block().span() != setup.block().span()
            || !core::ptr::eq(script.block().source(), setup.block().source())
            || !self
                .template()
                .is_some_and(|t| t.matches(template, descriptor.template_lang()))
        {
            return Err(source);
        }
        let (file, receipt) = if let Some(admitted) = self.admitted() {
            (admitted.file().file(), admitted.file().setup())
        } else {
            let [issue] = self.issues() else {
                return Err(source);
            };
            let rejected = self.rejected_file().ok_or(source)?;
            if issue.kind != NativeSfcIssueKind::FileRejected
                || !rejected.issues().is_empty()
                || rejected.ordinary().is_some()
            {
                return Err(source);
            }
            (rejected.file().ok_or(source)?, rejected.setup())
        };
        let receipt = receipt.ok_or(source)?;
        if script.unit() != Some(receipt.unit()) || receipt.span() != setup.block().span() {
            return Err(source);
        }
        let program = script
            .syntax()
            .and_then(|syntax| syntax.admitted_program())
            .ok_or(source)?;
        let mut sites = SmallVec::new();
        Model::new(file)
            .visit_setup_component_imports(setup, &program, template, |site| sites.push(site))
            .map_err(ComponentImportRefusal::Component)?;
        Ok(SetupComponentImports {
            source: descriptor.source(),
            sites,
        })
    }
}
