//! Deterministic work counts for imported-type resolution.
//!
//! Wall time on a shared machine is noise; the number of files read, modules
//! walked and locks taken is not. Tests read these counts to pin how the work
//! scales with components and with the modules behind a shared type. Outside
//! tests every call is an empty inlined function.

/// One unit of work a resolution performs.
#[derive(Clone, Copy, Debug)]
pub(in crate::script::context) enum Step {
    /// A file read from disk and digested.
    SourceRead,
    /// A file parsed into its type summary.
    SummaryParse,
    /// A specifier probed against the filesystem.
    Probe,
    /// A specifier resolution requested by a walk or a component.
    Resolution,
    /// A dependency edge examined by a walk.
    Edge,
    /// A module summary merged by a walk.
    Module,
    /// A declaration copied into a component's context.
    Declaration,
    /// A dependency path copied into a component's dependency list.
    Dependency,
    /// A process-wide or batch-wide lock acquired.
    SharedLock,
}

#[cfg(not(test))]
#[inline(always)]
pub(in crate::script::context) fn count(_step: Step, _times: usize) {}

#[cfg(test)]
pub(in crate::script::context) use counted::{Work, count, take};

#[cfg(test)]
mod counted {
    use super::Step;
    use std::cell::Cell;

    /// Totals of one thread since its last [`take`].
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub(in crate::script::context) struct Work {
        pub source_reads: usize,
        pub summary_parses: usize,
        pub probes: usize,
        pub resolutions: usize,
        pub edges: usize,
        pub modules: usize,
        pub declarations: usize,
        pub dependencies: usize,
        pub shared_locks: usize,
    }

    impl Work {
        pub(in crate::script::context) fn add(&mut self, other: Self) {
            self.source_reads += other.source_reads;
            self.summary_parses += other.summary_parses;
            self.probes += other.probes;
            self.resolutions += other.resolutions;
            self.edges += other.edges;
            self.modules += other.modules;
            self.declarations += other.declarations;
            self.dependencies += other.dependencies;
            self.shared_locks += other.shared_locks;
        }
    }

    std::thread_local! {
        static WORK: Cell<Work> = const { Cell::new(Work {
            source_reads: 0,
            summary_parses: 0,
            probes: 0,
            resolutions: 0,
            edges: 0,
            modules: 0,
            declarations: 0,
            dependencies: 0,
            shared_locks: 0,
        }) };
    }

    pub(in crate::script::context) fn count(step: Step, times: usize) {
        WORK.with(|cell| {
            let mut work = cell.get();
            let slot = match step {
                Step::SourceRead => &mut work.source_reads,
                Step::SummaryParse => &mut work.summary_parses,
                Step::Probe => &mut work.probes,
                Step::Resolution => &mut work.resolutions,
                Step::Edge => &mut work.edges,
                Step::Module => &mut work.modules,
                Step::Declaration => &mut work.declarations,
                Step::Dependency => &mut work.dependencies,
                Step::SharedLock => &mut work.shared_locks,
            };
            *slot += times;
            cell.set(work);
        });
    }

    /// This thread's totals, reset to zero.
    pub(in crate::script::context) fn take() -> Work {
        WORK.with(Cell::take)
    }
}
