use vize_carton::{String, append, cstr};

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
    pub mapping_count: usize,
    pub mappings_sha256: String,
    pub semantic_link_count: usize,
    pub semantic_links_sha256: String,
    pub diagnostic_count: usize,
    pub diagnostics_sha256: String,
    pub authored_hit_count: usize,
    pub authored_hits_sha256: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Drift {
    Mapping,
    Diagnostic,
    Other,
}

pub fn capture_fixture(fixture: &Fixture) -> ProjectionRecord {
    let source = fixture.source();
    if fixture.legacy_vue2 && !cfg!(feature = "legacy") {
        let disabled = LaneRecord::disabled("feature-disabled:vize_maestro/legacy");
        return ProjectionRecord {
            fixture: fixture.id.clone(),
            source_sha256: sha256(&source),
            canon: disabled.clone(),
            content_mapper: disabled.clone(),
            maestro: disabled,
        };
    }

    let mapper = capture_content_mapper(fixture, &source);
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
            mapping_count: 0,
            mappings_sha256: sha256(""),
            semantic_link_count: 0,
            semantic_links_sha256: sha256(""),
            diagnostic_count: 0,
            diagnostics_sha256: sha256(""),
            authored_hit_count: 0,
            authored_hits_sha256: sha256(""),
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
        if fixture.legacy_vue2 && !cfg!(feature = "legacy") {
            assert!(self.canon.status.starts_with("feature-disabled:"));
            return;
        }
        assert!(self.content_mapper.text_bytes > 0);
        if fixture
            .coverage
            .iter()
            .any(|coverage| coverage == "recovery")
        {
            assert!(self.content_mapper.diagnostic_count > 0);
            return;
        }
        assert_eq!(self.canon.status, "ok");
        assert_eq!(self.maestro.status, "ok");
        assert!(self.canon.mapping_count > 0);
        assert!(self.content_mapper.mapping_count > 0);
        assert!(self.maestro.mapping_count > 0);
        assert!(self.content_mapper.authored_hit_count > 0);
        assert!(self.maestro.authored_hit_count > 0);
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
                "[{name}]\nstatus={}\ntext={}:{}\nmappings={}:{}\nsemantic-links={}:{}\ndiagnostics={}:{}\nauthored-hits={}:{}\n",
                lane.status,
                lane.text_bytes,
                lane.text_sha256,
                lane.mapping_count,
                lane.mappings_sha256,
                lane.semantic_link_count,
                lane.semantic_links_sha256,
                lane.diagnostic_count,
                lane.diagnostics_sha256,
                lane.authored_hit_count,
                lane.authored_hits_sha256
            );
        }
        out
    }

    pub fn canary() -> Self {
        let lane = LaneRecord::disabled("ok");
        Self {
            fixture: "canary".into(),
            source_sha256: sha256("source"),
            canon: lane.clone(),
            content_mapper: lane.clone(),
            maestro: lane,
        }
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
