//! The croquis folio: the croquis "VIR" dump absorbed as the first folio
//! page.
//!
//! [`CroquisFolio`] is a document model of the text
//! `vize_croquis::Croquis::to_vir()` emits (the `[vir]` format). Printing
//! in [`FolioMode::Full`](super::FolioMode) reproduces that format
//! byte-for-byte for canonical content; parsing is the inverse. The live
//! renderer stays in `crates/vize_croquis/src/croquis/vir.rs`; this module
//! owns the format contract (see `davinci-road/plan/folio-format.md`).
//!
//! A folio models the dump, not the analysis: counts in the header are
//! carried as printed, and no cross-section invariants are enforced beyond
//! parent-reference resolution in `[scopes]`.

use alloc::vec::Vec;

use vize_s0::String;

use super::{Folio, FolioError, FolioMode};

mod parse;
mod print;

/// One prop line in `[surface.props]`: `{name}{!|?}[:{type}][=]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PropEntry {
    pub name: String,
    pub required: bool,
    pub ty: Option<String>,
    /// Trailing `=` marker (the prop has a default value). Elided in
    /// `Display` mode.
    pub has_default: bool,
}

/// One line in `[surface.expose]` or `[surface.slots]`: either the
/// carried-through argument text or a `@start:end` span fallback.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SurfaceEntry {
    /// Verbatim argument text (runtime args for expose, type args for
    /// slots).
    Args(String),
    /// Span-only fallback (`@start:end`). Elided in `Display` mode.
    Span { start: u32, end: u32 },
}

/// One line in `[macros]`: `@{name}[<{type_args}>] @{start}:{end}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MacroEntry {
    pub name: String,
    pub type_args: Option<String>,
    pub start: u32,
    pub end: u32,
}

/// One line in `[extern]`: `{source}[^][ {{a,b}}]` (`^` = type-only).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternEntry {
    pub source: String,
    pub type_only: bool,
    /// Imported binding names. Sorted under normalization (the renderer
    /// iterates a hash map here).
    pub bindings: Vec<String>,
}

/// Type-export kind marker in `[types]` (`t` / `i`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TypeExportMark {
    Type,
    Interface,
}

impl TypeExportMark {
    pub const fn as_char(self) -> char {
        match self {
            Self::Type => 't',
            Self::Interface => 'i',
        }
    }
}

/// One line in `[types]`: `{name}[^]{t|i}@{start}:{end}` (`^` = hoisted).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeEntry {
    pub name: String,
    pub hoisted: bool,
    pub kind: TypeExportMark,
    pub start: u32,
    pub end: u32,
}

/// One line in `[bindings]`: `{code}:{name,name,...}`.
///
/// `code` is the VIR binding-type code (`c`, `st`, `ist`, ...). Codes are
/// not injective over `BindingType` (several types share `ist`), so a dump
/// may carry several groups with the same code; group order is preserved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BindingGroup {
    pub code: String,
    /// Binding names. Sorted under normalization (the renderer iterates a
    /// hash map here).
    pub names: Vec<String>,
}

/// A scope display id: `{prefix}{index}` (`~0`, `!2`, `#1`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ScopeRef {
    /// `~` (universal), `!` (client-only), or `#` (server-only).
    pub prefix: char,
    pub index: u64,
}

/// One line in `[scopes]`:
/// `{id} {name} @{start}:{end}[ [a,b]][ < p, q]`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeEntry {
    pub id: ScopeRef,
    pub name: String,
    pub start: u32,
    pub end: u32,
    /// Binding names. Sorted under normalization (the renderer iterates a
    /// hash map here).
    pub bindings: Vec<String>,
    /// Parent references, order-preserving (the first parent is the
    /// lexical parent).
    pub parents: Vec<ScopeRef>,
}

/// One line in `[errors]`: `{name}={kind}@{start}:{end}`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorEntry {
    pub name: String,
    /// Invalid-export kind token (`Const`, `Let`, `Var`, `Function`,
    /// `Class`, `Default`); carried verbatim.
    pub kind: String,
    pub start: u32,
    pub end: u32,
}

/// Document model of a croquis VIR dump.
///
/// Sections with no entries are omitted from the printed text, matching
/// the renderer. `emits`, `models`, and `reactivity` lines carry no spans
/// or map-ordered content, so they are stored verbatim.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CroquisFolio {
    /// `[vir] script_setup=` header value.
    pub script_setup: bool,
    /// `[vir] scopes=` header count, carried as printed.
    pub scope_count: u64,
    /// `[vir] bindings=` header count, carried as printed.
    pub binding_count: u64,
    pub props: Vec<PropEntry>,
    /// `[surface.emits]` lines, verbatim.
    pub emits: Vec<String>,
    /// `[surface.models]` lines, verbatim.
    pub models: Vec<String>,
    pub expose: Vec<SurfaceEntry>,
    pub slots: Vec<SurfaceEntry>,
    pub macros: Vec<MacroEntry>,
    /// `[reactivity]` lines, verbatim.
    pub reactivity: Vec<String>,
    pub externs: Vec<ExternEntry>,
    pub types: Vec<TypeEntry>,
    pub bindings: Vec<BindingGroup>,
    pub scopes: Vec<ScopeEntry>,
    pub errors: Vec<ErrorEntry>,
}

impl CroquisFolio {
    /// Normalize in place: sort every map-derived name list and renumber
    /// scope ids sequentially per prefix in entry order (remapping parent
    /// references).
    ///
    /// `parse` already returns normalized values; `print` normalizes on
    /// the fly, so calling this is only needed for hand-built values that
    /// should compare structurally against parsed ones.
    pub fn normalize(&mut self) {
        for group in &mut self.bindings {
            group.names.sort_unstable();
        }
        for ext in &mut self.externs {
            ext.bindings.sort_unstable();
        }
        for scope in &mut self.scopes {
            scope.bindings.sort_unstable();
        }
        let remap = print::scope_id_remap(&self.scopes);
        for scope in &mut self.scopes {
            scope.id = print::remap_ref(&remap, scope.id);
            for parent in &mut scope.parents {
                *parent = print::remap_ref(&remap, *parent);
            }
        }
    }
}

impl Folio for CroquisFolio {
    fn print<W: core::fmt::Write>(&self, w: &mut W, mode: FolioMode) -> core::fmt::Result {
        print::print(self, w, mode)
    }

    fn parse(input: &str) -> Result<Self, FolioError> {
        let mut folio = parse::parse(input)?;
        folio.normalize();
        Ok(folio)
    }
}
