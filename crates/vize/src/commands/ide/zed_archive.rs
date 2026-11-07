use flate2::read::MultiGzDecoder;
use std::collections::BTreeSet;
use std::io::Cursor;
use std::path::Path;
use vize_l0::{String, cstr};

pub(super) const MAX_ZED_ARCHIVE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_ZED_EXTRACTED_BYTES: u64 = 8 * 1024 * 1024;
const MAX_ZED_TAR_STREAM_BYTES: u64 = 10 * 1024 * 1024;
const MAX_ZED_ARCHIVE_ENTRIES: usize = 256;

pub(super) fn unpack_zed_extension_archive(bytes: &[u8], destination: &Path) -> Result<(), String> {
    validate_zed_extension_archive(bytes)?;

    let decoder = MultiGzDecoder::new(Cursor::new(bytes));
    let mut archive = tar::Archive::new(decoder);
    let entries = archive
        .entries()
        .map_err(|error| cstr!("could not read the Zed extension archive: {error}"))?;
    for entry in entries {
        let mut entry = entry
            .map_err(|error| cstr!("could not read a Zed extension archive entry: {error}"))?;
        if !entry
            .unpack_in(destination)
            .map_err(|error| cstr!("could not extract the Zed extension archive: {error}"))?
        {
            return Err(cstr!(
                "refused a Zed extension archive entry outside the destination"
            ));
        }
    }
    Ok(())
}

fn validate_zed_extension_archive(bytes: &[u8]) -> Result<(), String> {
    if bytes.len() as u64 > MAX_ZED_ARCHIVE_BYTES {
        return Err(cstr!(
            "Zed extension archive exceeds the {MAX_ZED_ARCHIVE_BYTES}-byte limit"
        ));
    }

    let decoder = MultiGzDecoder::new(Cursor::new(bytes));
    let limited = std::io::Read::take(decoder, MAX_ZED_TAR_STREAM_BYTES + 1);
    let mut archive = tar::Archive::new(limited);
    let mut seen = BTreeSet::new();
    let mut extracted_bytes = 0_u64;
    let mut has_manifest = false;
    {
        let entries = archive
            .entries()
            .map_err(|error| cstr!("could not read the Zed extension archive: {error}"))?;
        for (index, entry) in entries.enumerate() {
            if index >= MAX_ZED_ARCHIVE_ENTRIES {
                return Err(cstr!(
                    "Zed extension archive exceeds the {MAX_ZED_ARCHIVE_ENTRIES}-entry limit"
                ));
            }
            let entry = entry
                .map_err(|error| cstr!("could not read a Zed extension archive entry: {error}"))?;
            let entry_type = entry.header().entry_type();
            let size = entry
                .header()
                .size()
                .map_err(|error| cstr!("invalid Zed extension archive entry size: {error}"))?;
            let name = validate_zed_archive_member(
                entry.path_bytes().as_ref(),
                entry_type,
                size,
                &mut seen,
            )?;
            extracted_bytes = extracted_bytes
                .checked_add(size)
                .filter(|total| *total <= MAX_ZED_EXTRACTED_BYTES)
                .ok_or_else(|| {
                    cstr!(
                        "Zed extension archive exceeds the {MAX_ZED_EXTRACTED_BYTES}-byte extraction limit"
                    )
                })?;
            has_manifest |= name == "zed/extension.toml" && entry_type.is_file();
        }
    }
    let mut limited = archive.into_inner();
    std::io::copy(&mut limited, &mut std::io::sink())
        .map_err(|error| cstr!("invalid Zed extension gzip stream: {error}"))?;
    let stream_bytes = MAX_ZED_TAR_STREAM_BYTES + 1 - limited.limit();
    if stream_bytes > MAX_ZED_TAR_STREAM_BYTES {
        return Err(cstr!(
            "Zed extension archive exceeds the {MAX_ZED_TAR_STREAM_BYTES}-byte stream limit"
        ));
    }
    if !has_manifest {
        return Err(cstr!(
            "release archive does not contain a regular zed/extension.toml"
        ));
    }
    Ok(())
}

pub(super) fn validate_zed_archive_member(
    raw_name: &[u8],
    entry_type: tar::EntryType,
    size: u64,
    seen: &mut BTreeSet<String>,
) -> Result<String, String> {
    if !entry_type.is_file() && !entry_type.is_dir() {
        return Err(cstr!(
            "Zed extension archive contains a link or special entry"
        ));
    }
    let name = std::str::from_utf8(raw_name)
        .map_err(|_| cstr!("Zed extension archive contains a non-UTF-8 path"))?;
    if name.starts_with('/') || name.contains('\\') || name.contains('\0') {
        return Err(cstr!("Zed extension archive contains an unsafe path"));
    }
    let normalized = name.strip_suffix('/').unwrap_or(name);
    let mut parts = normalized.split('/');
    if parts.next() != Some("zed")
        || parts
            .clone()
            .any(|part| part.is_empty() || part == "." || part == "..")
        || normalized == "zed" && !entry_type.is_dir()
        || name.ends_with('/') && !entry_type.is_dir()
        || entry_type.is_dir() && size != 0
    {
        return Err(cstr!(
            "Zed extension archive contains an unsafe path or entry"
        ));
    }
    if !seen.insert(normalized.into()) {
        return Err(cstr!(
            "Zed extension archive contains a duplicate path: {normalized}"
        ));
    }
    Ok(normalized.into())
}
