//! Fault isolation: file updates run on worker threads under
//! `catch_unwind`, so a stage that panics degrades its own file and every
//! other file keeps answering — threads plus unwinding instead of per-file
//! processes (Lean's worker-per-file model, scaled to SFC sizes).

use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::Mutex;

use vize_s0::String;

use super::cancel::{CancelToken, Cancelled};
use super::{SnapshotStats, SnapshotTree, Stages};
use crate::artifact::StageConfig;

/// One file's update request.
#[derive(Debug)]
pub struct FileJob<'a> {
    /// The file's current snapshot tree, if any.
    pub previous: Option<&'a SnapshotTree>,
    /// The file's new text.
    pub text: &'a str,
    /// The file task's token.
    pub token: CancelToken,
}

/// What one isolated update produced.
#[derive(Debug)]
pub enum FileOutcome {
    /// The new tree and its accounting.
    Ready(SnapshotTree, SnapshotStats),
    /// The update was cancelled; the previous tree stays current.
    Cancelled,
    /// A stage panicked; the previous tree stays current and the file
    /// reports this message (TS-47's degraded file).
    Degraded(String),
}

/// Update one file, catching a panicking stage.
#[must_use]
pub fn update_isolated(job: FileJob<'_>, config: StageConfig, stages: &Stages) -> FileOutcome {
    let FileJob {
        previous,
        text,
        token,
    } = job;
    let run = AssertUnwindSafe(|| SnapshotTree::update(previous, text, config, stages, token));
    match catch_unwind(run) {
        Ok(Ok((tree, stats))) => FileOutcome::Ready(tree, stats),
        Ok(Err(Cancelled)) => FileOutcome::Cancelled,
        Err(payload) => {
            let mut degraded = String::from("internal error in a stage task: ");
            degraded.push_str(&panic_message(payload.as_ref()));
            FileOutcome::Degraded(degraded)
        }
    }
}

/// The message of a panic payload: `panic!` with a literal carries a
/// `&str`, with a formatted message a std `String`.
#[expect(
    clippy::disallowed_types,
    reason = "the payload's std `String` is what `panic!` produced; it is read, not kept"
)]
fn panic_message(payload: &(dyn std::any::Any + Send)) -> String {
    payload
        .downcast_ref::<&str>()
        .map(|message| String::from(*message))
        .or_else(|| {
            payload
                .downcast_ref::<std::string::String>()
                .map(|message| String::from(message.as_str()))
        })
        .unwrap_or_else(|| String::from("stage task panicked"))
}

/// Update every file on `workers` threads (at least one); outcomes come back
/// in job order.
#[must_use]
pub fn update_files_isolated(
    jobs: Vec<FileJob<'_>>,
    config: StageConfig,
    stages: &Stages,
    workers: usize,
) -> Vec<FileOutcome> {
    let count = jobs.len();
    let queue = Mutex::new(jobs.into_iter().enumerate());
    let results: Mutex<Vec<Option<FileOutcome>>> = Mutex::new((0..count).map(|_| None).collect());
    std::thread::scope(|scope| {
        for _ in 0..workers.max(1) {
            scope.spawn(|| {
                loop {
                    let next = queue
                        .lock()
                        .unwrap_or_else(|poison| poison.into_inner())
                        .next();
                    let Some((index, job)) = next else {
                        break;
                    };
                    let outcome = update_isolated(job, config, stages);
                    if let Some(slot) = results
                        .lock()
                        .unwrap_or_else(|poison| poison.into_inner())
                        .get_mut(index)
                    {
                        *slot = Some(outcome);
                    }
                }
            });
        }
    });
    results
        .into_inner()
        .unwrap_or_else(|poison| poison.into_inner())
        .into_iter()
        .map(|outcome| {
            outcome
                .unwrap_or_else(|| FileOutcome::Degraded(String::from("file update did not run")))
        })
        .collect()
}
