//! Canonical Davinci stage names and their current crate spellings.
//!
//! Human-facing implementation names use `l0` through `l4`. Serialized
//! artifact identifiers retain `s0` through `s4` so existing keys and wire
//! documents keep their byte identity.

/// The stage a Davinci diagnostic came from.
///
/// The variants retain the logical names used by existing diagnostics and
/// folio prose. Use [`Stage::physical_id`] when a physical layer id is needed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Stage {
    /// L0: reading the authored file into source coordinates.
    Source,
    /// L1: the lossless surface tree.
    Surface,
    /// L2: the semantic IR.
    Semantic,
    /// L3: the lowered backend IR.
    Lowered,
    /// L4: emission.
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

    /// Stable schema identifier used in serialized artifacts and key hashes.
    #[inline]
    #[must_use]
    pub const fn wire_id(self) -> &'static str {
        match self {
            Stage::Source => "s0",
            Stage::Surface => "s1",
            Stage::Semantic => "s2",
            Stage::Lowered => "s3",
            Stage::Emit => "s4",
        }
    }

    /// Physical layer identifier for implementation names and user interfaces.
    #[inline]
    #[must_use]
    pub const fn physical_id(self) -> &'static str {
        match self {
            Stage::Source => "l0",
            Stage::Surface => "l1",
            Stage::Semantic => "l2",
            Stage::Lowered => "l3",
            Stage::Emit => "l4",
        }
    }
}

/// Bind canonical layer pipeline selectors to their stable schema identifiers.
/// Unknown identifiers stay available to caller-defined pipeline catalogues.
#[must_use]
pub fn pipeline_wire_id(id: &str) -> &str {
    match id {
        "l0" => "s0",
        "l1" => "s1",
        "l2" => "s2",
        "l3" => "s3",
        "l4" => "s4",
        "l1-to-l2" => "s1-to-s2",
        "l2-to-l3" => "s2-to-s3",
        _ => id,
    }
}

/// Canonical human-facing spelling of a pipeline's stable stage identifier.
#[must_use]
pub fn pipeline_display_id(id: &str) -> &str {
    match id {
        "s0" => "l0",
        "s1" => "l1",
        "s2" => "l2",
        "s3" => "l3",
        "s4" => "l4",
        "s1-to-s2" => "l1-to-l2",
        "s2-to-s3" => "l2-to-l3",
        _ => id,
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
/// New code should use [`LayerCrate`] because L0 is a layer without being an
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

/// L0: the source, arena, compact-storage, and span foundation.
pub const L0: LayerCrate = LayerCrate {
    id: "l0",
    crate_alias: "vize_l0",
    package: "vize_carton",
    role: "source model and compiler storage foundation",
};

/// L1: the lossless Vue-template surface tree.
pub const L1: LayerCrate = LayerCrate {
    id: "l1",
    crate_alias: "vize_l1",
    package: "vize_l1",
    role: "lossless Vue-template surface tree",
};

/// L2: the semantic IR.
pub const L2: LayerCrate = LayerCrate {
    id: "l2",
    crate_alias: "vize_l2",
    package: "vize_l2",
    role: "semantic UI IR",
};

/// L3: reactivity and backend scheduling IR.
pub const L3: LayerCrate = LayerCrate {
    id: "l3",
    crate_alias: "vize_l3",
    package: "vize_impeto",
    role: "reactivity and backend scheduling IR",
};

/// L1→L2: Vue lowering from the lossless surface tree into the semantic IR.
pub const L1_TO_L2: ConversionCrate = ConversionCrate {
    id: "l1_to_l2",
    crate_alias: "vize_l1_to_l2",
    package: "vize_l1_to_l2",
    from: L1.id,
    to: L2.id,
    role: "Vue surface-to-semantic lowering",
};

/// Davinci layer crates that exist in the workspace today.
pub const LAYERS: &[LayerCrate] = &[L0, L1, L2, L3];

/// Historical artifact-only view retained for compatibility.
///
/// New code should use [`LAYERS`] when it needs the complete L0/L1/L2/L3 stack.
pub const ARTIFACT_STAGES: &[StageCrate] = &[L1, L2, L3];

/// Conversion crates that exist in the workspace today.
pub const CONVERSIONS: &[ConversionCrate] = &[L1_TO_L2];

#[cfg(test)]
mod tests {
    use super::{
        ARTIFACT_STAGES, CONVERSIONS, L0, L1, L1_TO_L2, L2, L3, LAYERS, Stage, StageCrate,
        pipeline_display_id, pipeline_wire_id,
    };

    #[test]
    fn diagnostic_stage_identifiers_remain_logical_names() {
        assert_eq!(Stage::Source.as_str(), "source");
        assert_eq!(Stage::Surface.as_str(), "surface");
        assert_eq!(Stage::Semantic.as_str(), "semantic");
        assert_eq!(Stage::Lowered.as_str(), "lowered");
        assert_eq!(Stage::Emit.as_str(), "emit");
    }

    #[test]
    fn physical_layer_identifiers_are_l0_to_l4() {
        assert_eq!(Stage::Source.physical_id(), "l0");
        assert_eq!(Stage::Surface.physical_id(), "l1");
        assert_eq!(Stage::Semantic.physical_id(), "l2");
        assert_eq!(Stage::Lowered.physical_id(), "l3");
        assert_eq!(Stage::Emit.physical_id(), "l4");
    }

    #[test]
    fn wire_stage_identifiers_remain_s0_to_s4() {
        assert_eq!(Stage::Source.wire_id(), "s0");
        assert_eq!(Stage::Surface.wire_id(), "s1");
        assert_eq!(Stage::Semantic.wire_id(), "s2");
        assert_eq!(Stage::Lowered.wire_id(), "s3");
        assert_eq!(Stage::Emit.wire_id(), "s4");
    }

    #[test]
    fn pipeline_binding_keeps_legacy_and_custom_identifiers() {
        assert_eq!(pipeline_wire_id("l2"), "s2");
        assert_eq!(pipeline_wire_id("l2-to-l3"), "s2-to-s3");
        assert_eq!(pipeline_wire_id("s2"), "s2");
        assert_eq!(pipeline_wire_id("custom-stage"), "custom-stage");
        assert_eq!(pipeline_display_id("s2"), "l2");
        assert_eq!(pipeline_display_id("s2-to-s3"), "l2-to-l3");
        assert_eq!(pipeline_display_id("l2"), "l2");
        assert_eq!(pipeline_display_id("custom-stage"), "custom-stage");
    }

    #[test]
    fn stage_aliases_are_the_preferred_implementation_names() {
        assert_eq!(L0.crate_alias, "vize_l0");
        assert_eq!(L1.crate_alias, "vize_l1");
        assert_eq!(L2.crate_alias, "vize_l2");
        assert_eq!(L3.crate_alias, "vize_l3");
        assert_eq!(L1_TO_L2.crate_alias, "vize_l1_to_l2");
        assert_eq!(L1_TO_L2.id, "l1_to_l2");
    }

    #[test]
    fn package_ids_match_the_current_workspace_spelling() {
        assert_eq!(L0.package, "vize_carton");
        assert_eq!(L1.package, "vize_l1");
        assert_eq!(L2.package, "vize_l2");
        assert_eq!(L3.package, "vize_impeto");
        assert_eq!(L1_TO_L2.package, "vize_l1_to_l2");
    }

    #[test]
    fn conversion_names_its_stage_edges() {
        assert_eq!((L1_TO_L2.from, L1_TO_L2.to), (L1.id, L2.id));
        assert_eq!(LAYERS.len(), 4);
        assert_eq!(CONVERSIONS.len(), 1);
    }

    #[test]
    fn historical_artifact_view_excludes_the_l0_foundation() {
        let _: StageCrate = L1;
        assert_eq!(ARTIFACT_STAGES, [L1, L2, L3]);
    }
}
