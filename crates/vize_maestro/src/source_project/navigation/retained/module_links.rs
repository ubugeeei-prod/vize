//! Complete original operands remain local until one sealed owned reply is ready.

use super::super::{SnapshotRefusal, worker::module_links::Request};
use super::{Control, NavigationRefusal, RetainedNavigation, coordinates};
use tower_lsp::lsp_types::{DocumentLink, Range, Url};
use vize_l0::{SourceBlock, Span, String};
use vize_l1::embed::syntax::NativeSyntax;
use vize_l2::{
    file::FileArtifact,
    lang::js::{ModuleSourceError, OriginalModuleSources, ProgramInput, ProgramInputError},
};

pub(super) type Admission<'f, 'p, 'a> = Result<RetainedSources<'f, 'p, 'a>, ModuleOperandRefusal>;

/// The empty-operand case must retain the same original File join too.
/// Fields are private; only the normally owned factory below pairs this view.
pub(super) struct RetainedSources<'f, 'p, 'a> {
    file: &'f FileArtifact<'a>,
    view: OriginalModuleSources<'f, 'p, 'a>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(in crate::source_project) enum ModuleOperandRefusal {
    Navigation(NavigationRefusal),
    Input(ProgramInputError),
    Sources(ModuleSourceError),
    TargetCardinality { operands: usize, targets: usize },
}

#[derive(Debug, PartialEq, Eq)]
struct Operand {
    quoted: Span,
    range: Range,
    request: String,
}

/// Constructed only from the retained worker's original complete L2 view.
/// This owns response operands, never another Program, File or target authority.
#[derive(Debug, PartialEq, Eq)]
pub(in crate::source_project) struct ModuleOperands {
    operands: Vec<Operand>,
}

impl ModuleOperands {
    pub(in crate::source_project) fn len(&self) -> usize {
        self.operands.len()
    }

    pub(in crate::source_project) fn is_empty(&self) -> bool {
        self.operands.is_empty()
    }

    pub(in crate::source_project) fn decoded_requests(&self) -> impl Iterator<Item = &str> {
        self.operands.iter().map(|operand| operand.request.as_str())
    }

    /// Target authority is supplied by the same ready query's checked batch.
    /// A mismatched batch refuses before producing any links; no zip truncation.
    pub(in crate::source_project) fn into_links(
        self,
        targets: &[Url],
    ) -> Result<Vec<DocumentLink>, ModuleOperandRefusal> {
        if self.len() != targets.len() {
            return Err(ModuleOperandRefusal::TargetCardinality {
                operands: self.len(),
                targets: targets.len(),
            });
        }
        Ok(self
            .operands
            .into_iter()
            .zip(targets)
            .map(|(operand, target)| DocumentLink {
                range: operand.range,
                target: Some(target.clone()),
                tooltip: None,
                data: None,
            })
            .collect())
    }
}

pub(super) fn admit<'f, 'p, 'a>(
    syntax: &'p NativeSyntax<'a>,
    block: SourceBlock<'a>,
    file: &'f FileArtifact<'a>,
) -> Admission<'f, 'p, 'a> {
    let admitted = syntax
        .admitted_program()
        .ok_or(ModuleOperandRefusal::Navigation(NavigationRefusal::Syntax))?;
    // The original decoder retains strict-module violations without changing
    // neutral syntax admission. This consumer must refuse that same receipt.
    if admitted.has_legacy_literals() {
        return Err(ModuleOperandRefusal::Navigation(NavigationRefusal::Syntax));
    }
    let input = ProgramInput::checked(admitted, block, 0).map_err(ModuleOperandRefusal::Input)?;
    let view = file
        .original_module_sources(input)
        .map_err(ModuleOperandRefusal::Sources)?;
    Ok(RetainedSources { file, view })
}

pub(super) fn answer(
    query: &RetainedNavigation<'_, '_>,
    admission: Option<&Admission<'_, '_, '_>>,
    request: Request,
    control: &Control,
) {
    if request.reply.is_canceled() {
        return;
    }
    #[cfg(test)]
    control
        .queries
        .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let result = match admission {
        Some(Ok(view)) => collect(query, view, || {
            control.retired() || request.reply.is_canceled()
        }),
        Some(Err(refusal)) => Err(refusal.clone()),
        None => Err(ModuleOperandRefusal::Navigation(
            NavigationRefusal::Language,
        )),
    };
    let _ = request.reply.send(result);
}

fn collect(
    query: &RetainedNavigation<'_, '_>,
    retained: &RetainedSources<'_, '_, '_>,
    cancelled: impl Fn() -> bool,
) -> Result<ModuleOperands, ModuleOperandRefusal> {
    if !core::ptr::eq(retained.file, query.file)
        || !core::ptr::eq(retained.file.artifact().source(), query.snapshot.source())
    {
        return Err(ModuleOperandRefusal::Navigation(
            NavigationRefusal::Projection,
        ));
    }
    let mut operands = Vec::new();
    let mut refusal = None;
    // Every callback receives a sealed actual occurrence. L2 refusal wins over
    // its visited prefix, even when a consumer projection has already refused.
    retained
        .view
        .for_each(|original| {
            if refusal.is_some() {
                return;
            }
            if cancelled() {
                refusal = Some(ModuleOperandRefusal::Navigation(NavigationRefusal::Host(
                    SnapshotRefusal::Cancelled,
                )));
                return;
            }
            if !core::ptr::eq(original.file(), query.file)
                || !core::ptr::eq(original.file().artifact().source(), query.snapshot.source())
            {
                refusal = Some(ModuleOperandRefusal::Navigation(
                    NavigationRefusal::Projection,
                ));
                return;
            }
            let quoted = original.quoted_span();
            match coordinates::range(query.snapshot.source(), query.lines, quoted) {
                Ok(range) => operands.push(Operand {
                    quoted,
                    range,
                    request: String::from(original.decoded_request()),
                }),
                Err(error) => refusal = Some(ModuleOperandRefusal::Navigation(error)),
            }
        })
        .map_err(ModuleOperandRefusal::Sources)?;
    if cancelled() {
        return Err(ModuleOperandRefusal::Navigation(NavigationRefusal::Host(
            SnapshotRefusal::Cancelled,
        )));
    }
    if let Some(refusal) = refusal {
        return Err(refusal);
    }
    // L2 iteration order is unspecified. Preserve distinct equal requests;
    // only repeated rows of the same original occurrence can collapse.
    operands.sort_unstable_by(|left, right| {
        (left.quoted.start, left.quoted.end).cmp(&(right.quoted.start, right.quoted.end))
    });
    operands.dedup_by(|left, right| left.quoted == right.quoted);
    Ok(ModuleOperands { operands })
}

#[cfg(test)]
mod tests;
