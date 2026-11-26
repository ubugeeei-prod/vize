//! Checkable original JS/TS/JSX/TSX Modules, retaining their completed File owner.
//!
//! Vue template projection and the Canon default route remain unfinished
//! (#6849, #6879). This bounded entry never rewrites or reparses a Program.
//!
//! [`project_jsx_module`] projects a JSX/TSX module from the L2 definitions
//! of its markup roots into a [`TsDocument`]. No product route calls it yet.

use crate::write::{EmitDocument, LinkSink, NoLinks, Recorded, Writer};
use vize_l0::Span;
use vize_l2::file::{L2Ir, ScriptUnit};
use vize_l2::lang::js::JsxL2Ir;

#[cfg(feature = "component-projection")]
mod component;
mod control;
mod helpers;
mod mapping;
mod module;
mod ops;
mod output;
pub use helpers::HELPERS;
pub use mapping::MappingError;
pub use module::{JsxModuleRefusal, project_jsx_module};
pub use output::{DirectiveKind, DirectiveRange, LinkInfo, LinkMapping, LinkRole, TsDocument};
pub mod vue;

const MODULE_SUFFIX: &str = "\n;\nexport {};\n";

/// The original admitted parse language, also selecting the checker file kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    JavaScript,
    TypeScript,
    Jsx,
    Tsx,
}

impl SourceKind {
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::JavaScript => "mjs",
            Self::TypeScript => "ts",
            Self::Jsx => "jsx",
            Self::Tsx => "tsx",
        }
    }
}

/// A complete File is required, with one genuine whole-source Module unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProjectionError {
    IncompleteFile,
    ProgramCount,
    PartialSource,
    UnsupportedProfile,
    NestedScope,
    TemplateOperations,
    SourceTooLarge,
}

/// Original Program bytes and a checked document keep the same File borrow.
///
/// A detached document cannot construct a projection:
/// ```compile_fail
/// use vize_l2::file::L2Ir;
/// use vize_l4::{targets::ts::ProgramProjection, write::EmitDocument};
/// fn forge(l2ir: &L2Ir<'_>, document: EmitDocument) {
///     let _ = ProgramProjection { l2ir, document };
/// }
/// ```
/// The actual File cannot be dropped while the projection is live:
/// ```compile_fail
/// use vize_l2::file::L2Ir;
/// use vize_l4::targets::ts::project_program;
/// fn discard(l2ir: L2Ir<'_>) {
///     let projection = project_program(&l2ir).unwrap();
///     drop(l2ir);
///     let _ = projection.document();
/// }
/// ```
pub struct ProgramProjection<'file, 'arena> {
    l2ir: &'file L2Ir<'arena>,
    unit: &'file ScriptUnit,
    kind: SourceKind,
    document: EmitDocument,
}

impl<'file, 'arena> ProgramProjection<'file, 'arena> {
    #[must_use]
    pub fn l2ir(&self) -> &'file L2Ir<'arena> {
        self.l2ir
    }

    #[must_use]
    pub fn unit(&self) -> &'file ScriptUnit {
        self.unit
    }

    #[must_use]
    pub const fn source_kind(&self) -> SourceKind {
        self.kind
    }

    #[must_use]
    pub fn document(&self) -> &EmitDocument {
        &self.document
    }
}

/// Produce a recording checker document; all authored Program bytes stay exact.
pub fn project_program<'file, 'arena>(
    l2ir: &'file L2Ir<'arena>,
) -> Result<ProgramProjection<'file, 'arena>, ProjectionError> {
    project::<Recorded, false>(l2ir)
}

/// Identical output without links. Diagnostic mapping explicitly refuses it.
pub fn project_program_no_links<'file, 'arena>(
    l2ir: &'file L2Ir<'arena>,
) -> Result<ProgramProjection<'file, 'arena>, ProjectionError> {
    project::<NoLinks, false>(l2ir)
}

/// Preserve the whole original JSX/TSX Module through its genuine owning body.
/// This borrows the owner's actual File; it performs no parse or semantic walk.
///
/// A separately supplied File cannot establish JSX ownership:
/// ```compile_fail
/// use vize_l2::file::L2Ir;
/// use vize_l4::targets::ts::project_jsx_program;
/// fn forge(l2ir: &L2Ir<'_>) { let _ = project_jsx_program(l2ir); }
/// ```
/// The original owning body cannot be dropped while its projection is live:
/// ```compile_fail
/// use vize_l2::lang::js::JsxL2Ir;
/// use vize_l4::targets::ts::project_jsx_program;
/// fn discard(owner: JsxL2Ir<'_>) {
///     let projection = project_jsx_program(&owner).unwrap();
///     drop(owner);
///     let _ = projection.document();
/// }
/// ```
pub fn project_jsx_program<'file, 'arena>(
    owner: &'file JsxL2Ir<'arena>,
) -> Result<ProgramProjection<'file, 'arena>, ProjectionError> {
    project::<Recorded, true>(owner.l2ir())
}

/// Identical JSX/TSX output without links; diagnostic mapping refuses it.
pub fn project_jsx_program_no_links<'file, 'arena>(
    owner: &'file JsxL2Ir<'arena>,
) -> Result<ProgramProjection<'file, 'arena>, ProjectionError> {
    project::<NoLinks, true>(owner.l2ir())
}

// Only the two concrete built-in sinks can mint this result. A caller-defined
// LinkSink cannot substitute arbitrary output through into_document().
fn project<'file, 'arena, L: LinkSink, const JSX: bool>(
    l2ir: &'file L2Ir<'arena>,
) -> Result<ProgramProjection<'file, 'arena>, ProjectionError> {
    if !l2ir.is_complete() {
        return Err(ProjectionError::IncompleteFile);
    }
    let [unit] = l2ir.units() else {
        return Err(ProjectionError::ProgramCount);
    };
    let source = l2ir.artifact().source();
    let length = u32::try_from(source.len()).map_err(|_| ProjectionError::SourceTooLarge)?;
    let capacity = source
        .len()
        .checked_add(MODULE_SUFFIX.len())
        .filter(|length| u32::try_from(*length).is_ok())
        .ok_or(ProjectionError::SourceTooLarge)?;
    if unit.span != Span::new(0, length) {
        return Err(ProjectionError::PartialSource);
    }
    if !unit.profile.module || unit.profile.jsx != JSX {
        return Err(ProjectionError::UnsupportedProfile);
    }
    if !l2ir
        .scopes()
        .get(unit.scope.index() as usize)
        .is_some_and(|scope| scope.id == unit.scope && scope.parent.is_none())
    {
        return Err(ProjectionError::NestedScope);
    }
    if l2ir.artifact().node_count() != 0 || !l2ir.artifact().root().ops.is_empty() {
        return Err(ProjectionError::TemplateOperations);
    }
    let kind = match (unit.profile.typescript, JSX) {
        (false, false) => SourceKind::JavaScript,
        (true, false) => SourceKind::TypeScript,
        (false, true) => SourceKind::Jsx,
        (true, true) => SourceKind::Tsx,
    };
    let mut writer = Writer::<L>::with_capacity(capacity);
    writer.push_linked(source, unit.span);
    writer.push(MODULE_SUFFIX);
    Ok(ProgramProjection {
        l2ir,
        unit,
        kind,
        document: writer.finish().into_document(),
    })
}

#[cfg(feature = "module-ir")]
pub use module::{ModuleProjectionError, project_module};

#[cfg(feature = "component-projection")]
pub use helpers::COMPONENT_HELPERS;
#[cfg(feature = "component-projection")]
pub use module::project_module_with_component_checks;

#[cfg(feature = "diagnostic-positions")]
mod diagnostics;
#[cfg(feature = "diagnostic-positions")]
pub use diagnostics::DiagnosticMapper;
