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
//! `docs/davinci/plan/key-manifests.md`). A [`KeyManifest`] carries their
//! values' digests and folds into a key only when it sets **exactly** the
//! artifact's inputs: a missing one is the corruption bug, an extra one is an
//! undeclared input — both are errors, never silently accepted.

use crate::stage::Stage;

mod fold;

pub use fold::{InputSet, KeyManifest, ManifestError};

/// One ambient input. The discriminants are the manifest order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
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
    /// The JS plugin and source file identities seen by a plugin result.
    PluginIdentity,
    /// The JS plugin's declared version.
    PluginVersion,
    /// The digest of the JS plugin's rule sources.
    PluginCode,
    /// The node kinds the JS plugin visits.
    PluginVisits,
    /// The fact groups the JS plugin demands.
    PluginDemands,
}

impl AmbientInput {
    /// Every ambient input, in manifest order.
    pub const ALL: [Self; 12] = [
        Self::ProjectIdentity,
        Self::TsconfigContent,
        Self::ProjectConfig,
        Self::ToolchainVersion,
        Self::CorsaVersion,
        Self::FeatureFlags,
        Self::Platform,
        Self::PluginIdentity,
        Self::PluginVersion,
        Self::PluginCode,
        Self::PluginVisits,
        Self::PluginDemands,
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
            Self::PluginIdentity => "plugin-identity",
            Self::PluginVersion => "plugin-version",
            Self::PluginCode => "plugin-code",
            Self::PluginVisits => "plugin-visits",
            Self::PluginDemands => "plugin-demands",
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
    /// A JS plugin's diagnostics for one source file (P5-13).
    PluginResult,
}

impl CachedArtifact {
    /// Every cached artifact, in `key-manifests.md` order.
    pub const ALL: [Self; 6] = [
        Self::SourceBlock,
        Self::SurfacePage,
        Self::SemanticPage,
        Self::VirtualTsProjection,
        Self::CorsaSession,
        Self::PluginResult,
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
            Self::PluginResult => "plugin.result",
        }
    }

    /// The stages whose [`ArtifactKey`](crate::key::ArtifactKey)s may be this artifact's content key:
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
            Self::PluginResult => &[Stage::Source],
        }
    }

    /// The ambient inputs the artifact reads, in manifest order.
    #[must_use]
    pub const fn inputs(self) -> &'static [AmbientInput] {
        use AmbientInput::{
            CorsaVersion, FeatureFlags, Platform, PluginCode, PluginDemands, PluginIdentity,
            PluginVersion, PluginVisits, ProjectConfig, ProjectIdentity, ToolchainVersion,
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
            Self::PluginResult => &[
                ToolchainVersion,
                FeatureFlags,
                PluginIdentity,
                PluginVersion,
                PluginCode,
                PluginVisits,
                PluginDemands,
            ],
        }
    }
}
