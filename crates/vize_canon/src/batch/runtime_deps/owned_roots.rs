//! Observations made by the runtime dependency writer, never a package-name grant.

use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

/// One successful operation of the existing runtime link producer. This does
/// not certify declaration membership, Vue authority or input completeness.
#[derive(Debug, PartialEq, Eq)]
pub(in crate::batch) struct RuntimeDependencyLink {
    pub(in crate::batch) materialized: PathBuf,
    pub(in crate::batch) source: PathBuf,
}

/// Default materialization performs no extra observations. A terminal owner
/// can request a finite roster from the same writer, then scan/seal the roots.
#[derive(Default)]
pub(in crate::batch) struct RuntimeDependencyRoots {
    observed: bool,
    refused: bool,
    links: BTreeMap<PathBuf, PathBuf>,
}

impl RuntimeDependencyRoots {
    pub(in crate::batch) fn observed() -> Self {
        Self {
            observed: true,
            ..Self::default()
        }
    }

    pub(super) fn record(&mut self, source: &Path, target: &Path) {
        if !self.observed || self.refused {
            return;
        }
        let Ok(source) = fs::canonicalize(source) else {
            self.refused = true;
            return;
        };
        if !source.is_absolute()
            || !target.is_absolute()
            || fs::canonicalize(target).ok().as_ref() != Some(&source)
        {
            self.refused = true;
            return;
        }
        self.links.insert(target.to_path_buf(), source);
    }

    /// Retargeting or a failed observation disqualifies the optional plan.
    /// It does not change the old materializer's success or repair any leaf.
    pub(in crate::batch) fn finish(self) -> Option<Vec<RuntimeDependencyLink>> {
        if !self.observed || self.refused {
            return None;
        }
        self.links
            .into_iter()
            .map(|(materialized, source)| {
                (fs::canonicalize(&materialized).ok().as_ref() == Some(&source)).then_some(
                    RuntimeDependencyLink {
                        materialized,
                        source,
                    },
                )
            })
            .collect()
    }
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;

    #[test]
    fn writer_roster_is_whole_ordered_and_excludes_unobserved_links() {
        let owner = tempfile::tempdir().unwrap();
        let root = owner.path().canonicalize().unwrap();
        let first = root.join("first");
        let second = root.join("second");
        fs::create_dir(&first).unwrap();
        fs::create_dir(&second).unwrap();
        let z = root.join("z");
        let a = root.join("a");
        let unrelated = root.join("unrelated");
        let mut observed = RuntimeDependencyRoots::observed();
        super::super::recorded_symlink_path(&first, &z, &mut observed).unwrap();
        super::super::recorded_symlink_path(&second, &a, &mut observed).unwrap();
        symlink(&first, &unrelated).unwrap();
        assert_eq!(
            observed.finish(),
            Some(vec![
                RuntimeDependencyLink {
                    materialized: a,
                    source: second
                },
                RuntimeDependencyLink {
                    materialized: z,
                    source: first
                },
            ])
        );
        assert!(unrelated.is_symlink());
    }

    #[test]
    fn default_writer_and_observation_failure_do_not_change_link_decisions() {
        let owner = tempfile::tempdir().unwrap();
        let root = owner.path().canonicalize().unwrap();
        let absent = root.join("absent");
        let target = root.join("target");
        let mut plain = RuntimeDependencyRoots::default();
        super::super::recorded_symlink_path(&absent, &target, &mut plain).unwrap();
        assert!(plain.finish().is_none());
        assert_eq!(fs::read_link(&target).unwrap(), absent);
        let mut observed = RuntimeDependencyRoots::observed();
        observed.record(&absent, &target);
        assert!(observed.finish().is_none());
        assert_eq!(fs::read_link(&target).unwrap(), absent);
    }

    #[test]
    fn final_roster_refuses_retired_or_retargeted_links_without_repair() {
        let owner = tempfile::tempdir().unwrap();
        let root = owner.path().canonicalize().unwrap();
        let original = root.join("original");
        let other = root.join("other");
        fs::create_dir(&original).unwrap();
        fs::create_dir(&other).unwrap();
        let target = root.join("target");
        let mut observed = RuntimeDependencyRoots::observed();
        super::super::recorded_symlink_path(&original, &target, &mut observed).unwrap();
        fs::remove_file(&target).unwrap();
        symlink(&other, &target).unwrap();
        assert!(observed.finish().is_none());
        assert_eq!(fs::read_link(&target).unwrap(), other);
    }
}
