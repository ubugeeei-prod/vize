//! P5-1b — ambient key manifests.
//!
//! Every declared input of every cached artifact changes its key; nothing
//! else does; a manifest that declares a missing or an extra input cannot
//! key the artifact at all; and `docs/davinci/plan/key-manifests.md` lists
//! exactly the artifacts and inputs `vize_davinci::key::manifest` declares.

#![expect(
    clippy::expect_used,
    clippy::string_slice,
    reason = "tests assert by panicking"
)]

use std::fmt::Write as _;

use vize_davinci::key::{
    AmbientInput, ArtifactKey, CachedArtifact, InputSet, KeyManifest, ManifestError,
    source_block_key,
};
use vize_davinci::stage::Stage;
use vize_s0::String;

const DOC: &str = include_str!("../../../docs/davinci/plan/key-manifests.md");

/// A content key of `stage`, or `None` for an artifact keyed by its
/// manifest alone.
fn content_key(artifact: CachedArtifact) -> Option<ArtifactKey> {
    let stage = *artifact.content_stages().first()?;
    let mut sink = vize_davinci::key::KeySink::new(stage, 1, 0);
    sink.feed_str("content");
    Some(sink.finish())
}

/// A manifest setting every declared input of `artifact` to `base-<name>`.
fn full_manifest(artifact: CachedArtifact) -> KeyManifest {
    let mut manifest = KeyManifest::new();
    for input in artifact.inputs() {
        let mut value = String::from("base-");
        value.push_str(input.name());
        manifest.set(*input, &value);
    }
    manifest
}

/// The cache key of `artifact` under `manifest`.
fn key(artifact: CachedArtifact, manifest: &KeyManifest) -> Result<[u8; 16], ManifestError> {
    match content_key(artifact) {
        Some(content) => content
            .with_manifest(artifact, manifest)
            .map(|key| key.hash()),
        None => manifest.fingerprint(artifact),
    }
}

#[test]
fn every_declared_input_changes_every_artifacts_key() {
    for artifact in CachedArtifact::ALL {
        let base = key(artifact, &full_manifest(artifact)).expect("a full manifest keys");
        let mut seen = vec![base];
        for input in artifact.inputs() {
            let flipped = full_manifest(artifact).with(*input, "flipped");
            let flipped = key(artifact, &flipped).expect("a flipped manifest keys");
            assert_ne!(flipped, base, "{} / {}", artifact.name(), input.name());
            seen.push(flipped);
        }
        // Pairwise distinct: each input moves the key somewhere of its own.
        let mut distinct = seen.clone();
        distinct.sort_unstable();
        distinct.dedup();
        assert_eq!(
            distinct.len(),
            artifact.inputs().len() + 1,
            "{}",
            artifact.name()
        );
    }
}

#[test]
fn nothing_but_a_declared_value_changes_the_key() {
    for artifact in CachedArtifact::ALL {
        let base = key(artifact, &full_manifest(artifact)).expect("keys");
        // Insertion order and re-setting an unchanged value are invisible.
        let mut reversed = KeyManifest::new();
        for input in artifact.inputs().iter().rev() {
            let mut value = String::from("base-");
            value.push_str(input.name());
            reversed.set(*input, "placeholder");
            reversed.set(*input, &value);
        }
        assert_eq!(key(artifact, &reversed), Ok(base), "{}", artifact.name());
        assert_eq!(key(artifact, &full_manifest(artifact)), Ok(base));
    }
}

#[test]
fn a_manifest_with_a_missing_or_extra_input_cannot_key() {
    let artifact = CachedArtifact::SemanticPage;
    let without = KeyManifest::new()
        .with(AmbientInput::ToolchainVersion, "0.425.1")
        .with(AmbientInput::FeatureFlags, "");
    assert_eq!(
        key(artifact, &without),
        Err(ManifestError::Undeclared {
            artifact,
            missing: InputSet::of(&[AmbientInput::ProjectConfig]),
            extra: InputSet::EMPTY,
        })
    );
    let extra = full_manifest(artifact).with(AmbientInput::Platform, "aarch64-apple-darwin");
    assert_eq!(
        key(artifact, &extra),
        Err(ManifestError::Undeclared {
            artifact,
            missing: InputSet::EMPTY,
            extra: InputSet::of(&[AmbientInput::Platform]),
        })
    );
    assert_eq!(
        InputSet::of(&[AmbientInput::Platform, AmbientInput::ProjectIdentity])
            .iter()
            .collect::<Vec<_>>(),
        [AmbientInput::ProjectIdentity, AmbientInput::Platform]
    );
}

#[test]
fn a_content_key_folds_only_into_its_own_artifact() {
    let s0 = source_block_key("template", &[], "<div/>");
    let manifest = full_manifest(CachedArtifact::SemanticPage);
    assert_eq!(
        s0.with_manifest(CachedArtifact::SemanticPage, &manifest),
        Err(ManifestError::WrongStage {
            artifact: CachedArtifact::SemanticPage,
            stage: Stage::Source,
        })
    );
    let folded = s0
        .with_manifest(
            CachedArtifact::SourceBlock,
            &full_manifest(CachedArtifact::SourceBlock),
        )
        .expect("an S0 key folds into the source-block manifest");
    assert_eq!(
        (folded.stage(), folded.schema_version()),
        (s0.stage(), s0.schema_version())
    );
    assert_ne!(folded, s0);
}

/// Per table row between `<!-- {marker}:start -->` and `<!-- {marker}:end -->`,
/// the backticked names of each cell.
fn doc_rows(marker: &str) -> Vec<Vec<Vec<&'static str>>> {
    let start = DOC
        .find(&["<!-- ", marker, ":start -->"].concat())
        .expect("start");
    let end = DOC
        .find(&["<!-- ", marker, ":end -->"].concat())
        .expect("end");
    DOC[start..end]
        .lines()
        .filter(|line| line.starts_with("| `"))
        .map(|line| {
            line.split('|')
                .map(|cell| cell.split('`').skip(1).step_by(2).collect())
                .collect()
        })
        .collect()
}

#[test]
fn the_manifest_doc_lists_every_input_and_every_artifact_exactly() {
    let inputs: Vec<&str> = doc_rows("ambient-inputs")
        .iter()
        .map(|cells| cells[1][0])
        .collect();
    let declared: Vec<&str> = AmbientInput::ALL.iter().map(|input| input.name()).collect();
    assert_eq!(inputs, declared);

    let mut documented = String::default();
    for cells in doc_rows("key-manifests") {
        let (artifact, stages, inputs) = (&cells[1], &cells[2], &cells[3]);
        writeln!(
            documented,
            "{} [{}] {}",
            artifact.join(","),
            stages.join(","),
            inputs.join(",")
        )
        .expect("string write");
    }
    let mut expected = String::default();
    for artifact in CachedArtifact::ALL {
        let stages: Vec<&str> = artifact
            .content_stages()
            .iter()
            .map(|stage| stage.physical_id())
            .collect();
        let inputs: Vec<&str> = artifact.inputs().iter().map(|input| input.name()).collect();
        writeln!(
            expected,
            "{} [{}] {}",
            artifact.name(),
            stages.join(","),
            inputs.join(",")
        )
        .expect("string write");
    }
    assert_eq!(documented, expected);
}
