//! Source facts shared by every reachability scan of one preparation pass.
//!
//! Importers of different packages scan overlapping closures, and each scan
//! used to stat, read and parse the same package files again (#9727). A pass
//! now loads each file once, reading a scan's whole frontier in parallel,
//! and resolves each of its path and package edges once. Equal routes reached
//! from different importer scopes share one finished scan, and so do callers
//! whose aliases answer that scan's bare spellings the same way.
//! Facts never outlive the pass that treats the filesystem as stable.

#![expect(
    clippy::disallowed_types,
    reason = "parsed specifiers are shared by every scan that visits a file"
)]

use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};

use oxc_span::SourceType;
use rayon::prelude::*;
use vize_carton::{FxHashMap, FxHashSet, String as CompactString};

use super::{PackageRouteReachability, ReachabilityBudget};

pub(super) type Occurrences = Arc<[(CompactString, crate::PackageResolutionMode)]>;

/// A readable source's length and unique module specifiers, in first-seen order.
#[derive(Clone)]
pub(super) struct SourceText {
    pub(super) len: usize,
    pub(super) occurrences: Occurrences,
    /// Resolutions of `occurrences`, by index, filled by the first scan that
    /// needs each one.
    pub(super) edges: Arc<[ResolvedEdge]>,
}

/// Consulted inputs, interned per pass: importers in one directory consult
/// the same paths, so a scan records each distinct list once.
pub(super) type InputList = Arc<[PathBuf]>;

/// Resolution outcome and consulted inputs.
pub(super) type Resolution<T> = (Option<T>, InputList);

/// Lookups that depend only on the importer and the specifier, so every scan
/// of a pass shares them. Alias targets are left out: the canon walk resolves
/// them against the scanned route's package root.
#[derive(Default)]
pub(super) struct ResolvedEdge {
    /// Relative or absolute spellings.
    pub(super) path: OnceLock<Resolution<PathBuf>>,
    /// Bare spellings that no alias resolved.
    pub(super) package: OnceLock<Resolution<crate::PackageRoute>>,
}

#[derive(Clone)]
enum Facts {
    /// `metadata` failed: the scan skips the file.
    Missing,
    /// `text` stays unread while the file exceeded the loading scan's
    /// per-file budget; `Some(None)` records a failed read.
    Sized {
        len: u64,
        text: Option<Option<SourceText>>,
    },
}

/// A bare spelling a scan resolved through the caller's aliases, and the
/// answer it got. Path spellings and package edges never depend on aliases.
pub(super) struct AliasProbe {
    importer: PathBuf,
    specifier: CompactString,
    answer: (Option<PathBuf>, Vec<PathBuf>),
}

impl AliasProbe {
    pub(super) fn new(
        importer: &Path,
        specifier: &str,
        answer: &(Option<PathBuf>, Vec<PathBuf>),
    ) -> Self {
        Self {
            importer: importer.to_path_buf(),
            specifier: specifier.into(),
            answer: answer.clone(),
        }
    }
}

struct FinishedScan {
    budget: ReachabilityBudget,
    route: crate::PackageRoute,
    reachability: PackageRouteReachability,
    /// Alias scopes known to answer every probe as the scan saw it.
    scopes: Vec<u64>,
    probes: Box<[AliasProbe]>,
}

#[derive(Default)]
pub struct ReachabilitySources {
    files: FxHashMap<PathBuf, Facts>,
    /// Finished scans by manifest. A scan reads only its route, the budget,
    /// the facts above and its alias answers, so an equal route repeats the
    /// same outcome wherever those answers agree.
    scans: FxHashMap<PathBuf, Vec<FinishedScan>>,
    /// The alias configuration of the scans that follow; see
    /// [`Self::set_alias_scope`].
    alias_scope: u64,
    finished_scans: usize,
    input_lists: FxHashSet<InputList>,
    /// Scans keep no inputs; see [`Self::untracked`].
    untracked: bool,
}

impl ReachabilitySources {
    /// Sources whose scans report no invalidation inputs, for a one-shot
    /// check that never refreshes: collecting and sorting every scan's inputs
    /// was a quarter of its import walk (#9727). Outcomes and work are the
    /// same as a tracked scan's.
    pub fn untracked() -> Self {
        Self {
            untracked: true,
            ..Self::default()
        }
    }

    pub(super) fn is_untracked(&self) -> bool {
        self.untracked
    }

    /// Identify the aliases behind the caller's local resolver. Sources may
    /// outlive one alias configuration, e.g. across a check's referenced
    /// programs: a scan finished under another scope is reused only when each
    /// bare spelling it resolved through aliases resolves the same way now.
    /// Callers keep one set of sources per import options and package
    /// resolution settings, since path and package edges depend on those.
    pub fn set_alias_scope(&mut self, scope: u64) {
        self.alias_scope = scope;
    }

    /// Scans these sources ran, not counting reused ones.
    pub fn finished_scans(&self) -> usize {
        self.finished_scans
    }

    pub(super) fn intern(&mut self, mut inputs: Vec<PathBuf>) -> InputList {
        if self.untracked {
            inputs.clear();
        }
        if let Some(list) = self.input_lists.get(inputs.as_slice()) {
            return list.clone();
        }
        let list = InputList::from(inputs);
        self.input_lists.insert(list.clone());
        list
    }

    pub(super) fn scanned(
        &mut self,
        route: &crate::PackageRoute,
        budget: ReachabilityBudget,
        resolve_local: &mut impl FnMut(&Path, &str) -> (Option<PathBuf>, Vec<PathBuf>),
    ) -> Option<PackageRouteReachability> {
        let scope = self.alias_scope;
        let scans = self.scans.get_mut(&route.manifest_path)?;
        let mut unverified = Vec::new();
        for scan in scans
            .iter_mut()
            .filter(|scan| scan.budget == budget && scan.route == *route)
        {
            if scan.scopes.contains(&scope) {
                return Some(scan.reachability.clone());
            }
            unverified.push(scan);
        }
        let scan = unverified.into_iter().find(|scan| {
            scan.probes.iter().all(|probe| {
                resolve_local(&probe.importer, probe.specifier.as_str()) == probe.answer
            })
        })?;
        scan.scopes.push(scope);
        Some(scan.reachability.clone())
    }

    pub(super) fn remember_scan(
        &mut self,
        route: &crate::PackageRoute,
        budget: ReachabilityBudget,
        probes: Vec<AliasProbe>,
        reachability: PackageRouteReachability,
    ) {
        self.finished_scans += 1;
        self.scans
            .entry(route.manifest_path.clone())
            .or_default()
            .push(FinishedScan {
                budget,
                route: route.clone(),
                reachability,
                scopes: vec![self.alias_scope],
                probes: probes.into_boxed_slice(),
            });
    }

    /// File size from metadata, or `None` when the file cannot be stat'ed.
    pub(super) fn len(&mut self, path: &Path) -> Option<u64> {
        if !self.files.contains_key(path) {
            self.files.insert(path.to_path_buf(), stat(path));
        }
        match self.files.get(path)? {
            Facts::Missing => None,
            Facts::Sized { len, .. } => Some(*len),
        }
    }

    /// The parsed source, read on first use; `None` when it cannot be read.
    pub(super) fn text(&mut self, path: &Path) -> Option<SourceText> {
        let facts = self
            .files
            .entry(path.to_path_buf())
            .or_insert_with(|| stat(path));
        match facts {
            Facts::Missing => None,
            Facts::Sized {
                text: Some(text), ..
            } => text.clone(),
            Facts::Sized { text, .. } => text.insert(read(path)).clone(),
        }
    }

    pub(super) fn is_loaded(&self, path: &Path) -> bool {
        matches!(
            self.files.get(path),
            Some(Facts::Missing | Facts::Sized { text: Some(_), .. })
        )
    }

    /// Load every path not loaded yet in parallel. Sources over
    /// `max_file_bytes` are only stat'ed: a scan rejects them before reading.
    pub(super) fn prefetch<'a>(
        &mut self,
        paths: impl IntoIterator<Item = &'a PathBuf>,
        max_file_bytes: usize,
    ) {
        let pending = paths
            .into_iter()
            .filter(|path| !self.is_loaded(path))
            .collect::<Vec<_>>();
        let loaded = pending
            .into_par_iter()
            .map(|path| (path.clone(), load(path, max_file_bytes)))
            .collect::<Vec<_>>();
        self.files.extend(loaded);
    }
}

fn stat(path: &Path) -> Facts {
    match std::fs::metadata(path) {
        Ok(metadata) => Facts::Sized {
            len: metadata.len(),
            text: None,
        },
        Err(_) => Facts::Missing,
    }
}

fn load(path: &Path, max_file_bytes: usize) -> Facts {
    match stat(path) {
        Facts::Sized { len, .. } if u64::try_from(max_file_bytes).is_ok_and(|max| len <= max) => {
            Facts::Sized {
                len,
                text: Some(read(path)),
            }
        }
        facts => facts,
    }
}

fn read(path: &Path) -> Option<SourceText> {
    let content = std::fs::read_to_string(path).ok()?;
    let source_type = if path.extension().is_some_and(|extension| extension == "tsx") {
        SourceType::tsx()
    } else {
        SourceType::ts()
    };
    let mut seen = FxHashSet::default();
    let occurrences: Occurrences = crate::batch::ImportRewriter::new()
        .collect_all_specifier_occurrences(&content, source_type)
        .into_iter()
        .filter(|occurrence| seen.insert(occurrence.clone()))
        .collect();
    let edges = std::iter::repeat_with(ResolvedEdge::default)
        .take(occurrences.len())
        .collect();
    Some(SourceText {
        len: content.len(),
        occurrences,
        edges,
    })
}
