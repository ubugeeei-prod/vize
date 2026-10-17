//! Exact host-owned artifact projection for failed configuration evaluations.
//!
//! The loader records complete adapter spellings when it creates that file.
//! Only those spellings may project to the authored config for equality. The
//! original stderr, exit status, and displayed error remain untouched.

use std::{
    error::Error,
    fmt, io,
    path::{Path, PathBuf},
    process::{ExitStatus, Output},
};

/// An opaque identity for one failed, observed host evaluation.
///
/// This is notification metadata, never authored configuration or watch input.
/// Missing identity requires the caller's existing strict error comparison.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ConfigLoadFailureIdentity {
    config_path: PathBuf,
    status: ExitStatus,
    stderr: Vec<u8>,
}

#[derive(Debug)]
pub(super) struct NodeEvaluationFailure {
    output: Output,
    identity: Option<ConfigLoadFailureIdentity>,
}

impl NodeEvaluationFailure {
    pub(super) fn new(
        config_path: &Path,
        output: Output,
        artifacts: &[(std::string::String, std::string::String)],
    ) -> Self {
        let identity = (!artifacts.is_empty()
            && artifacts.iter().all(|(from, _)| !from.is_empty()))
        .then(|| ConfigLoadFailureIdentity {
            config_path: config_path.to_path_buf(),
            status: output.status,
            stderr: project_owned_artifacts(&output.stderr, artifacts),
        });
        Self { output, identity }
    }
}

impl fmt::Display for NodeEvaluationFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "node failed to load config: {}",
            String::from_utf8_lossy(&self.output.stderr).trim()
        )
    }
}

impl Error for NodeEvaluationFailure {}

pub(super) fn from_error(error: &(dyn Error + 'static)) -> Option<ConfigLoadFailureIdentity> {
    error
        .downcast_ref::<io::Error>()?
        .get_ref()?
        .downcast_ref::<NodeEvaluationFailure>()?
        .identity
        .clone()
}

fn project_owned_artifacts(
    bytes: &[u8],
    artifacts: &[(std::string::String, std::string::String)],
) -> Vec<u8> {
    // URLs can contain the absolute path spelling. Match the complete longest
    // spelling first, preserving URL encoding and every other diagnostic byte.
    let mut ordered = artifacts.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|(from, _)| std::cmp::Reverse(from.len()));
    let mut projected = bytes.to_vec();
    for (from, to) in ordered {
        if from.is_empty() {
            return bytes.to_vec();
        }
        let previous = projected;
        projected = Vec::new();
        let mut remaining = previous.as_slice();
        while let Some(offset) = remaining
            .windows(from.len())
            .position(|candidate| candidate == from.as_bytes())
        {
            let Some((prefix, suffix)) = remaining.split_at_checked(offset) else {
                break;
            };
            let Some(suffix) = suffix.strip_prefix(from.as_bytes()) else {
                break;
            };
            projected.extend_from_slice(prefix);
            projected.extend_from_slice(to.as_bytes());
            remaining = suffix;
        }
        projected.extend_from_slice(remaining);
    }
    projected
}

#[cfg(test)]
mod tests {
    use super::{NodeEvaluationFailure, from_error, project_owned_artifacts};
    use std::{io, path::Path, process::Command};

    fn artifacts(nonce: &str) -> Vec<(String, String)> {
        vec![
            (
                crate::cstr!("/tmp/vize-config-module-{nonce}/config.mjs").into(),
                "/source root/vite.config.ts".into(),
            ),
            (
                crate::cstr!("file:///tmp/vize-config-module-{nonce}/config.mjs").into(),
                "file:///source%20root/vite.config.ts".into(),
            ),
            (
                crate::cstr!("../../vize-config-module-{nonce}/config.mjs").into(),
                "vite.config.ts".into(),
            ),
        ]
    }

    #[test]
    fn projects_longest_complete_owned_spellings_only_and_preserves_other_bytes() {
        let mut stderr = b"reason/code=17\nfile:///tmp/vize-config-module-one/config.mjs\n/tmp/vize-config-module-one/config.mjs\n../../vize-config-module-one/config.mjs\nunchanged vize-config-module-two/config.mjs /tmp/foreign/config.mjs\n".to_vec();
        stderr.extend_from_slice(&[0xff, 0x80]);
        let mut expected = b"reason/code=17\nfile:///source%20root/vite.config.ts\n/source root/vite.config.ts\nvite.config.ts\nunchanged vize-config-module-two/config.mjs /tmp/foreign/config.mjs\n".to_vec();
        expected.extend_from_slice(&[0xff, 0x80]);
        assert_eq!(
            project_owned_artifacts(&stderr, &artifacts("one")),
            expected
        );
    }

    #[test]
    fn typed_identity_preserves_raw_display_distinguishes_errors_status_and_source() {
        let make = |nonce: &str, reason: &str, status: u8, source: &str| {
            let mut output = Command::new("node")
                .arg("-e")
                .arg(crate::cstr!("process.exit({status});").as_str())
                .output()
                .unwrap();
            output.stderr = crate::cstr!("{reason}\n../../vize-config-module-{nonce}/config.mjs\n")
                .as_bytes()
                .to_vec();
            NodeEvaluationFailure::new(Path::new(source), output, &artifacts(nonce))
        };
        let first = make("one", "authored-reason", 17, "/source root/vite.config.ts");
        let second = make("two", "authored-reason", 17, "/source root/vite.config.ts");
        assert_eq!(first.identity, second.identity);
        assert_eq!(
            crate::cstr!("{first}"),
            "node failed to load config: authored-reason\n../../vize-config-module-one/config.mjs"
        );
        for different in [
            make(
                "two",
                "different-authored-reason",
                17,
                "/source root/vite.config.ts",
            ),
            make("two", "authored-reason", 19, "/source root/vite.config.ts"),
            make("two", "authored-reason", 17, "/other/vite.config.ts"),
        ] {
            assert_ne!(first.identity, different.identity);
        }
        let expected = first.identity.clone();
        let boxed = io::Error::new(io::ErrorKind::InvalidData, first);
        assert_eq!(from_error(&boxed), expected);
        assert!(from_error(&io::Error::other("authored-reason")).is_none());
    }

    #[test]
    fn absent_or_empty_owned_metadata_keeps_the_original_strict_identity_contract() {
        for artifacts in [Vec::new(), vec![(String::new(), "anything".into())]] {
            let output = Command::new("node")
                .arg("-e")
                .arg("process.stderr.write('exact authored error'); process.exit(17);")
                .output()
                .unwrap();
            let error =
                NodeEvaluationFailure::new(Path::new("/source/vite.config.ts"), output, &artifacts);
            assert!(error.identity.is_none());
            assert_eq!(
                crate::cstr!("{error}"),
                "node failed to load config: exact authored error"
            );
        }
    }
}
