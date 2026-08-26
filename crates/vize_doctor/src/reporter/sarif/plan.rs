//! Fail-closed SARIF preflight and deterministic render indexing.

use std::collections::{BTreeMap, BTreeSet};

use vize_s0::{String, ToCompactString, cstr};

use super::source::{IndexedSource, SarifMissingSourcePolicy, validate_path};
use crate::{
    DoctorFinding, DoctorReport, FixSafety, ReporterError, ReporterErrorKind, SourceLocation,
};

pub(super) struct ArtifactPlan<'source> {
    uri: String,
    source: Option<&'source IndexedSource<'source>>,
}

impl ArtifactPlan<'_> {
    pub(super) fn uri(&self) -> &str {
        &self.uri
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct SarifRegion {
    start_line: u32,
    start_column: u32,
    end_line: u32,
    end_column: u32,
}

pub(super) struct SarifPlan<'report, 'source> {
    report: &'report DoctorReport,
    artifacts: BTreeMap<&'report str, ArtifactPlan<'source>>,
    rules: Vec<&'report DoctorFinding>,
}

impl<'report, 'source> SarifPlan<'report, 'source> {
    pub(super) fn new(
        report: &'report DoctorReport,
        sources: &'source BTreeMap<&'source str, IndexedSource<'source>>,
        missing_source_policy: SarifMissingSourcePolicy,
    ) -> Result<Self, ReporterError> {
        let mut paths = BTreeSet::new();
        let mut rule_map = BTreeMap::new();

        for finding in report.findings() {
            validate_rule_id(&finding.code)?;
            rule_map.entry(finding.code.as_str()).or_insert(finding);
            paths.insert(finding.primary.path.as_str());
            paths.extend(
                finding
                    .related
                    .iter()
                    .map(|related| related.location.path.as_str()),
            );
            paths.extend(
                finding
                    .evidence
                    .iter()
                    .filter_map(|evidence| evidence.location.as_ref())
                    .map(|location| location.path.as_str()),
            );
            if let Some(fix) = &finding.fix {
                if fix.safety == FixSafety::Unavailable && !fix.edits.is_empty() {
                    return Err(invalid_data(cstr!(
                        "finding {} has source edits but marks its fix unavailable",
                        finding.code
                    )));
                }
                validate_edits(finding)?;
                paths.extend(fix.edits.iter().map(|edit| edit.location.path.as_str()));
            }
        }

        let mut artifacts = BTreeMap::new();
        for path in paths {
            validate_path(path).map_err(|error| invalid_data(error.to_compact_string()))?;
            let source = sources.get(path);
            if source.is_none() && missing_source_policy == SarifMissingSourcePolicy::Reject {
                return Err(invalid_data(cstr!(
                    "SARIF source {} is required to translate its UTF-8 byte spans",
                    path
                )));
            }
            artifacts.insert(
                path,
                ArtifactPlan {
                    uri: encode_relative_uri(path),
                    source,
                },
            );
        }

        let plan = Self {
            report,
            artifacts,
            rules: rule_map.into_values().collect(),
        };
        plan.validate_locations()?;
        Ok(plan)
    }

    pub(super) const fn report(&self) -> &'report DoctorReport {
        self.report
    }

    pub(super) fn rules(&self) -> &[&'report DoctorFinding] {
        &self.rules
    }

    pub(super) fn rule_index(&self, code: &str) -> usize {
        self.rules
            .binary_search_by_key(&code, |finding| finding.code.as_str())
            .expect("preflight indexed every finding rule")
    }

    pub(super) fn artifact(&self, path: &str) -> &ArtifactPlan<'source> {
        self.artifacts
            .get(path)
            .expect("preflight indexed every finding artifact")
    }

    pub(super) fn region(&self, location: &SourceLocation) -> Option<SarifRegion> {
        let source = self.artifact(&location.path).source?;
        let (start_line, start_column) = source.position(location.start)?;
        let (end_line, end_column) = source.position(location.end)?;
        Some(SarifRegion {
            start_line,
            start_column,
            end_line,
            end_column,
        })
    }

    pub(super) fn can_render_fix(&self, fix: &crate::FindingFix) -> bool {
        fix.edits
            .iter()
            .all(|edit| self.region(&edit.location).is_some())
    }

    fn validate_locations(&self) -> Result<(), ReporterError> {
        for finding in self.report.findings() {
            self.validate_location(&finding.primary)?;
            for related in &finding.related {
                self.validate_location(&related.location)?;
            }
            for evidence in &finding.evidence {
                if let Some(location) = &evidence.location {
                    self.validate_location(location)?;
                }
            }
            if let Some(fix) = &finding.fix {
                for edit in &fix.edits {
                    self.validate_location(&edit.location)?;
                }
            }
        }
        Ok(())
    }

    fn validate_location(&self, location: &SourceLocation) -> Result<(), ReporterError> {
        let artifact = self.artifact(&location.path);
        let Some(source) = artifact.source else {
            return Ok(());
        };
        if source.position(location.start).is_none() {
            return Err(invalid_span(location, "start"));
        }
        if source.position(location.end).is_none() {
            return Err(invalid_span(location, "end"));
        }
        Ok(())
    }
}

fn validate_edits(finding: &DoctorFinding) -> Result<(), ReporterError> {
    let Some(fix) = &finding.fix else {
        return Ok(());
    };
    let mut previous: Option<&SourceLocation> = None;
    for edit in &fix.edits {
        if let Some(before) = previous
            && before.path == edit.location.path
            && (edit.location.start < before.end || edit.location.start == before.start)
        {
            return Err(invalid_data(cstr!(
                "finding {} has ambiguous overlapping edits in {}",
                finding.code,
                edit.location.path
            )));
        }
        previous = Some(&edit.location);
    }
    Ok(())
}

fn validate_rule_id(code: &str) -> Result<(), ReporterError> {
    if code.is_empty()
        || code
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        return Err(invalid_data(cstr!(
            "Doctor rule id {:?} is not a valid SARIF hierarchical string",
            code
        )));
    }
    Ok(())
}

fn invalid_span(location: &SourceLocation, boundary: &str) -> ReporterError {
    invalid_data(cstr!(
        "SARIF location {}:{}..{} has an out-of-range or non-UTF-8 {boundary} boundary",
        location.path,
        location.start,
        location.end
    ))
}

fn invalid_data(message: impl Into<String>) -> ReporterError {
    ReporterError::new(ReporterErrorKind::InvalidData, message)
}

fn encode_relative_uri(path: &str) -> String {
    const HEX: &[u8; 16] = b"0123456789ABCDEF";
    let mut encoded = String::with_capacity(path.len());
    for byte in path.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'/') {
            encoded.push(byte as char);
        } else {
            encoded.push('%');
            encoded.push(HEX[(byte >> 4) as usize] as char);
            encoded.push(HEX[(byte & 0x0f) as usize] as char);
        }
    }
    encoded
}

#[cfg(test)]
mod unit_tests {
    use super::encode_relative_uri;

    #[test]
    fn artifact_uris_are_relative_and_utf8_percent_encoded() {
        assert_eq!(
            encode_relative_uri("src/画面 #1%.vue"),
            "src/%E7%94%BB%E9%9D%A2%20%231%25.vue"
        );
    }
}
