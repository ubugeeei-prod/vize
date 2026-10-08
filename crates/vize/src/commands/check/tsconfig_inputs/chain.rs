//! Operation-scoped memoization for effective tsconfig inheritance.

use std::{
    io,
    path::{Path, PathBuf},
};

use vize_l0::{FxHashMap, FxHashSet};

/// A config shared by multiple extends entries must be merged into each entry.
/// Only the active ancestry closes a cycle; completed acyclic values are reused
/// rather than suppressed. Cycle-cut values depend on that ancestry and cannot
/// be reused by a sibling. Each loader keeps its existing default cycle result.
#[derive(Default)]
pub(super) struct ConfigChain<T> {
    active: FxHashSet<PathBuf>,
    completed: FxHashMap<PathBuf, T>,
    cycle_cut: bool,
}

impl<T: Clone + Default> ConfigChain<T> {
    pub(super) fn load(
        &mut self,
        path: &Path,
        read: impl FnOnce(&mut Self) -> io::Result<T>,
    ) -> io::Result<T> {
        if let Some(value) = self.completed.get(path) {
            return Ok(value.clone());
        }
        if !self.active.insert(path.to_path_buf()) {
            self.cycle_cut = true;
            return Ok(T::default());
        }
        let enclosing_cycle_cut = std::mem::replace(&mut self.cycle_cut, false);
        let result = read(self);
        self.active.remove(path);
        let cycle_cut = self.cycle_cut;
        self.cycle_cut = enclosing_cycle_cut || cycle_cut;
        if !cycle_cut && let Ok(value) = &result {
            self.completed.insert(path.to_path_buf(), value.clone());
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn diamond(
        chain: &mut ConfigChain<usize>,
        depth: usize,
        reads: &mut usize,
    ) -> io::Result<usize> {
        let path = PathBuf::from_iter(std::iter::repeat_n("base", depth + 1));
        chain.load(&path, |chain| {
            *reads += 1;
            if depth == 0 {
                return Ok(1);
            }
            let first = diamond(chain, depth - 1, reads)?;
            let second = diamond(chain, depth - 1, reads)?;
            assert_eq!(first, second, "a completed ancestor must replay its value");
            Ok(second + 1)
        })
    }

    #[test]
    fn deep_acyclic_diamonds_read_each_unique_config_once() {
        let mut chain = ConfigChain::default();
        let mut reads = 0;
        assert_eq!(diamond(&mut chain, 24, &mut reads).unwrap(), 25);
        assert_eq!(reads, 25);
    }

    fn cycle(chain: &mut ConfigChain<usize>, name: &str, reads: &mut usize) -> io::Result<usize> {
        chain.load(Path::new(name), |chain| {
            *reads += 1;
            Ok(cycle(chain, if name == "a" { "b" } else { "a" }, reads)? + 1)
        })
    }

    #[test]
    fn cycle_cut_results_do_not_leak_into_a_different_ancestry() {
        let mut chain = ConfigChain::default();
        let mut reads = 0;
        assert_eq!(cycle(&mut chain, "a", &mut reads).unwrap(), 2);
        assert_eq!(cycle(&mut chain, "b", &mut reads).unwrap(), 2);
        assert_eq!(reads, 4);
    }

    #[test]
    fn failed_reads_release_active_ancestry_and_are_not_cached() {
        let mut chain = ConfigChain::<usize>::default();
        let path = Path::new("unreadable");
        let failure = chain
            .load(path, |_| {
                Err(io::Error::from(io::ErrorKind::PermissionDenied))
            })
            .unwrap_err();
        assert_eq!(failure.kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(chain.load(path, |_| Ok(7)).unwrap(), 7);
    }
}
