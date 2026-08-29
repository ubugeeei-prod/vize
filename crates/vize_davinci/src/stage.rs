//! Canonical Davinci stage names and their current crate spellings.
//!
//! Human-facing implementation names are the short stage aliases (`s0`, `s1`,
//! `s2`, `s1_to_s2`). S1 and S2 have been mechanically renamed to their stage
//! names; remaining art-name packages stay visible until their own rename PRs
//! land.

/// The stage a Davinci diagnostic came from.
///
/// The variants retain the logical names used by existing diagnostics and
/// folio prose. Use [`Stage::physical_id`] when a physical layer id is needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stage {
    /// S0: reading the authored file into source coordinates.
    Source,
    /// S1: the lossless surface tree.
    Surface,
    /// S2: the semantic IR.
    Semantic,
    /// S3: the lowered backend IR.
    Lowered,
    /// S4: emission.
    Emit,
}

impl Stage {
    /// Stable logical identifier used by today's diagnostic output.
    #[inline]
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Stage::Source => "source",
            Stage::Surface => "surface",
            Stage::Semantic => "semantic",
            Stage::Lowered => "lowered",
            Stage::Emit => "emit",
        }
    }

    /// Physical layer identifier for implementation names and stage-local keys.
    #[inline]
    #[must_use]
    pub const fn physical_id(self) -> &'static str {
        match self {
            Stage::Source => "s0",
            Stage::Surface => "s1",
            Stage::Semantic => "s2",
            Stage::Lowered => "s3",
            Stage::Emit => "s4",
        }
    }
}

/// A Davinci layer crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerCrate {
    /// Short stage id used in implementation names and docs.
    pub id: &'static str,
    /// Preferred Rust dependency alias for new code.
    pub crate_alias: &'static str,
    /// Current Cargo package id.
    pub package: &'static str,
    /// Stable one-line role.
    pub role: &'static str,
}

/// Historical artifact-stage type name retained for compatibility.
///
/// New code should use [`LayerCrate`] because S0 is a layer without being an
/// artifact-producing stage.
pub type StageCrate = LayerCrate;

/// A Davinci conversion crate between two stage artifacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConversionCrate {
    /// Short conversion id used in implementation names and docs.
    pub id: &'static str,
    /// Preferred Rust dependency alias for new code.
    pub crate_alias: &'static str,
    /// Current Cargo package id.
    pub package: &'static str,
    /// Input stage id.
    pub from: &'static str,
    /// Output stage id.
    pub to: &'static str,
    /// Stable one-line role.
    pub role: &'static str,
}

/// S0: the source, arena, compact-storage, and span foundation.
pub const S0: LayerCrate = LayerCrate {
    id: "s0",
    crate_alias: "vize_s0",
    package: "vize_carton",
    role: "source model and compiler storage foundation",
};

/// S1: the lossless Vue-template surface tree.
pub const S1: LayerCrate = LayerCrate {
    id: "s1",
    crate_alias: "vize_s1",
    package: "vize_s1",
    role: "lossless Vue-template surface tree",
};

/// S2: the semantic IR.
pub const S2: LayerCrate = LayerCrate {
    id: "s2",
    crate_alias: "vize_s2",
    package: "vize_s2",
    role: "semantic UI IR",
};

/// S1→S2: Vue lowering from the lossless surface tree into the semantic IR.
pub const S1_TO_S2: ConversionCrate = ConversionCrate {
    id: "s1_to_s2",
    crate_alias: "vize_s1_to_s2",
    package: "vize_s1_to_s2",
    from: S1.id,
    to: S2.id,
    role: "Vue surface-to-semantic lowering",
};

/// Davinci layer crates that exist in the workspace today.
pub const LAYERS: &[LayerCrate] = &[S0, S1, S2];

/// Historical artifact-only view retained for compatibility.
///
/// New code should use [`LAYERS`] when it needs the complete S0/S1/S2 stack.
pub const ARTIFACT_STAGES: &[StageCrate] = &[S1, S2];

/// Conversion crates that exist in the workspace today.
pub const CONVERSIONS: &[ConversionCrate] = &[S1_TO_S2];

#[cfg(test)]
mod tests {
    use super::{ARTIFACT_STAGES, CONVERSIONS, LAYERS, S0, S1, S1_TO_S2, S2, Stage, StageCrate};

    #[test]
    fn diagnostic_stage_identifiers_remain_logical_names() {
        assert_eq!(Stage::Source.as_str(), "source");
        assert_eq!(Stage::Surface.as_str(), "surface");
        assert_eq!(Stage::Semantic.as_str(), "semantic");
        assert_eq!(Stage::Lowered.as_str(), "lowered");
        assert_eq!(Stage::Emit.as_str(), "emit");
    }

    #[test]
    fn physical_stage_identifiers_are_s0_to_s4() {
        assert_eq!(Stage::Source.physical_id(), "s0");
        assert_eq!(Stage::Surface.physical_id(), "s1");
        assert_eq!(Stage::Semantic.physical_id(), "s2");
        assert_eq!(Stage::Lowered.physical_id(), "s3");
        assert_eq!(Stage::Emit.physical_id(), "s4");
    }

    #[test]
    fn stage_aliases_are_the_preferred_implementation_names() {
        assert_eq!(S0.crate_alias, "vize_s0");
        assert_eq!(S1.crate_alias, "vize_s1");
        assert_eq!(S2.crate_alias, "vize_s2");
        assert_eq!(S1_TO_S2.crate_alias, "vize_s1_to_s2");
        assert_eq!(S1_TO_S2.id, "s1_to_s2");
    }

    #[test]
    fn package_ids_match_the_current_workspace_spelling() {
        assert_eq!(S0.package, "vize_carton");
        assert_eq!(S1.package, "vize_s1");
        assert_eq!(S2.package, "vize_s2");
        assert_eq!(S1_TO_S2.package, "vize_s1_to_s2");
    }

    #[test]
    fn conversion_names_its_stage_edges() {
        assert_eq!((S1_TO_S2.from, S1_TO_S2.to), (S1.id, S2.id));
        assert_eq!(LAYERS.len(), 3);
        assert_eq!(CONVERSIONS.len(), 1);
    }

    #[test]
    fn historical_artifact_view_excludes_the_s0_foundation() {
        let _: StageCrate = S1;
        assert_eq!(ARTIFACT_STAGES, [S1, S2]);
    }
}
