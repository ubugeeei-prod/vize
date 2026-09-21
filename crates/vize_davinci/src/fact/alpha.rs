//! The α form of a fact group (P4-2) — the Lean environment-extension
//! import.
//!
//! Every group has a **β** form: the in-memory [`FactTable`] a manager
//! rebuilds on demand. A group that crosses a compile boundary also declares
//! an **α** form: an owned, versioned page produced by an explicit
//! [`AlphaExport::export`] and read back by [`AlphaExport::import`], which is
//! what the P5-2 per-SFC summary serializes. α is versioned per group
//! ([`AlphaExport::ALPHA_SCHEMA`]), independently of any other group and of
//! the β table's in-memory shape.
//!
//! Every α page is printed inside an [`AlphaDocument`], whose header names
//! the group and its `schema_version` (the P2-17 rule), so no α page can be
//! written without its version and none is read back under another one.
//! Every α group is documented in `davinci-road/plan/fact-alpha-schemas.md`;
//! [`check_alpha_schema_doc`] is the executable half of that rule.

use core::fmt;
use core::marker::PhantomData;

use vize_s0::{String, cstr};

use super::{FactGroup, FactTable};
use crate::folio::{Folio, FolioError, FolioMode};
use crate::pass::AnalysisId;

/// A fact group with an exported α form.
pub trait AlphaExport: FactGroup {
    /// The α page's schema version: bumped whenever the page's shape or
    /// meaning changes, independently of every other group.
    const ALPHA_SCHEMA: u16;
    /// The owned α page. `'static` is the P1-11 arena/cache contract: an α
    /// value never borrows an arena or a source.
    type Alpha: Folio + PartialEq + Send + Sync + 'static;

    /// Export the β table as its α page.
    fn export(table: &FactTable<Self>) -> Self::Alpha;
    /// Rebuild the β table from an α page; `import(export(t)) == t`.
    fn import(alpha: Self::Alpha) -> FactTable<Self>;

    /// This group's α identity, for the schema-doc check.
    const ALPHA_DESC: AlphaDesc = AlphaDesc {
        id: Self::ID,
        name: Self::NAME,
        schema: Self::ALPHA_SCHEMA,
    };
}

/// One α group as const data: what the schema doc must list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AlphaDesc {
    pub id: AnalysisId,
    pub name: &'static str,
    pub schema: u16,
}

/// Every production group with an α form. A group gains its row here in the
/// same change that implements [`AlphaExport`] for it, and the schema doc
/// documents it before that change can land.
pub const ALPHA_GROUPS: &[AlphaDesc] = &[];

/// An α page inside its versioned header.
///
/// ```text
/// [fact-alpha]
/// group=<name>
/// schema_version=<n>
///
/// <the α page>
/// ```
pub struct AlphaDocument<G: AlphaExport> {
    pub alpha: G::Alpha,
    group: PhantomData<G>,
}

impl<G: AlphaExport> PartialEq for AlphaDocument<G> {
    fn eq(&self, other: &Self) -> bool {
        self.alpha == other.alpha
    }
}

impl<G: AlphaExport> fmt::Debug for AlphaDocument<G>
where
    G::Alpha: fmt::Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AlphaDocument")
            .field("group", &G::NAME)
            .field("schema_version", &G::ALPHA_SCHEMA)
            .field("alpha", &self.alpha)
            .finish()
    }
}

impl<G: AlphaExport> AlphaDocument<G> {
    /// Export `table` as a versioned α document.
    #[must_use]
    pub fn export(table: &FactTable<G>) -> Self {
        Self {
            alpha: G::export(table),
            group: PhantomData,
        }
    }

    /// Rebuild the β table.
    #[must_use]
    pub fn import(self) -> FactTable<G> {
        G::import(self.alpha)
    }
}

const HEADER_LINES: usize = 4;

impl<G: AlphaExport> Folio for AlphaDocument<G> {
    fn print<W: fmt::Write>(&self, w: &mut W, mode: FolioMode) -> fmt::Result {
        writeln!(w, "[fact-alpha]")?;
        writeln!(w, "group={}", G::NAME)?;
        writeln!(w, "schema_version={}", G::ALPHA_SCHEMA)?;
        writeln!(w)?;
        self.alpha.print(w, mode)
    }

    fn parse(input: &str) -> Result<Self, FolioError> {
        let mut lines = input.splitn(HEADER_LINES + 1, '\n');
        let mut expect = |line: usize, expected: &str| match lines.next() {
            Some(found) if found == expected => Ok(()),
            Some(found) => Err(FolioError::new(
                line,
                cstr!("expected `{expected}`, found `{found}`"),
            )),
            None => Err(FolioError::new(0, String::from("truncated α header"))),
        };
        expect(1, "[fact-alpha]")?;
        expect(2, cstr!("group={}", G::NAME).as_str())?;
        expect(3, cstr!("schema_version={}", G::ALPHA_SCHEMA).as_str())?;
        expect(4, "")?;
        let body = lines.next().unwrap_or_default();
        let alpha = G::Alpha::parse(body).map_err(|error| {
            let line = if error.line == 0 {
                0
            } else {
                error.line + HEADER_LINES
            };
            FolioError::new(line, error.message)
        })?;
        Ok(Self {
            alpha,
            group: PhantomData,
        })
    }
}

/// Why the α schema doc does not cover the registered groups.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlphaDocError {
    /// A registered α group has no row in the doc.
    Undocumented { group: &'static str },
    /// The doc's row disagrees with the code about the id or version.
    Mismatch {
        group: &'static str,
        documented: String,
    },
}

/// Check that every group in `groups` has a row `| \`name\` | id | schema |`
/// in the schema doc's table, with the same id and schema version.
///
/// # Errors
///
/// The first group, in `groups` order, the doc misses or contradicts.
pub fn check_alpha_schema_doc(doc: &str, groups: &[AlphaDesc]) -> Result<(), AlphaDocError> {
    for group in groups {
        let quoted = cstr!("`{}`", group.name);
        let row = doc.lines().find_map(|line| {
            let mut cells = line.split('|').map(str::trim).skip(1);
            (cells.next() == Some(quoted.as_str())).then(|| (cells.next(), cells.next()))
        });
        let Some((id, schema)) = row else {
            return Err(AlphaDocError::Undocumented { group: group.name });
        };
        let expected = (cstr!("{}", group.id.index()), cstr!("{}", group.schema));
        if (id, schema) != (Some(expected.0.as_str()), Some(expected.1.as_str())) {
            return Err(AlphaDocError::Mismatch {
                group: group.name,
                documented: cstr!("id={} schema={}", id.unwrap_or(""), schema.unwrap_or("")),
            });
        }
    }
    Ok(())
}
