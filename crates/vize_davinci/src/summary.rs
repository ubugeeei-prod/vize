//! The per-SFC summary (P5-2) — one component's GHC `.hi` interface.
//!
//! A summary is built from the α pages of the interface: the component
//! signature, prop / emit / slot types, the reactivity class of each
//! exported binding, and the components it references. Each declaration on
//! those pages has its own fingerprint. A consumer records the declarations
//! it used and is stale exactly when one of those fingerprints is missing
//! or different.
//!
//! The fingerprint covers that declaration alone — its α group, that
//! group's schema, its identity, its contract. It does not cover the file
//! or any sibling declaration. [`AlphaPages`] has no body and no S3
//! code-shape field, and [`Facet::from_alpha_group`] refuses every group
//! that is not one of those pages, so a hot-path optimization has nowhere
//! to go.

mod facet;
mod folio;
mod global;
mod usage;

pub use facet::schema;
pub use facet::{Facet, SummaryError};
pub use global::{
    GlobalEntry, GlobalError, GlobalFacet, GlobalFacts, GlobalResolution, GlobalSummary,
};
pub use usage::Usage;

use alloc::vec::Vec;
use core::fmt;

use vize_s0::hash::StableHasher128;
use vize_s0::{String, cstr};

/// The XXH3-128 fingerprint of one declaration. Not a file hash.
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Fingerprint([u8; 16]);

impl Fingerprint {
    /// The digest bytes.
    #[must_use]
    pub const fn bytes(self) -> [u8; 16] {
        self.0
    }
}

impl fmt::Debug for Fingerprint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fingerprint(")?;
        for byte in self.0 {
            write!(f, "{byte:02x}")?;
        }
        write!(f, ")")
    }
}

/// Domain tag, distinct from artifact keys and manifest folds.
const DOMAIN: &[u8] = b"vize.davinci.sfc-summary\0";

fn fingerprint(facet: Facet, name: &str, contract: &str) -> Fingerprint {
    digest(DOMAIN, facet.group(), facet.schema(), name, contract)
}

pub(super) fn digest(
    domain: &[u8],
    group: &str,
    schema: u16,
    name: &str,
    contract: &str,
) -> Fingerprint {
    let mut hasher = StableHasher128::new();
    hasher.update(domain);
    feed_str(&mut hasher, group);
    hasher.update(&schema.to_le_bytes());
    feed_str(&mut hasher, name);
    feed_str(&mut hasher, contract);
    Fingerprint(hasher.digest())
}

fn feed_str(hasher: &mut StableHasher128, value: &str) {
    hasher.update(&(value.len() as u64).to_le_bytes());
    hasher.update(value.as_bytes());
}

/// Identity of one declaration: the facet plus the α key.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DeclarationId {
    facet: Facet,
    name: String,
}

impl DeclarationId {
    /// A declaration of `facet` named `name`.
    #[must_use]
    pub fn new(facet: Facet, name: impl Into<String>) -> Self {
        Self {
            facet,
            name: name.into(),
        }
    }

    /// The facet.
    #[must_use]
    pub const fn facet(&self) -> Facet {
        self.facet
    }

    /// The α key.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// The component signature α page: one declaration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    /// Exported component name. The declaration's identity.
    pub name: String,
    /// Type-parameter spelling. Empty when the component is not generic.
    /// The declaration's contract.
    pub params: String,
}

/// One α entry: the key (a declaration identity) and the contract.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlphaEntry {
    /// Declaration identity. Not a source offset.
    pub name: String,
    /// The contract spelling. Not a body.
    pub contract: String,
}

/// The α pages a per-SFC summary is built from.
///
/// There is no field for a function body or an S3 code-shape decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlphaPages {
    /// `component-signature`.
    pub signature: Signature,
    /// `prop-types`.
    pub props: Vec<AlphaEntry>,
    /// `emit-types`.
    pub emits: Vec<AlphaEntry>,
    /// `slot-types`.
    pub slots: Vec<AlphaEntry>,
    /// `reactivity-classes`.
    pub reactivity: Vec<AlphaEntry>,
    /// `component-references`, keyed by resolved identity.
    pub components: Vec<AlphaEntry>,
}

/// One stored declaration. The contract is the whole interface fact.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Declaration {
    facet: Facet,
    name: String,
    contract: String,
}

impl Declaration {
    fn id(&self) -> DeclarationId {
        DeclarationId::new(self.facet, self.name.clone())
    }

    fn fingerprint(&self) -> Fingerprint {
        fingerprint(self.facet, &self.name, &self.contract)
    }
}

/// The per-SFC summary: interface declarations, each fingerprinted alone.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SfcSummary {
    declarations: Vec<Declaration>,
}

impl SfcSummary {
    /// Build a summary from the interface α pages.
    ///
    /// # Errors
    ///
    /// A bad declaration name, a contract that contains a newline, or two
    /// entries of the same facet with the same name.
    pub fn from_alpha(pages: AlphaPages) -> Result<Self, SummaryError> {
        let mut declarations = Vec::new();
        insert(
            &mut declarations,
            Facet::Signature,
            pages.signature.name,
            pages.signature.params,
        )?;
        insert_all(&mut declarations, Facet::Prop, pages.props)?;
        insert_all(&mut declarations, Facet::Emit, pages.emits)?;
        insert_all(&mut declarations, Facet::Slot, pages.slots)?;
        insert_all(&mut declarations, Facet::Reactivity, pages.reactivity)?;
        insert_all(&mut declarations, Facet::Component, pages.components)?;
        Ok(finish(declarations))
    }

    /// How many declarations the summary exports.
    #[must_use]
    pub fn len(&self) -> usize {
        self.declarations.len()
    }

    /// Whether the summary exports nothing.
    ///
    /// Every summary `from_alpha` and the folio parser accept has its
    /// signature, so this is false for those values.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.declarations.is_empty()
    }

    /// Every declaration, in `(facet, name)` order: facet, name, contract.
    #[must_use]
    pub fn iter(&self) -> impl ExactSizeIterator<Item = (Facet, &str, &str)> + '_ {
        self.declarations
            .iter()
            .map(|decl| (decl.facet, decl.name.as_str(), decl.contract.as_str()))
    }

    /// The fingerprint of one declaration, if the summary exports it.
    #[must_use]
    pub fn fingerprint(&self, facet: Facet, name: &str) -> Option<Fingerprint> {
        let index = self
            .declarations
            .binary_search_by(|decl| (decl.facet, decl.name.as_str()).cmp(&(facet, name)))
            .ok()?;
        self.declarations.get(index).map(|decl| decl.fingerprint())
    }

    /// Declarations whose fingerprint differs, plus declarations only one
    /// side exports, in `(facet, name)` order.
    #[must_use]
    pub fn changed(&self, next: &Self) -> Vec<DeclarationId> {
        let mut changed = Vec::new();
        let mut left = self.declarations.iter();
        let mut right = next.declarations.iter();
        let mut lhs = left.next();
        let mut rhs = right.next();
        loop {
            match (lhs, rhs) {
                (None, None) => break,
                (Some(only), None) => {
                    changed.push(only.id());
                    lhs = left.next();
                }
                (None, Some(only)) => {
                    changed.push(only.id());
                    rhs = right.next();
                }
                (Some(prev), Some(next_decl)) => {
                    match (prev.facet, prev.name.as_str())
                        .cmp(&(next_decl.facet, next_decl.name.as_str()))
                    {
                        core::cmp::Ordering::Less => {
                            changed.push(prev.id());
                            lhs = left.next();
                        }
                        core::cmp::Ordering::Greater => {
                            changed.push(next_decl.id());
                            rhs = right.next();
                        }
                        core::cmp::Ordering::Equal => {
                            if prev.fingerprint() != next_decl.fingerprint() {
                                changed.push(prev.id());
                            }
                            lhs = left.next();
                            rhs = right.next();
                        }
                    }
                }
            }
        }
        changed
    }
}

fn insert_all(
    out: &mut Vec<Declaration>,
    facet: Facet,
    entries: Vec<AlphaEntry>,
) -> Result<(), SummaryError> {
    for entry in entries {
        insert(out, facet, entry.name, entry.contract)?;
    }
    Ok(())
}

fn insert(
    out: &mut Vec<Declaration>,
    facet: Facet,
    name: String,
    contract: String,
) -> Result<(), SummaryError> {
    if name.is_empty() || name.chars().any(|ch| ch.is_whitespace() || ch == '=') {
        return Err(SummaryError::BadName { facet, name });
    }
    if contract.chars().any(|ch| ch == '\n' || ch == '\r') {
        return Err(SummaryError::BadContract { facet, name });
    }
    if out
        .iter()
        .any(|decl| decl.facet == facet && decl.name == name)
    {
        return Err(SummaryError::Duplicate { facet, name });
    }
    out.push(Declaration {
        facet,
        name,
        contract,
    });
    Ok(())
}

fn finish(mut declarations: Vec<Declaration>) -> SfcSummary {
    declarations.sort_unstable_by(|lhs, rhs| {
        (lhs.facet, lhs.name.as_str()).cmp(&(rhs.facet, rhs.name.as_str()))
    });
    debug_assert_eq!(
        declarations
            .iter()
            .filter(|decl| decl.facet == Facet::Signature)
            .count(),
        1
    );
    SfcSummary { declarations }
}

fn folio_error(err: SummaryError, line: usize) -> crate::folio::FolioError {
    let message = match err {
        SummaryError::BadName { name, .. } => cstr!("invalid declaration name `{name}`"),
        SummaryError::BadContract { name, .. } => cstr!("invalid contract of `{name}`"),
        SummaryError::Duplicate { name, .. } => cstr!("duplicate declaration `{name}`"),
        SummaryError::NotInterface { group } => cstr!("not an interface group `{group}`"),
        SummaryError::Schema {
            group,
            expected,
            found,
        } => cstr!("schema of `{group}` is {found}, expected {expected}"),
        SummaryError::Unknown { name, .. } => cstr!("unknown declaration `{name}`"),
        SummaryError::EmptyConsumer => cstr!("empty consumer"),
        SummaryError::DuplicateUse { name, .. } => cstr!("duplicate use `{name}`"),
    };
    crate::folio::FolioError::new(line, message)
}
