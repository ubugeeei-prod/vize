use vize_carton::{SmallVec, String, append, cstr};

use super::canon::{capture_canon, capture_content_mapper};
use super::maestro::capture_maestro;
use super::matrix::Fixture;
use super::normalize::{normalized_error, sha256};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProjectionRecord {
    pub fixture: String,
    pub source_sha256: String,
    pub canon: LaneRecord,
    pub content_mapper: LaneRecord,
    pub maestro: LaneRecord,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LaneRecord {
    pub status: String,
    pub text_bytes: usize,
    pub text_sha256: String,
    pub pre_rewrite_text_bytes: usize,
    pub pre_rewrite_text_sha256: String,
    pub import_rewrite_count: usize,
    pub import_source_map_sha256: String,
    pub import_source_map_probe_count: usize,
    pub import_source_map_probes_sha256: String,
    pub mapping_count: usize,
    pub mappings_sha256: String,
    pub semantic_link_count: usize,
    pub semantic_links_sha256: String,
    pub diagnostic_count: usize,
    pub diagnostics_sha256: String,
    pub authored_hit_count: usize,
    pub authored_hits_sha256: String,
    pub authored_hit_anchors: SmallVec<[String; 8]>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drift {
    Mapping,
    Diagnostic,
    Other,
}

pub fn capture_fixture(fixture: &Fixture) -> ProjectionRecord {
    let source = fixture.source();
    let mapper = capture_content_mapper(fixture, &source);
    if fixture.legacy_vue2 && !cfg!(feature = "legacy") {
        return ProjectionRecord {
            fixture: fixture.id.clone(),
            source_sha256: sha256(&source),
            canon: LaneRecord::disabled("feature-disabled:vize_canon/legacy"),
            content_mapper: mapper,
            maestro: LaneRecord::disabled("feature-disabled:vize_maestro/legacy"),
        };
    }

    ProjectionRecord {
        fixture: fixture.id.clone(),
        source_sha256: sha256(&source),
        canon: capture_canon(fixture, &source, &mapper),
        content_mapper: mapper,
        maestro: capture_maestro(fixture, &source),
    }
}

impl LaneRecord {
    pub(super) fn disabled(status: &str) -> Self {
        Self {
            status: status.into(),
            text_bytes: 0,
            text_sha256: sha256(""),
            pre_rewrite_text_bytes: 0,
            pre_rewrite_text_sha256: sha256(""),
            import_rewrite_count: 0,
            import_source_map_sha256: sha256(""),
            import_source_map_probe_count: 0,
            import_source_map_probes_sha256: sha256(""),
            mapping_count: 0,
            mappings_sha256: sha256(""),
            semantic_link_count: 0,
            semantic_links_sha256: sha256(""),
            diagnostic_count: 0,
            diagnostics_sha256: sha256(""),
            authored_hit_count: 0,
            authored_hits_sha256: sha256(""),
            authored_hit_anchors: SmallVec::new(),
        }
    }

    pub(super) fn error(error: impl core::fmt::Display) -> Self {
        let status = cstr!("error:{}", normalized_error(error));
        Self {
            status,
            ..Self::disabled("error")
        }
    }
}

impl ProjectionRecord {
    pub fn assert_non_empty(&self, fixture: &Fixture) {
        assert!(!self.fixture.is_empty());
        assert_eq!(self.source_sha256.len(), 64);
        assert!(self.content_mapper.text_bytes > 0);
        assert_eq!(
            self.content_mapper.status,
            if fixture.legacy_vue2 {
                "ok:vue3-fixed-production"
            } else {
                "ok"
            }
        );
        if fixture.legacy_vue2 && !cfg!(feature = "legacy") {
            assert_eq!(self.canon.status, "feature-disabled:vize_canon/legacy");
            assert_eq!(self.maestro.status, "feature-disabled:vize_maestro/legacy");
            assert!(self.content_mapper.mapping_count > 0);
            assert_exact_authored_hit_anchors("content-mapper", fixture, &self.content_mapper);
            return;
        }
        if fixture
            .coverage
            .iter()
            .any(|coverage| coverage == "recovery")
        {
            assert!(self.content_mapper.diagnostic_count > 0);
            return;
        }
        assert_eq!(
            self.canon.status,
            if fixture.legacy_vue2 {
                "ok:legacy-feature-projection"
            } else {
                "ok"
            }
        );
        assert_eq!(
            self.maestro.status,
            if fixture.legacy_vue2 {
                "ok:legacy-feature-projection"
            } else {
                "ok"
            }
        );
        assert!(self.canon.mapping_count > 0);
        assert!(self.content_mapper.mapping_count > 0);
        assert!(self.maestro.mapping_count > 0);
        assert_exact_authored_hit_anchors("content-mapper", fixture, &self.content_mapper);
        assert_exact_authored_hit_anchors("maestro", fixture, &self.maestro);
        if fixture
            .coverage
            .iter()
            .any(|coverage| coverage == "local-vue-import")
        {
            assert!(self.canon.import_rewrite_count > 0);
            assert_ne!(self.canon.pre_rewrite_text_sha256, self.canon.text_sha256);
        }
    }

    pub fn render(&self) -> String {
        let mut out = cstr!(
            "fixture={}\nsource-sha256={}\n",
            self.fixture,
            self.source_sha256
        );
        for (name, lane) in [
            ("canon", &self.canon),
            ("content-mapper", &self.content_mapper),
            ("maestro", &self.maestro),
        ] {
            append!(
                out,
                "[{name}]\nstatus={}\ntext={}:{}\npre-rewrite-text={}:{}\nimport-source-map={}:{}\nimport-source-map-probes={}:{}\nmappings={}:{}\nsemantic-links={}:{}\ndiagnostics={}:{}\nauthored-hits={}:{}\nauthored-hit-anchors={:?}\n",
                lane.status,
                lane.text_bytes,
                lane.text_sha256,
                lane.pre_rewrite_text_bytes,
                lane.pre_rewrite_text_sha256,
                lane.import_rewrite_count,
                lane.import_source_map_sha256,
                lane.import_source_map_probe_count,
                lane.import_source_map_probes_sha256,
                lane.mapping_count,
                lane.mappings_sha256,
                lane.semantic_link_count,
                lane.semantic_links_sha256,
                lane.diagnostic_count,
                lane.diagnostics_sha256,
                lane.authored_hit_count,
                lane.authored_hits_sha256,
                lane.authored_hit_anchors
            );
        }
        out
    }
}

fn assert_exact_authored_hit_anchors(lane_name: &str, fixture: &Fixture, lane: &LaneRecord) {
    assert_eq!(
        lane.authored_hit_anchors.len(),
        fixture.anchors.len(),
        "{}: {lane_name} must map every declared anchor; expected {:?}, got {:?}",
        fixture.id,
        fixture.anchors,
        lane.authored_hit_anchors
    );
    for (actual, expected) in lane.authored_hit_anchors.iter().zip(fixture.anchors.iter()) {
        assert_eq!(
            actual, expected,
            "{}: {lane_name} authored anchor identity or order drifted",
            fixture.id
        );
    }
}

pub fn verify_exact(expected: &ProjectionRecord, actual: &ProjectionRecord) -> Result<(), Drift> {
    if expected.canon.mappings_sha256 != actual.canon.mappings_sha256
        || expected.content_mapper.mappings_sha256 != actual.content_mapper.mappings_sha256
        || expected.maestro.mappings_sha256 != actual.maestro.mappings_sha256
    {
        return Err(Drift::Mapping);
    }
    if expected.canon.diagnostics_sha256 != actual.canon.diagnostics_sha256
        || expected.content_mapper.diagnostics_sha256 != actual.content_mapper.diagnostics_sha256
        || expected.maestro.diagnostics_sha256 != actual.maestro.diagnostics_sha256
    {
        return Err(Drift::Diagnostic);
    }
    if expected == actual {
        Ok(())
    } else {
        Err(Drift::Other)
    }
}
