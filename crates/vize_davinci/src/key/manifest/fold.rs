//! The manifest values, the declared-input check, and the fold into a key.
//!
//! Storage-free by construction: a manifest is one optional 128-bit digest
//! per ambient input (a value is hashed when set — a tsconfig's content can
//! be large, its identity is not), and input sets are a bitset.

use vize_s0::hash::StableHasher128;

use super::{AmbientInput, CachedArtifact};
use crate::key::sink::MANIFEST_DOMAIN;
use crate::key::{ArtifactKey, KeySink};
use crate::stage::Stage;

/// A set of ambient inputs.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct InputSet(u16);

impl InputSet {
    /// The empty set.
    pub const EMPTY: Self = Self(0);

    /// The set of `inputs`.
    #[must_use]
    pub fn of(inputs: &[AmbientInput]) -> Self {
        let mut set = Self::EMPTY;
        for input in inputs {
            set.0 |= bit(*input);
        }
        set
    }

    /// Whether `input` is in the set.
    #[must_use]
    pub const fn contains(self, input: AmbientInput) -> bool {
        self.0 & bit(input) != 0
    }

    /// Whether the set is empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// The members, in manifest order.
    pub fn iter(self) -> impl Iterator<Item = AmbientInput> {
        AmbientInput::ALL
            .into_iter()
            .filter(move |input| self.contains(*input))
    }
}

const fn bit(input: AmbientInput) -> u16 {
    1 << input as u8
}

/// Why a manifest cannot key an artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ManifestError {
    /// The content key belongs to a stage the artifact is not keyed by.
    WrongStage {
        /// The artifact being keyed.
        artifact: CachedArtifact,
        /// The content key's stage.
        stage: Stage,
    },
    /// The manifest does not set exactly the artifact's declared inputs.
    Undeclared {
        /// The artifact being keyed.
        artifact: CachedArtifact,
        /// Declared inputs the manifest lacks (the corruption bug).
        missing: InputSet,
        /// Set inputs the artifact does not declare.
        extra: InputSet,
    },
}

/// The values of ambient inputs: one digest per set input.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct KeyManifest {
    values: [Option<[u8; 16]>; AmbientInput::ALL.len()],
}

/// Fold markers: a manifest over a content key, or a manifest alone.
const WITH_CONTENT: u8 = 1;
const NO_CONTENT: u8 = 0;

impl KeyManifest {
    /// An empty manifest.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            values: [None; AmbientInput::ALL.len()],
        }
    }

    /// Set `input` to `value` (replacing an earlier value).
    #[must_use]
    pub fn with(mut self, input: AmbientInput, value: &str) -> Self {
        self.set(input, value);
        self
    }

    /// Set `input` to `value` (replacing an earlier value).
    pub fn set(&mut self, input: AmbientInput, value: &str) {
        let mut hasher = StableHasher128::new();
        hasher.update(value.as_bytes());
        self.values[input as usize] = Some(hasher.digest());
    }

    /// The inputs this manifest sets.
    #[must_use]
    pub fn inputs(&self) -> InputSet {
        let mut set = InputSet::EMPTY;
        for input in AmbientInput::ALL {
            if self.values[input as usize].is_some() {
                set.0 |= bit(input);
            }
        }
        set
    }

    /// Check that the manifest sets exactly `artifact`'s declared inputs.
    pub fn check(&self, artifact: CachedArtifact) -> Result<(), ManifestError> {
        let declared = InputSet::of(artifact.inputs());
        let set = self.inputs();
        let missing = InputSet(declared.0 & !set.0);
        let extra = InputSet(set.0 & !declared.0);
        if missing.is_empty() && extra.is_empty() {
            return Ok(());
        }
        Err(ManifestError::Undeclared {
            artifact,
            missing,
            extra,
        })
    }

    /// The manifest's own key for `artifact` — the whole key of an artifact
    /// with no content key (a Corsa session).
    pub fn fingerprint(&self, artifact: CachedArtifact) -> Result<[u8; 16], ManifestError> {
        self.check(artifact)?;
        // No content key: the domain's stage and version slots are fixed.
        let mut sink = KeySink::in_domain(MANIFEST_DOMAIN, Stage::Source, 0, 0);
        sink.feed_tag(NO_CONTENT);
        self.fold_into(&mut sink, artifact);
        Ok(sink.finish().hash())
    }

    fn fold_into(&self, sink: &mut KeySink, artifact: CachedArtifact) {
        sink.feed_str(artifact.name());
        for input in AmbientInput::ALL {
            if let Some(digest) = &self.values[input as usize] {
                sink.feed_str(input.name());
                sink.feed_digest(digest);
            }
        }
    }
}

impl ArtifactKey {
    /// The cache key of `artifact`: this content key with the manifest's
    /// ambient inputs folded in. Stage and recipe version are kept; only the
    /// hash changes. Fails unless the manifest sets exactly the artifact's
    /// inputs and the content key is one of the artifact's
    /// [`content_stages`](CachedArtifact::content_stages).
    pub fn with_manifest(
        self,
        artifact: CachedArtifact,
        manifest: &KeyManifest,
    ) -> Result<Self, ManifestError> {
        if !artifact.content_stages().contains(&self.stage) {
            return Err(ManifestError::WrongStage {
                artifact,
                stage: self.stage,
            });
        }
        manifest.check(artifact)?;
        let mut sink = KeySink::in_domain(MANIFEST_DOMAIN, self.stage, self.schema_version, 0);
        sink.feed_tag(WITH_CONTENT);
        sink.feed_digest(&self.hash);
        manifest.fold_into(&mut sink, artifact);
        Ok(sink.finish())
    }
}
