//! Process-level shards of one canonical corpus sweep.
//!
//! The production-reach and SSR sweeps clear and read the process-wide
//! profiler around every compile, so threads inside one test process would
//! mix counters. CI instead builds the test binary once and runs several
//! copies of it, each with `VIZE_DAVINCI_DIFFERENTIAL_CORPUS_SHARD=<index>/<count>`.
//! Shard `index` keeps every file whose position in the complete collected
//! vector is congruent to `index` modulo `count`, writes those logical paths
//! (NUL-terminated, in sweep order) to the file named by
//! `VIZE_DAVINCI_DIFFERENTIAL_CORPUS_SHARD_VECTOR`, and reports one line. The
//! observer recorder sums the shard counters and proves the vectors are an
//! exact, disjoint cover of the canonical file manifest. Without the shard
//! variable the sweep is the complete corpus, exactly as before.

use std::fs;
use std::path::{Path, PathBuf};

use vize_l0::{CompactString, cstr};

use super::{CorpusSweep, resolve_env_sweep};

/// The environment variable naming this process's shard as `index/count`.
pub const SHARD_ENV: &str = "VIZE_DAVINCI_DIFFERENTIAL_CORPUS_SHARD";

/// The environment variable naming the file that receives the shard's
/// NUL-terminated logical path vector.
pub const SHARD_VECTOR_ENV: &str = "VIZE_DAVINCI_DIFFERENTIAL_CORPUS_SHARD_VECTOR";

/// One process-level slice of a collected corpus vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CorpusShard {
    /// Zero-based shard position, below `count`.
    pub index: usize,
    /// The number of shards that together cover the corpus.
    pub count: usize,
}

fn decimal(value: &str) -> Option<usize> {
    let canonical = !value.is_empty()
        && value.bytes().all(|byte| byte.is_ascii_digit())
        && (value == "0" || !value.starts_with('0'));
    canonical.then(|| value.parse().ok()).flatten()
}

impl CorpusShard {
    /// Parse `index/count`, refusing anything but two canonical decimals with
    /// `index < count`.
    pub fn parse(spec: &str) -> Result<Self, CompactString> {
        let invalid = || cstr!("{SHARD_ENV} must be `index/count` with index < count: `{spec}`");
        let (index, count) = spec.split_once('/').ok_or_else(invalid)?;
        let (Some(index), Some(count)) = (decimal(index), decimal(count)) else {
            return Err(invalid());
        };
        if index >= count {
            return Err(invalid());
        }
        Ok(Self { index, count })
    }

    /// Keep the files whose position in the complete vector belongs to this
    /// shard, in their original order.
    #[must_use]
    pub fn select<T>(self, files: Vec<T>) -> Vec<T> {
        files
            .into_iter()
            .enumerate()
            .filter(|(position, _)| position % self.count == self.index)
            .map(|(_, file)| file)
            .collect()
    }
}

/// The NUL-terminated logical paths of `files` relative to `root`.
///
/// # Panics
///
/// Fails the calling test when a selected file is outside `root` or its
/// logical path is not UTF-8; such a path could not be matched against the
/// canonical manifest.
#[must_use]
pub fn logical_vector(root: &Path, files: &[PathBuf]) -> Vec<u8> {
    let mut vector = Vec::new();
    for file in files {
        let logical = file
            .strip_prefix(root)
            .unwrap_or_else(|_| panic!("shard file outside the corpus root: {}", file.display()));
        let logical = logical.to_str().unwrap_or_else(|| {
            panic!(
                "shard file has a non-UTF-8 logical path: {}",
                file.display()
            )
        });
        vector.extend_from_slice(logical.as_bytes());
        vector.push(0);
    }
    vector
}

/// [`resolve_env_sweep`], restricted to the shard named by [`SHARD_ENV`].
///
/// Returns the complete sweep when [`SHARD_ENV`] is unset. A shard requires
/// [`SHARD_VECTOR_ENV`]; its vector is written before any file is swept.
///
/// # Panics
///
/// Fails the calling test on an invalid shard specification, a missing vector
/// path, or a vector that cannot be written.
pub fn resolve_env_shard() -> Option<CorpusSweep> {
    let sweep = resolve_env_sweep()?;
    let Some(spec) = std::env::var_os(SHARD_ENV) else {
        return Some(sweep);
    };
    let spec = spec
        .to_str()
        .unwrap_or_else(|| panic!("{SHARD_ENV} must be UTF-8"));
    let shard = CorpusShard::parse(spec).unwrap_or_else(|error| panic!("{error}"));
    let vector_path = std::env::var_os(SHARD_VECTOR_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| panic!("{SHARD_ENV} requires {SHARD_VECTOR_ENV}"));
    let CorpusSweep { root, scope, files } = sweep;
    let planned = files.len();
    let files = shard.select(files);
    fs::write(&vector_path, logical_vector(&root, &files)).unwrap_or_else(|error| {
        panic!(
            "cannot write shard vector {}: {error}",
            vector_path.display()
        )
    });
    let sweep = CorpusSweep { root, scope, files };
    eprintln!(
        "davinci-differential corpus shard: scope={} index={} count={} planned={planned} selected={}",
        sweep.scope_label(),
        shard.index,
        shard.count,
        sweep.files.len(),
    );
    Some(sweep)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{CorpusShard, logical_vector};

    #[test]
    fn shard_specifications_are_canonical_and_bounded() {
        assert_eq!(
            CorpusShard::parse("0/1"),
            Ok(CorpusShard { index: 0, count: 1 })
        );
        assert_eq!(
            CorpusShard::parse("15/16"),
            Ok(CorpusShard {
                index: 15,
                count: 16
            })
        );
        for spec in [
            "", "1", "/1", "0/", "1/1", "2/1", "0/0", "-1/2", "+1/2", "01/2", "0/02", " 0/2",
            "0/2 ", "0/2/3", "a/2",
        ] {
            assert!(CorpusShard::parse(spec).is_err(), "{spec:?}");
        }
    }

    #[test]
    fn shards_partition_the_whole_vector_in_order() {
        let files: Vec<u32> = (0..23).collect();
        for count in 1..=7 {
            let mut covered = Vec::new();
            for index in 0..count {
                let shard = CorpusShard { index, count };
                let selected = shard.select(files.clone());
                assert!(selected.windows(2).all(|pair| pair[0] < pair[1]));
                assert!(selected.iter().all(|file| *file as usize % count == index));
                covered.extend(selected);
            }
            covered.sort_unstable();
            assert_eq!(covered, files, "count={count}");
        }
        let empty = CorpusShard { index: 4, count: 5 }.select(vec![1, 2, 3]);
        assert!(empty.is_empty());
    }

    #[test]
    fn shard_vectors_hold_nul_terminated_logical_paths() {
        let root = Path::new("/corpus/root");
        let files = [
            PathBuf::from("/corpus/root/alpha/App.vue"),
            PathBuf::from("/corpus/root/beta/日本語 Page.vue"),
        ];
        assert_eq!(
            logical_vector(root, &files),
            "alpha/App.vue\0beta/日本語 Page.vue\0".as_bytes()
        );
        assert!(logical_vector(root, &[]).is_empty());
    }
}
