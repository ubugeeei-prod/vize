//! Ambient key manifests (P5-1b): every input a cached artifact depends on
//! that is not its own content.
//!
//! A cache key covering less than its artifact's inputs is a corruption bug,
//! not a performance detail — two different computations would share one
//! entry. Content is covered by the artifact key (P5-1a). Everything else —
//! the project, its tsconfig, the Vize configuration, the toolchain, the
//! Corsa build, feature flags, the platform — is **ambient**, and every
//! cached artifact declares which ambient inputs it reads
//! ([`CachedArtifact::inputs`], documented row for row in
//! `davinci-road/plan/key-manifests.md`). A [`KeyManifest`] carries their
//! values and folds into a key only when it declares **exactly** the
//! artifact's inputs: a missing one is the corruption bug, an extra one is an
//! undeclared input — both are errors, never silently accepted.

use alloc::vec::Vec;

use vize_s0::String;

use super::sink::MANIFEST_DOMAIN;
use super::{ArtifactKey, KeySink};
use crate::stage::Stage;

/// One ambient input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AmbientInput {
    /// The project's identity: its canonical root / `tsconfig.json` path.
    ProjectIdentity,
    /// The resolved `tsconfig.json` content (extends chain included).
    TsconfigContent,
    /// The Vize configuration the stage reads (Vue line, dialect options).
    ProjectConfig,
    /// The Vize toolchain version that produced the artifact.
    ToolchainVersion,
    /// The Corsa (TypeScript) build version.
    CorsaVersion,
    /// Feature flags that change the stage's output.
    FeatureFlags,
    /// The host platform (target triple).
    Platform,
}

impl AmbientInput {
    /// Every ambient input, in manifest order.
    pub const ALL: [Self; 7] = [
        Self::ProjectIdentity,
        Self::TsconfigContent,
        Self::ProjectConfig,
        Self::ToolchainVersion,
        Self::CorsaVersion,
        Self::FeatureFlags,
        Self::Platform,
    ];

    /// The stable spelling used in `key-manifests.md`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::ProjectIdentity => "project-identity",
            Self::TsconfigContent => "tsconfig-content",
            Self::ProjectConfig => "project-config",
            Self::ToolchainVersion => "toolchain-version",
            Self::CorsaVersion => "corsa-version",
            Self::FeatureFlags => "feature-flags",
            Self::Platform => "platform",
        }
    }
}

/// Every cached artifact, with its declared ambient inputs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum CachedArtifact {
    /// An S0 source block (the SFC split).
    SourceBlock,
    /// An S1 surface page.
    SurfacePage,
    /// An S2 page.
    SemanticPage,
    /// A block's virtual TypeScript projection segment (P5-7).
    VirtualTsProjection,
    /// A Corsa `ProjectSession` reused across `vize check` runs (P5-8).
    CorsaSession,
}

impl CachedArtifact {
    /// Every cached artifact, in `key-manifests.md` order.
    pub const ALL: [Self; 5] = [
        Self::SourceBlock,
        Self::SurfacePage,
        Self::SemanticPage,
        Self::VirtualTsProjection,
        Self::CorsaSession,
    ];

    /// The stable spelling used in `key-manifests.md`.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::SourceBlock => "s0.source-block",
            Self::SurfacePage => "s1.surface-page",
            Self::SemanticPage => "s2.page",
            Self::VirtualTsProjection => "projection.virtual-ts",
            Self::CorsaSession => "corsa.session",
        }
    }

    /// The stages whose [`ArtifactKey`]s may be this artifact's content key:
    /// a projection segment is keyed by its script block's S0 key or its
    /// template's S2 page key; a Corsa session has no content key and is
    /// keyed by its manifest alone ([`KeyManifest::fingerprint`]).
    #[must_use]
    pub const fn content_stages(self) -> &'static [Stage] {
        match self {
            Self::SourceBlock => &[Stage::Source],
            Self::SurfacePage => &[Stage::Surface],
            Self::SemanticPage => &[Stage::Semantic],
            Self::VirtualTsProjection => &[Stage::Source, Stage::Semantic],
            Self::CorsaSession => &[],
        }
    }

    /// The ambient inputs the artifact reads, in manifest order.
    #[must_use]
    pub const fn inputs(self) -> &'static [AmbientInput] {
        use AmbientInput::{
            CorsaVersion, FeatureFlags, Platform, ProjectConfig, ProjectIdentity, ToolchainVersion,
            TsconfigContent,
        };
        match self {
            Self::SourceBlock => &[ToolchainVersion],
            Self::SurfacePage => &[ToolchainVersion, FeatureFlags],
            Self::SemanticPage => &[ProjectConfig, ToolchainVersion, FeatureFlags],
            Self::VirtualTsProjection => &[
                TsconfigContent,
                ProjectConfig,
                ToolchainVersion,
                FeatureFlags,
            ],
            Self::CorsaSession => &[
                ProjectIdentity,
                TsconfigContent,
                ToolchainVersion,
                CorsaVersion,
                FeatureFlags,
                Platform,
            ],
        }
    }
}

/// Why a manifest cannot key an artifact.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ManifestError {
    /// The content key belongs to another stage than the artifact.
    WrongStage {
        /// The artifact being keyed.
        artifact: CachedArtifact,
        /// The content key's stage.
        stage: Stage,
    },
    /// The manifest does not declare exactly the artifact's inputs.
    Undeclared {
        /// The artifact being keyed.
        artifact: CachedArtifact,
        /// Declared inputs the manifest lacks (the corruption bug).
        missing: Vec<AmbientInput>,
        /// Manifest inputs the artifact does not declare.
        extra: Vec<AmbientInput>,
    },
}

/// Values of ambient inputs, one per input, in manifest order.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash)]
pub struct KeyManifest {
    values: Vec<(AmbientInput, String)>,
}

impl KeyManifest {
    /// An empty manifest.
    #[must_use]
    pub const fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// Set `input` to `value` (replacing an earlier value).
    #[must_use]
    pub fn with(mut self, input: AmbientInput, value: &str) -> Self {
        self.set(input, value);
        self
    }

    /// Set `input` to `value` (replacing an earlier value).
    pub fn set(&mut self, input: AmbientInput, value: &str) {
        match self.values.binary_search_by_key(&input, |(held, _)| *held) {
            Ok(at) => self.values[at].1 = String::from(value),
            Err(at) => self.values.insert(at, (input, String::from(value))),
        }
    }

    /// The value of `input`, if set.
    #[must_use]
    pub fn get(&self, input: AmbientInput) -> Option<&str> {
        self.values
            .binary_search_by_key(&input, |(held, _)| *held)
            .ok()
            .map(|at| self.values[at].1.as_str())
    }

    /// Check that the manifest sets exactly `artifact`'s declared inputs.
    pub fn check(&self, artifact: CachedArtifact) -> Result<(), ManifestError> {
        let declared = artifact.inputs();
        let missing: Vec<AmbientInput> = declared
            .iter()
            .copied()
            .filter(|input| self.get(*input).is_none())
            .collect();
        let extra: Vec<AmbientInput> = self
            .values
            .iter()
            .map(|(input, _)| *input)
            .filter(|input| !declared.contains(input))
            .collect();
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
        sink.feed_u32(self.values.len() as u32);
        for (input, value) in &self.values {
            sink.feed_str(input.name());
            sink.feed_str(value.as_str());
        }
    }
}

impl ArtifactKey {
    /// The cache key of `artifact`: this content key with the manifest's
    /// ambient inputs folded in. Stage and recipe version are kept; only the
    /// hash changes. Fails unless the manifest declares exactly the
    /// artifact's inputs and the content key is one of the artifact's
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

/// Fold markers: a manifest over a content key, or a manifest alone.
const WITH_CONTENT: u8 = 1;
const NO_CONTENT: u8 = 0;
