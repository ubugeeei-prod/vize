//! Resident per-SFC interface firewalls. The α producer supplies
//! [`AlphaPages`]; no body or S3 code shape can enter that input. Reading the
//! source revision is deliberate: a body edit rechecks the interface, then
//! salsa backdates an unchanged summary and its declaration fingerprints.

use salsa::{Durability, Setter as _};
use vize_davinci::summary::{AlphaPages, Facet, Fingerprint, SfcSummary, SummaryError};
use vize_s0::{String, cstr};

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

/// The upstream α producer's current pages for one SFC. The source is kept
/// separately so body edits cannot be accidentally included in the summary.
#[salsa::input(debug)]
pub struct SummaryInput {
    #[returns(copy)]
    pub file: SourceFile,
    #[returns(ref)]
    pub pages: AlphaPages,
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
) -> Result<SfcSummary, SummaryError> {
    let _source_revision = input.file(db).text(db);
    let _tsconfig = TsConfig::get(db).text(db);
    SfcSummary::from_alpha(input.pages(db).clone())
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
        SummaryInput::builder(file, pages)
            .durability(Durability::LOW)
            .new(self)
    }

    /// Replace the α pages in the same revision as the corresponding edit.
    pub fn revise_alpha(&mut self, input: SummaryInput, pages: AlphaPages) {
        input
            .set_pages(self)
            .with_durability(Durability::LOW)
            .to(pages);
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
