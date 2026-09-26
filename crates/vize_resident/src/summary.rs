//! Resident per-SFC interface firewalls. The α producer supplies
//! [`AlphaPages`]; no body or L3 code shape can enter that input. Reading the
//! source revision is deliberate: a body edit rechecks the interface, then
//! salsa backdates an unchanged summary and its declaration fingerprints.

use salsa::{Durability, Setter as _};
use vize_davinci::summary::{AlphaPages, Facet, Fingerprint, SfcSummary, SummaryError};
use vize_l0::hash::StableHasher128;
use vize_l0::{String, cstr};

use crate::db::{ResidentDatabase, SourceFile};

/// Bounded memo capacities in the selected `[resource.<preset>]` entry.
/// These are entry counts, not an RSS proof: P5-4b still needs its synthetic
/// 10k-file session to establish the process-level memory ceiling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SummaryCachePolicy {
    /// Maximum cached per-SFC summaries.
    pub summaries: usize,
    /// Maximum cached declaration fingerprints.
    pub declarations: usize,
}

impl SummaryCachePolicy {
    /// Read the policy bundled in `budgets.toml [resource]`.
    pub fn from_resource_preset(preset: &str) -> Result<Self, String> {
        let budgets: toml::Table = include_str!("../../../docs/davinci/plan/budgets.toml")
            .parse()
            .map_err(|error: toml::de::Error| cstr!("{error}"))?;
        let cache = budgets
            .get("resource")
            .and_then(|v| v.get(preset))
            .and_then(|v| v.get("methodology"))
            .and_then(|v| v.get("summary_cache"))
            .ok_or_else(|| String::from("missing resource summary_cache policy"))?;
        let positive = |key: &str| -> Result<usize, String> {
            let value = cache
                .get(key)
                .and_then(toml::Value::as_integer)
                .ok_or_else(|| cstr!("missing summary_cache.{key}"))?;
            usize::try_from(value)
                .ok()
                .filter(|value| *value > 0)
                .ok_or_else(|| cstr!("summary_cache.{key} must be positive"))
        };
        if positive("intern_revisions")? != 2 {
            return Err(String::from(
                "summary_cache.intern_revisions must match salsa's two-revision GC",
            ));
        }
        Ok(Self {
            summaries: positive("summaries")?,
            declarations: positive("declarations")?,
        })
    }
}

pub(crate) fn apply_cache_policy(db: &mut ResidentDatabase, policy: SummaryCachePolicy) {
    sfc_summary::set_lru_capacity(db, policy.summaries);
    declaration_fingerprint::set_lru_capacity(db, policy.declarations);
}

/// TypeScript configuration is stable across editor keystrokes, but changing
/// it may change the interpretation of every α page.
#[salsa::input(singleton, debug)]
pub struct TsConfig {
    #[returns(ref)]
    pub text: String,
}

/// The editor's imported-source snapshot changes at buffer durability.
#[salsa::input(singleton, debug)]
pub(crate) struct SourceWorld {
    #[returns(copy)]
    pub revision: u64,
}

/// The upstream α producer's current pages for one SFC. The source is kept
/// separately so body edits cannot be accidentally included in the summary.
#[salsa::input(debug)]
pub struct SummaryInput {
    #[returns(copy)]
    pub file: SourceFile,
    #[returns(ref)]
    pub pages: AlphaPages,
    #[returns(copy)]
    pub source_stamp: [u8; 16],
    #[returns(copy)]
    pub config_stamp: [u8; 16],
    #[returns(copy)]
    pub world_revision: u64,
}

/// A stale upstream α export is an error, not an unchanged interface.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResidentSummaryError {
    /// The source or tsconfig changed since the α pages were published.
    StaleAlpha,
    /// The published α pages were not a valid P5-2 interface.
    InvalidAlpha(SummaryError),
}

fn stamp(domain: &[u8], text: &str) -> [u8; 16] {
    let mut hasher = StableHasher128::new();
    hasher.update(domain);
    hasher.update(&(text.len() as u64).to_le_bytes());
    hasher.update(text.as_bytes());
    hasher.digest()
}

fn source_stamp(db: &dyn salsa::Database, file: SourceFile) -> [u8; 16] {
    let mut hasher = StableHasher128::new();
    hasher.update(b"vize.resident.alpha.source\0");
    for value in [file.path(db).as_str(), file.text(db).as_str()] {
        hasher.update(&(value.len() as u64).to_le_bytes());
        hasher.update(value.as_bytes());
    }
    hasher.digest()
}

fn config_stamp(text: &str) -> [u8; 16] {
    stamp(b"vize.resident.alpha.config\0", text)
}

pub(crate) fn alpha_is_current(db: &ResidentDatabase, input: SummaryInput) -> bool {
    source_stamp(db, input.file(db)) == input.source_stamp(db)
        && config_stamp(TsConfig::get(db).text(db)) == input.config_stamp(db)
        && SourceWorld::get(db).revision(db) == input.world_revision(db)
}

/// A declaration identity shared by users. Salsa reclaims interned names
/// after two revisions without use; IDs must be re-interned on each read.
#[salsa::interned(revisions = 2)]
pub struct DeclarationName<'db> {
    #[returns(ref)]
    pub name: String,
}

/// The interface for one SFC. Unchanged values are backdated even if the
/// source body or tsconfig triggered re-evaluation.
#[salsa::tracked(returns(ref), lru = 128)]
pub fn sfc_summary(
    db: &dyn salsa::Database,
    input: SummaryInput,
) -> Result<SfcSummary, ResidentSummaryError> {
    let config = TsConfig::get(db).text(db);
    if source_stamp(db, input.file(db)) != input.source_stamp(db)
        || config_stamp(config) != input.config_stamp(db)
        || SourceWorld::get(db).revision(db) != input.world_revision(db)
    {
        return Err(ResidentSummaryError::StaleAlpha);
    }
    SfcSummary::from_alpha(input.pages(db).clone()).map_err(ResidentSummaryError::InvalidAlpha)
}

/// The exact declaration a dependent used. A sibling declaration can
/// change while this query returns the same fingerprint and backdates it.
#[salsa::tracked(returns(copy), lru = 512)]
pub fn declaration_fingerprint<'db>(
    db: &'db dyn salsa::Database,
    input: SummaryInput,
    facet: Facet,
    name: DeclarationName<'db>,
) -> Option<Fingerprint> {
    sfc_summary(db, input)
        .as_ref()
        .ok()?
        .fingerprint(facet, name.name(db))
}

impl ResidentDatabase {
    /// Register the α pages produced for a resident file.
    pub fn publish_alpha(&self, file: SourceFile, pages: AlphaPages) -> SummaryInput {
        let source_stamp = source_stamp(self, file);
        let config_stamp = config_stamp(TsConfig::get(self).text(self));
        let world_revision = SourceWorld::get(self).revision(self);
        SummaryInput::builder(file, pages, source_stamp, config_stamp, world_revision)
            .durability(Durability::LOW)
            .new(self)
    }

    /// Replace the α pages in the same revision as the corresponding edit.
    pub fn revise_alpha(&mut self, input: SummaryInput, pages: AlphaPages) {
        let source_stamp = source_stamp(self, input.file(self));
        let config_stamp = config_stamp(TsConfig::get(self).text(self));
        input
            .set_pages(self)
            .with_durability(Durability::LOW)
            .to(pages);
        input
            .set_source_stamp(self)
            .with_durability(Durability::LOW)
            .to(source_stamp);
        input
            .set_config_stamp(self)
            .with_durability(Durability::LOW)
            .to(config_stamp);
        let world_revision = SourceWorld::get(self).revision(self);
        input
            .set_world_revision(self)
            .with_durability(Durability::LOW)
            .to(world_revision);
    }

    /// Replace a buffer and its freshly exported α pages before any query
    /// can observe the new revision. A caller with no new pages gets
    /// `StaleAlpha` rather than an old declaration fingerprint.
    pub fn edit_with_alpha(
        &mut self,
        file: SourceFile,
        input: SummaryInput,
        text: &str,
        pages: AlphaPages,
    ) {
        self.edit(file, text);
        self.revise_alpha(input, pages);
    }

    /// Change the high-durability TypeScript project configuration.
    pub fn configure_tsconfig(&mut self, text: &str) {
        TsConfig::get(self)
            .set_text(self)
            .with_durability(Durability::HIGH)
            .to(String::from(text));
    }

    /// Intern a declaration name for the current revision.
    pub fn declaration_name(&self, name: &str) -> DeclarationName<'_> {
        DeclarationName::new(self, String::from(name))
    }
}
