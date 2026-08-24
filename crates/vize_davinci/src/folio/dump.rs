//! Per-pass folio dumping with the `--folio-after-change` hash gate (P2-13).
//!
//! [`FolioDump`] collects the pages a `--folio-dir` run writes: one page per
//! executed pass, or - hash-gated - only the passes whose artifact actually
//! changed. It works on the artifact's **canonical `Full`-mode text** rather
//! than on a typed artifact, because the Folio equality laws make the two
//! interchangeable: canonical text is injective over values, so hashing the
//! text is hashing the artifact, and the driver holding the artifact prints
//! once and feeds every consumer the same bytes.
//!
//! The library half is deliberately IO-free (`no_std + alloc`): pages carry
//! their text plus stage/pass provenance (which the P2-18 Spolvero feed
//! serializes), and writing them into a directory is the host binary's job
//! (`davinci-opt`, the `vize` CLI). Page names are
//! `{seq:03}-{stage}.{pass}.folio` in emission order, so a directory listing
//! sorts into the order the passes ran and two dumps of the same pass name
//! (one pipeline may run a pass twice) cannot collide.

use alloc::vec::Vec;

use vize_s0::hash::hash_str;
use vize_s0::{String, cstr};

use crate::pass::PassEvent;

/// One page a dump run produced: the file name to write and its text,
/// plus the stage/pass provenance the Spolvero feed (P2-18) serializes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DumpPage {
    /// `{seq:03}-{stage}.{pass}.folio`, emission-ordered. Derived from
    /// (`seq`, [`stage`](Self::stage), [`pass`](Self::pass)) once, at
    /// emission - the one place the sequence number is known - and cached
    /// here; the semantic fields are the two below.
    pub name: String,
    /// Stage whose pipeline emitted the page (`event.pipeline.stage`).
    pub stage: String,
    /// Pass after which the page was emitted (`event.desc().name`).
    pub pass: String,
    /// The artifact's canonical `Full`-mode text after the named pass.
    pub text: String,
}

/// Collects per-pass folio pages, optionally hash-gated.
#[derive(Debug, Default)]
pub struct FolioDump {
    /// When set, a pass emits a page only if the artifact's hash changed
    /// across it (`--folio-after-change`). A no-op pass emits nothing.
    pub after_change_only: bool,
    /// Hash of the artifact as last seen ([`seed`](Self::seed) or the last
    /// emitted page). `None` until seeded, so an unseeded dump emits its
    /// first page unconditionally.
    last_hash: Option<u64>,
    /// The pages emitted so far, in emission order.
    pub pages: Vec<DumpPage>,
}

impl FolioDump {
    /// A dump collector; `after_change_only` is the hash gate.
    #[must_use]
    pub const fn new(after_change_only: bool) -> Self {
        Self {
            after_change_only,
            last_hash: None,
            pages: Vec::new(),
        }
    }

    /// Record the artifact's pre-run hash, so the gate compares the first
    /// pass against the input rather than emitting it unconditionally.
    pub fn seed(&mut self, canonical_text: &str) {
        self.last_hash = Some(hash_str(canonical_text));
    }

    /// Offer the artifact's canonical text after `event`'s pass.
    ///
    /// Emits a page unless the gate is on and the hash is unchanged. The
    /// stored hash advances only when a page is emitted, which for the gated
    /// mode is the same thing (an unchanged hash stores nothing new).
    pub fn after_pass(&mut self, event: &PassEvent<'_>, canonical_text: &str) {
        let hash = hash_str(canonical_text);
        if self.after_change_only && self.last_hash == Some(hash) {
            return;
        }
        self.last_hash = Some(hash);
        self.pages.push(DumpPage {
            name: cstr!(
                "{:03}-{}.{}.folio",
                self.pages.len(),
                event.pipeline.stage,
                event.desc().name
            ),
            stage: String::from(event.pipeline.stage),
            pass: String::from(event.desc().name),
            text: String::from(canonical_text),
        });
    }
}
