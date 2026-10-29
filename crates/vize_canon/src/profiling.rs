//! Profiling-only source identity census for generated-module parse inputs.

use std::fs;
use std::io::{self, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use oxc_span::{FileExtension, SourceType};
use serde::Serialize;
use sha2::{Digest, Sha256};
use vize_carton::{FxHashMap, String as CompactString};

const OUTPUT_ENV: &str = "VIZE_VTS_SOURCE_CENSUS_PATH";
const DIGEST_DOMAIN: &[u8] = b"vize.vts-source-census.sha256.v1\0";

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
struct Identity {
    source_type: [u8; 4],
    byte_length: u64,
    sha256: [u8; 32],
}

#[derive(Default)]
struct Census {
    producers: FxHashMap<Identity, u64>,
    consumers: FxHashMap<Identity, u64>,
}

static OUTPUT_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
static CENSUS: OnceLock<Mutex<Census>> = OnceLock::new();

fn output_path() -> Option<&'static Path> {
    OUTPUT_PATH
        .get_or_init(|| std::env::var_os(OUTPUT_ENV).map(PathBuf::from))
        .as_deref()
}

fn lexical_absolute(path: &Path) -> io::Result<PathBuf> {
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut normalized = PathBuf::new();
    for component in absolute.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(
                    normalized.components().next_back(),
                    Some(Component::Normal(_))
                ) {
                    normalized.pop();
                }
            }
            _ => normalized.push(component.as_os_str()),
        }
    }
    Ok(normalized)
}

fn ensure_distinct_output_paths(profile: &Path, receipt: &Path) -> io::Result<()> {
    // Profiling custody does not claim aliases equal only after symlink resolution.
    if lexical_absolute(profile)? == lexical_absolute(receipt)? {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "profile JSON and VTS source census receipt paths must differ",
        ));
    }
    Ok(())
}

/// Reject a receipt path that would be overwritten by the profile export.
pub fn ensure_vts_source_census_output_distinct(profile: &Path) -> io::Result<()> {
    let Some(receipt) = output_path() else {
        return Ok(());
    };
    if !receipt.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "VTS source census path must be absolute",
        ));
    }
    ensure_distinct_output_paths(profile, receipt)
}

fn source_type_identity(source_type: SourceType) -> [u8; 4] {
    let language = if source_type.is_typescript_definition() {
        2
    } else if source_type.is_typescript() {
        1
    } else {
        0
    };
    let module = if source_type.is_commonjs() {
        3
    } else if source_type.is_unambiguous() {
        2
    } else if source_type.is_module() {
        1
    } else {
        0
    };
    let variant = u8::from(source_type.is_jsx());
    let extension = match source_type.extension() {
        None => 0,
        Some(FileExtension::Js) => 1,
        Some(FileExtension::Mjs) => 2,
        Some(FileExtension::Cjs) => 3,
        Some(FileExtension::Jsx) => 4,
        Some(FileExtension::Ts) => 5,
        Some(FileExtension::Mts) => 6,
        Some(FileExtension::Cts) => 7,
        Some(FileExtension::Tsx) => 8,
    };
    [language, module, variant, extension]
}

fn identity(source: &str, source_type: SourceType) -> Identity {
    let source_type = source_type_identity(source_type);
    let mut digest = Sha256::new();
    digest.update(DIGEST_DOMAIN);
    digest.update(source_type);
    digest.update((source.len() as u64).to_le_bytes());
    digest.update(source.as_bytes());
    Identity {
        source_type,
        byte_length: source.len() as u64,
        sha256: digest.finalize().into(),
    }
}

fn observe(source: &str, source_type: SourceType, producer: bool) {
    if !vize_l0::profiler::global_profiler().is_enabled() || output_path().is_none() {
        return;
    }
    let key = identity(source, source_type);
    let mut census = CENSUS
        .get_or_init(|| Mutex::new(Census::default()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let multiset = if producer {
        &mut census.producers
    } else {
        &mut census.consumers
    };
    let count = multiset.entry(key).or_default();
    *count = count.saturating_add(1);
}

/// Observe a generated rewrite input immediately before its parser call.
pub fn observe_rewrite_input(source: &str, source_type: SourceType) {
    observe(source, source_type, true);
}

/// Observe an uncached dependency input immediately before its parser call.
pub fn observe_dependency_input(source: &str, source_type: SourceType) {
    observe(source, source_type, false);
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct IdentityEntry {
    source_type: CompactString,
    byte_length: u64,
    sha256: CompactString,
    count: u64,
}

fn sha256_hex(digest: &[u8; 32]) -> CompactString {
    digest
        .iter()
        .map(|byte| vize_l0::cstr!("{byte:02x}"))
        .collect()
}

fn entries(multiset: &FxHashMap<Identity, u64>) -> Vec<IdentityEntry> {
    let mut entries = multiset
        .iter()
        .map(|(identity, count)| IdentityEntry {
            source_type: vize_l0::cstr!(
                "language={};module={};variant={};extension={}",
                identity.source_type[0],
                identity.source_type[1],
                identity.source_type[2],
                identity.source_type[3]
            ),
            byte_length: identity.byte_length,
            sha256: sha256_hex(&identity.sha256),
            count: *count,
        })
        .collect::<Vec<_>>();
    entries.sort_by(|left, right| {
        left.source_type
            .cmp(&right.source_type)
            .then(left.byte_length.cmp(&right.byte_length))
            .then(left.sha256.cmp(&right.sha256))
    });
    entries
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
struct Comparison {
    producer_calls: u64,
    consumer_calls: u64,
    matched_calls: u64,
    producer_only_calls: u64,
    consumer_only_calls: u64,
    producer_complete: bool,
    consumer_complete: bool,
    matches: bool,
}

fn comparison(census: &Census) -> Comparison {
    let producer_calls = census
        .producers
        .values()
        .copied()
        .fold(0_u64, u64::saturating_add);
    let consumer_calls = census
        .consumers
        .values()
        .copied()
        .fold(0_u64, u64::saturating_add);
    let mut matched_calls = 0_u64;
    let mut producer_only_calls = 0_u64;
    for (identity, producer_count) in &census.producers {
        let consumer_count = census.consumers.get(identity).copied().unwrap_or(0);
        matched_calls = matched_calls.saturating_add((*producer_count).min(consumer_count));
        producer_only_calls =
            producer_only_calls.saturating_add(producer_count.saturating_sub(consumer_count));
    }
    let consumer_only_calls = census
        .consumers
        .iter()
        .map(|(identity, consumer_count)| {
            consumer_count.saturating_sub(census.producers.get(identity).copied().unwrap_or(0))
        })
        .fold(0_u64, u64::saturating_add);
    Comparison {
        producer_calls,
        consumer_calls,
        matched_calls,
        producer_only_calls,
        consumer_only_calls,
        producer_complete: matched_calls.saturating_add(producer_only_calls) == producer_calls,
        consumer_complete: matched_calls.saturating_add(consumer_only_calls) == consumer_calls,
        matches: consumer_only_calls == 0,
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Receipt {
    schema: &'static str,
    version: u64,
    algorithm: &'static str,
    domain: &'static str,
    identity: &'static str,
    multiplicity: &'static str,
    producers: Vec<IdentityEntry>,
    consumers: Vec<IdentityEntry>,
    comparison: Comparison,
}

fn receipt(census: &Census) -> Receipt {
    Receipt {
        schema: "vize.vts-source-identity",
        version: 1,
        algorithm: "sha256",
        domain: "vize.vts-source-census.sha256.v1",
        identity: "canonical SourceType tuple, byte length and domain-separated source digest",
        multiplicity: "count-preserving order-independent multiset",
        producers: entries(&census.producers),
        consumers: entries(&census.consumers),
        comparison: comparison(census),
    }
}

/// Write the configured create-new sidecar after the profiler snapshots its metrics.
pub fn write_vts_source_census_receipt() -> io::Result<()> {
    let Some(path) = output_path() else {
        return Ok(());
    };
    if !path.is_absolute() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "VTS source census path must be absolute",
        ));
    }
    let census = CENSUS
        .get_or_init(|| Mutex::new(Census::default()))
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let packet = receipt(&census);
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)?;
    serde_json::to_writer_pretty(&mut file, &packet).map_err(io::Error::other)?;
    file.write_all(b"\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn add(multiset: &mut FxHashMap<Identity, u64>, source: &str, source_type: SourceType) {
        let count = multiset.entry(identity(source, source_type)).or_default();
        *count += 1;
    }

    fn assert_complete(result: Comparison) {
        assert!(result.consumer_complete);
        assert!(result.producer_complete);
        assert_eq!(
            result.matched_calls + result.consumer_only_calls,
            result.consumer_calls
        );
        assert_eq!(
            result.matched_calls + result.producer_only_calls,
            result.producer_calls
        );
        assert_eq!(result.matches, result.consumer_only_calls == 0);
    }

    #[test]
    fn exact_source_and_type_match() {
        let mut census = Census::default();
        add(
            &mut census.producers,
            "export const value = 1;",
            SourceType::ts(),
        );
        add(
            &mut census.consumers,
            "export const value = 1;",
            SourceType::ts(),
        );
        let result = comparison(&census);
        assert_complete(result);
        assert_eq!((result.matched_calls, result.consumer_only_calls), (1, 0));
        assert!(result.matches);
    }

    #[test]
    fn duplicate_identity_retains_multiplicity() {
        let mut census = Census::default();
        for _ in 0..2 {
            add(&mut census.producers, "same", SourceType::ts());
        }
        for _ in 0..3 {
            add(&mut census.consumers, "same", SourceType::ts());
        }
        let result = comparison(&census);
        assert_complete(result);
        assert_eq!(
            (
                result.matched_calls,
                result.producer_only_calls,
                result.consumer_only_calls
            ),
            (2, 0, 1)
        );
        assert!(!result.matches);
    }

    #[test]
    fn source_type_mismatch_is_observed_not_reclassified() {
        let mut census = Census::default();
        add(&mut census.producers, "same", SourceType::ts());
        add(&mut census.consumers, "same", SourceType::tsx());
        let result = comparison(&census);
        assert_complete(result);
        assert_eq!(
            (
                result.matched_calls,
                result.producer_only_calls,
                result.consumer_only_calls
            ),
            (0, 1, 1)
        );
        assert!(!result.matches);
    }

    #[test]
    fn byte_mismatch_is_observed_not_reclassified() {
        let mut census = Census::default();
        add(&mut census.producers, "first", SourceType::ts());
        add(&mut census.consumers, "second", SourceType::ts());
        let result = comparison(&census);
        assert_complete(result);
        assert_eq!(
            (
                result.matched_calls,
                result.producer_only_calls,
                result.consumer_only_calls
            ),
            (0, 1, 1)
        );
        assert!(!result.matches);
    }

    #[test]
    fn output_paths_reject_lexically_equal_aliases_but_admit_distinct_files() {
        let receipt = std::env::current_dir()
            .unwrap()
            .join("target/vts-census/receipt.json");
        let equal = Path::new("target/vts-census/./nested/../receipt.json");
        ensure_distinct_output_paths(equal, &receipt).unwrap_err();
        assert!(ensure_distinct_output_paths(Path::new("profile.json"), &receipt).is_ok());
        #[cfg(unix)]
        assert_eq!(
            lexical_absolute(Path::new("/../../receipt.json")).unwrap(),
            PathBuf::from("/receipt.json")
        );
    }
}
