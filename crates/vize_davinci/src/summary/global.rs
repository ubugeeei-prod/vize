//! Project summary for facts no single SFC owns (P5-3).
//!
//! Global components are the P4-3b project group. App-level provide/inject
//! pairs are the P4-3f project group. Dialect-wide directives sit beside
//! them. Each fact has its own fingerprint, in a domain that is not the
//! per-file summary's, so a project edit cannot move an `SfcSummary`.

use alloc::vec::Vec;

use vize_s0::String;

use super::{Fingerprint, digest};
use crate::fact::ids::{COMPONENT_USAGES, PROVIDE_INJECT};

const DOMAIN: &[u8] = b"vize.davinci.global-summary\0";

const _: () = {
    assert!(COMPONENT_USAGES.index() == 4);
    assert!(PROVIDE_INJECT.index() == 8);
};

/// One project-level fact group.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub enum GlobalFacet {
    /// A component registered for the whole project (P4-3b).
    Component = 0,
    /// An app-level provide (P4-3f).
    Provide = 1,
    /// A directive the dialect defines for every file.
    Directive = 2,
}

impl GlobalFacet {
    const fn group(self) -> &'static str {
        match self {
            Self::Component => "global-components",
            Self::Provide => "app-provide-inject",
            Self::Directive => "dialect-directives",
        }
    }

    const fn schema(self) -> u16 {
        1
    }
}

/// One orphan fact: identity plus the contract spelling, not a body.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalEntry {
    /// Fact identity.
    pub name: String,
    /// Contract spelling. Not a function body.
    pub contract: String,
}

/// The three orphan groups, still empty when the project has none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalFacts {
    /// P4-3b global components.
    pub components: Vec<GlobalEntry>,
    /// P4-3f app-level provides.
    pub provides: Vec<GlobalEntry>,
    /// Dialect-wide directives.
    pub directives: Vec<GlobalEntry>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Fact {
    facet: GlobalFacet,
    name: String,
    contract: String,
}

impl Fact {
    fn fingerprint(&self) -> Fingerprint {
        digest(
            DOMAIN,
            self.facet.group(),
            self.facet.schema(),
            &self.name,
            &self.contract,
        )
    }
}

/// The project summary. It does not store per-file declarations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalSummary {
    facts: Vec<Fact>,
}

impl GlobalSummary {
    /// Build the summary from the three orphan groups.
    ///
    /// # Errors
    ///
    /// A bad name, a contract that contains a newline, or two facts of the
    /// same facet with the same name.
    pub fn from_facts(facts: GlobalFacts) -> Result<Self, GlobalError> {
        let mut stored = Vec::new();
        insert_all(&mut stored, GlobalFacet::Component, facts.components)?;
        insert_all(&mut stored, GlobalFacet::Provide, facts.provides)?;
        insert_all(&mut stored, GlobalFacet::Directive, facts.directives)?;
        stored.sort_unstable_by(|lhs, rhs| {
            (lhs.facet, lhs.name.as_str()).cmp(&(rhs.facet, rhs.name.as_str()))
        });
        Ok(Self { facts: stored })
    }

    /// The fingerprint of one project fact, if this summary exports it.
    #[must_use]
    pub fn fingerprint(&self, facet: GlobalFacet, name: &str) -> Option<Fingerprint> {
        let index = self
            .facts
            .binary_search_by(|fact| (fact.facet, fact.name.as_str()).cmp(&(facet, name)))
            .ok()?;
        Some(self.facts[index].fingerprint())
    }

    /// Files in `resolutions` order whose recorded project facts are not all
    /// still this summary's. Everyone else is left out.
    #[must_use]
    pub fn invalidated<'a>(&self, resolutions: &'a [GlobalResolution]) -> Vec<&'a str> {
        resolutions
            .iter()
            .filter(|resolution| !resolution.is_fresh(self))
            .map(GlobalResolution::file)
            .collect()
    }
}

/// The project facts one file resolved, each with the fingerprint it saw.
///
/// A name this summary did not export is stored with no fingerprint. Adding
/// that fact later makes the file stale. A name the file did not resolve
/// cannot invalidate it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GlobalResolution {
    file: String,
    resolved: Vec<(GlobalFacet, String, Option<Fingerprint>)>,
}

impl GlobalResolution {
    /// Record `resolved` as read from `summary` by `file`.
    ///
    /// # Errors
    ///
    /// An empty file name, or the same fact twice.
    pub fn record(
        file: &str,
        summary: &GlobalSummary,
        resolved: &[(GlobalFacet, &str)],
    ) -> Result<Self, GlobalError> {
        if file.is_empty() {
            return Err(GlobalError::EmptyFile);
        }
        let mut stored = Vec::with_capacity(resolved.len());
        for &(facet, name) in resolved {
            if stored
                .iter()
                .any(|(seen_facet, seen_name, _)| *seen_facet == facet && seen_name == name)
            {
                return Err(GlobalError::Duplicate {
                    facet,
                    name: String::from(name),
                });
            }
            stored.push((facet, String::from(name), summary.fingerprint(facet, name)));
        }
        Ok(Self {
            file: String::from(file),
            resolved: stored,
        })
    }

    /// The file's name.
    #[must_use]
    pub fn file(&self) -> &str {
        &self.file
    }

    /// Whether every recorded fingerprint is unchanged in `next`.
    #[must_use]
    pub fn is_fresh(&self, next: &GlobalSummary) -> bool {
        self.resolved
            .iter()
            .all(|(facet, name, seen)| next.fingerprint(*facet, name) == *seen)
    }
}

/// Why a project summary could not be built, or a resolution could not be recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GlobalError {
    /// The fact name is empty or contains whitespace or `=`.
    BadName { facet: GlobalFacet, name: String },
    /// The contract contains a newline.
    BadContract { facet: GlobalFacet, name: String },
    /// Two facts of `facet` share `name`.
    Duplicate { facet: GlobalFacet, name: String },
    /// A resolution named no file.
    EmptyFile,
}

fn insert_all(
    out: &mut Vec<Fact>,
    facet: GlobalFacet,
    entries: Vec<GlobalEntry>,
) -> Result<(), GlobalError> {
    for entry in entries {
        insert(out, facet, entry.name, entry.contract)?;
    }
    Ok(())
}

fn insert(
    out: &mut Vec<Fact>,
    facet: GlobalFacet,
    name: String,
    contract: String,
) -> Result<(), GlobalError> {
    if name.is_empty() || name.chars().any(|ch| ch.is_whitespace() || ch == '=') {
        return Err(GlobalError::BadName { facet, name });
    }
    if contract.chars().any(|ch| ch == '\n' || ch == '\r') {
        return Err(GlobalError::BadContract { facet, name });
    }
    if out
        .iter()
        .any(|fact| fact.facet == facet && fact.name == name)
    {
        return Err(GlobalError::Duplicate { facet, name });
    }
    out.push(Fact {
        facet,
        name,
        contract,
    });
    Ok(())
}
