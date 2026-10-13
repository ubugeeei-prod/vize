//! Lossless failure-log storage with finite decoded custody and no terminal rendering.
use super::{metadata, retire_archive as archive};
use flate2::{Compression, bufread::GzDecoder, write::GzEncoder};
use serde_json::Value;
use std::{
    collections::BTreeSet,
    io::{Cursor, Read, Write},
    path::Path,
    process::{Command, Stdio},
};

pub(super) fn gzip(raw: &[u8]) -> Result<Vec<u8>, String> {
    if raw.is_empty() || raw.len() > archive::MAX_BYTES {
        return Err("Complete decoded failure log is empty or exceeds 64 MiB".into());
    }
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder
        .write_all(raw)
        .map_err(|_| "Could not encode complete failure log")?;
    encoder
        .finish()
        .map_err(|_| "Could not finish complete failure log encoding".into())
}

pub(super) fn blob_identity(raw: &[u8], root: &Path) -> Result<String, String> {
    let mut child = Command::new("git")
        .args(["hash-object", "--stdin"])
        .current_dir(root)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|_| "Could not authenticate complete failure log")?;
    child
        .stdin
        .take()
        .ok_or("Missing failure log identity stdin")?
        .write_all(raw)
        .map_err(|_| "Could not write complete failure log identity")?;
    let result = child
        .wait_with_output()
        .map_err(|_| "Could not finish complete failure log identity")?;
    if !result.status.success() {
        return Err(
            "Complete failure log identity command failed; raw diagnostics withheld".into(),
        );
    }
    let oid = String::from_utf8(result.stdout)
        .map_err(|_| "Invalid failure log identity encoding")?
        .trim()
        .to_owned();
    super::super::pr_contract::sha(&oid)?;
    Ok(oid)
}

pub(super) fn decode(stored: &[u8], expected_bytes: usize) -> Result<Vec<u8>, String> {
    if stored.is_empty()
        || stored.len() > archive::MAX_LOG_BYTES
        || expected_bytes == 0
        || expected_bytes > archive::MAX_BYTES
    {
        return Err("Failure log stored/decoded size is outside its finite bounds".into());
    }
    let mut decoder = GzDecoder::new(Cursor::new(stored));
    let mut raw = Vec::new();
    decoder
        .by_ref()
        .take(expected_bytes as u64 + 1)
        .read_to_end(&mut raw)
        .map_err(|_| "Complete compressed failure log is corrupt")?;
    if raw.len() != expected_bytes || decoder.into_inner().position() != stored.len() as u64 {
        return Err("Complete compressed failure log length/member custody differs".into());
    }
    Ok(raw)
}

fn verify(stored_oid: &str, source: &Value, root: &Path) -> Result<usize, String> {
    let raw_oid = source["oid"]
        .as_str()
        .ok_or("Missing original failure log identity")?;
    super::super::pr_contract::sha(raw_oid)?;
    let raw_bytes = source["bytes"]
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or("Missing original failure log size")?;
    // The caller has already authenticated the stored blob type and bounded size.
    // This byte helper never performs UTF-8 conversion, trimming or terminal output.
    let stored = metadata::bytes(
        &["--no-replace-objects", "cat-file", "blob", stored_oid],
        root,
    )?;
    let raw = decode(&stored, raw_bytes)?;
    if blob_identity(&raw, root)? != raw_oid {
        return Err("Complete decoded failure log identity differs".into());
    }
    Ok(raw_bytes)
}

pub(super) fn verify_all(
    receipt: &Value,
    objects: &serde_json::Map<String, Value>,
    expected: &BTreeSet<String>,
    root: &Path,
) -> Result<(), String> {
    if archive::log_suffix(receipt) != ".log.gz" {
        return Ok(());
    }
    if receipt["originalEvidence"]["failureLogEncoding"] != "gzip" {
        return Err("Missing exact compressed failure log encoding".into());
    }
    let sources = receipt["originalEvidence"]["failureLogSources"]
        .as_object()
        .ok_or("Missing complete original failure log identities")?;
    if sources.keys().cloned().collect::<BTreeSet<_>>() != *expected {
        return Err("Original and stored failure log identities differ".into());
    }
    let metadata_bytes = serde_json::to_vec_pretty(receipt)
        .map_err(|_| "Invalid complete retirement metadata")?
        .len();
    let mut decoded_total = metadata_bytes;
    for (path, object) in objects {
        let source = &sources[path];
        let bytes = source["bytes"]
            .as_u64()
            .and_then(|value| usize::try_from(value).ok())
            .ok_or("Missing original failure log size")?;
        decoded_total = decoded_total
            .checked_add(bytes)
            .filter(|total| *total <= archive::MAX_BYTES)
            .ok_or("Complete decoded retirement archive exceeds bounded 64 MiB")?;
        let oid = object["oid"]
            .as_str()
            .ok_or("Missing stored failure log identity")?;
        verify(oid, source, root)?;
    }
    Ok(())
}

#[cfg(test)]
#[path = "pr_pin_retire_log_storage_tests.rs"]
mod tests;
