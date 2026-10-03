//! Checkable original JS/TS Modules, retaining their sole completed File owner.
//!
//! Vue template projection and the Canon default route remain unfinished
//! (#6849, #6879). This bounded entry never rewrites or reparses a Program.

use crate::write::{EmitDocument, LinkSink, NoLinks, Recorded, Writer};
use vize_l0::Span;
use vize_l2::file::{FileArtifact, ScriptUnit};

mod mapping;
pub use mapping::MappingError;
pub mod vue;

const MODULE_SUFFIX: &str = "\n;\nexport {};\n";

/// The original admitted parse language, also selecting the checker file kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceKind {
    JavaScript,
    TypeScript,
}

impl SourceKind {
    #[must_use]
    pub const fn extension(self) -> &'static str {
        match self {
            Self::JavaScript => "mjs",
            Self::TypeScript => "ts",
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
/// use vize_l2::file::FileArtifact;
/// use vize_l4::{targets::ts::ProgramProjection, write::EmitDocument};
/// fn forge(file: &FileArtifact<'_>, document: EmitDocument) {
///     let _ = ProgramProjection { file, document };
/// }
/// ```
/// The actual File cannot be dropped while the projection is live:
/// ```compile_fail
/// use vize_l2::file::FileArtifact;
/// use vize_l4::targets::ts::project_program;
/// fn discard(file: FileArtifact<'_>) {
///     let projection = project_program(&file).unwrap();
///     drop(file);
///     let _ = projection.document();
/// }
/// ```
pub struct ProgramProjection<'file, 'arena> {
    file: &'file FileArtifact<'arena>,
    unit: &'file ScriptUnit,
    kind: SourceKind,
    document: EmitDocument,
}

impl<'file, 'arena> ProgramProjection<'file, 'arena> {
    #[must_use]
    pub fn file(&self) -> &'file FileArtifact<'arena> {
        self.file
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
    file: &'file FileArtifact<'arena>,
) -> Result<ProgramProjection<'file, 'arena>, ProjectionError> {
    project::<Recorded>(file)
}

/// Identical output without links. Diagnostic mapping explicitly refuses it.
pub fn project_program_no_links<'file, 'arena>(
    file: &'file FileArtifact<'arena>,
) -> Result<ProgramProjection<'file, 'arena>, ProjectionError> {
    project::<NoLinks>(file)
}

// Only the two concrete built-in sinks can mint this result. A caller-defined
// LinkSink cannot substitute arbitrary output through into_document().
fn project<'file, 'arena, L: LinkSink>(
    file: &'file FileArtifact<'arena>,
) -> Result<ProgramProjection<'file, 'arena>, ProjectionError> {
    if !file.is_complete() {
        return Err(ProjectionError::IncompleteFile);
    }
    let [unit] = file.units() else {
        return Err(ProjectionError::ProgramCount);
    };
    let source = file.artifact().source();
    let length = u32::try_from(source.len()).map_err(|_| ProjectionError::SourceTooLarge)?;
    let capacity = source
        .len()
        .checked_add(MODULE_SUFFIX.len())
        .filter(|length| u32::try_from(*length).is_ok())
        .ok_or(ProjectionError::SourceTooLarge)?;
    if unit.span != Span::new(0, length) {
        return Err(ProjectionError::PartialSource);
    }
    if !unit.profile.module || unit.profile.jsx {
        return Err(ProjectionError::UnsupportedProfile);
    }
    if !file
        .scopes()
        .get(unit.scope.index() as usize)
        .is_some_and(|scope| scope.id == unit.scope && scope.parent.is_none())
    {
        return Err(ProjectionError::NestedScope);
    }
    if file.artifact().node_count() != 0 || !file.artifact().root().ops.is_empty() {
        return Err(ProjectionError::TemplateOperations);
    }
    let kind = if unit.profile.typescript {
        SourceKind::TypeScript
    } else {
        SourceKind::JavaScript
    };
    let mut writer = Writer::<L>::with_capacity(capacity);
    writer.push_linked(source, unit.span);
    writer.push(MODULE_SUFFIX);
    Ok(ProgramProjection {
        file,
        unit,
        kind,
        document: writer.finish().into_document(),
    })
}
