//! Reply writing, and the admission credit it hands back to the reader.

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Condvar, Mutex, PoisonError};

use super::wire::Frame;

/// When a finished reply may go out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::commands::content_mapper) enum ReplyOrder {
    /// In request order, for clients that pair replies by position.
    #[default]
    Request,
    /// As soon as it is ready. TypeScript hosts pair replies by id
    /// (`ipc.AsyncConn`), so a slow transform delays only its own reply.
    Ready,
}

/// Frames written so far. The reader admits a request only while fewer than
/// the worker count are unanswered, and runs project requests only once
/// every earlier frame is out. Both are counts, so they hold in either order.
pub(super) struct WriteProgress {
    state: Mutex<WriteState>,
    ready: Condvar,
}

struct WriteState {
    flushed: u64,
    failed: bool,
}

impl WriteProgress {
    pub(super) fn new() -> Self {
        Self {
            state: Mutex::new(WriteState {
                flushed: 0,
                failed: false,
            }),
            ready: Condvar::new(),
        }
    }

    fn record(&self, flushed: u64) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.flushed = flushed;
        self.ready.notify_all();
    }

    pub(super) fn fail(&self) {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        state.failed = true;
        self.ready.notify_all();
    }

    pub(super) fn wait_until(&self, flushed: u64) -> io::Result<()> {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        while state.flushed < flushed && !state.failed {
            state = self
                .ready
                .wait(state)
                .unwrap_or_else(PoisonError::into_inner);
        }
        if state.failed && state.flushed < flushed {
            Err(io::Error::other("content-mapper writer stopped"))
        } else {
            Ok(())
        }
    }
}

/// Where a finished reply goes.
pub(super) enum Replies<'a, W> {
    /// Written by the thread that finished it ([`ReplyOrder::Ready`]): no
    /// hand-off to a writer thread, one wake-up fewer per reply.
    Direct(&'a DirectReplies<'a, W>),
    /// To the writer thread, which restores request order.
    Ordered(Sender<(u64, Frame)>),
}

impl<W> Clone for Replies<'_, W> {
    fn clone(&self) -> Self {
        match self {
            Self::Direct(direct) => Self::Direct(direct),
            Self::Ordered(sender) => Self::Ordered(sender.clone()),
        }
    }
}

impl<W: Write> Replies<'_, W> {
    pub(super) fn send(&self, seq: u64, frame: Frame) -> io::Result<()> {
        match self {
            Self::Direct(direct) => direct.write(&frame),
            Self::Ordered(sender) => sender
                .send((seq, frame))
                .map_err(|_| io::Error::other("content-mapper writer stopped")),
        }
    }
}

/// The output for [`ReplyOrder::Ready`], shared by every thread that answers.
pub(super) struct DirectReplies<'a, W> {
    output: Mutex<DirectOutput<'a, W>>,
    progress: &'a WriteProgress,
    error: &'a Mutex<Option<io::Error>>,
}

struct DirectOutput<'a, W> {
    writer: &'a mut W,
    written: u64,
    failed: bool,
}

impl<'a, W: Write> DirectReplies<'a, W> {
    pub(super) fn new(
        writer: &'a mut W,
        progress: &'a WriteProgress,
        error: &'a Mutex<Option<io::Error>>,
    ) -> Self {
        Self {
            output: Mutex::new(DirectOutput {
                writer,
                written: 0,
                failed: false,
            }),
            progress,
            error,
        }
    }

    /// Write one frame whole and count it, as the writer thread would. The
    /// first failure is kept for the caller and stops every later write.
    fn write(&self, frame: &Frame) -> io::Result<()> {
        let mut output = self.output.lock().unwrap_or_else(PoisonError::into_inner);
        let output = &mut *output;
        if output.failed {
            return Err(io::Error::other("content-mapper writer stopped"));
        }
        match output
            .writer
            .write_all(frame)
            .and_then(|()| output.writer.flush())
        {
            Ok(()) => {
                output.written += 1;
                self.progress.record(output.written);
                Ok(())
            }
            Err(error) => {
                output.failed = true;
                self.progress.fail();
                *self.error.lock().unwrap_or_else(PoisonError::into_inner) = Some(error);
                Err(io::Error::other("content-mapper writer stopped"))
            }
        }
    }
}

/// Write `(seq, frame)` replies in request order until every sender is gone.
pub(super) fn write_replies<W: Write>(
    writer: &mut W,
    done: Receiver<(u64, Frame)>,
    progress: &WriteProgress,
) -> io::Result<()> {
    let mut next = 0_u64;
    let mut pending = BTreeMap::<u64, Frame>::new();
    while let Ok((seq, frame)) = done.recv() {
        pending.insert(seq, frame);
        while let Some(frame) = pending.remove(&next) {
            writer.write_all(&frame)?;
            writer.flush()?;
            next += 1;
            progress.record(next);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Cursor};
    use std::sync::mpsc;

    use serde_json::Value;

    use super::{DirectReplies, Replies, ReplyOrder, WriteProgress, write_replies};
    use crate::commands::content_mapper::read_frame;
    use crate::commands::content_mapper::test_support::{
        close_project_request, frames, initialize_request, open_project_request, transform_request,
    };

    fn flushed(progress: &WriteProgress) -> u64 {
        progress
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .flushed
    }

    #[test]
    fn request_order_holds_a_finished_reply_behind_an_earlier_one() {
        let (done, replies) = mpsc::channel();
        // The second request finished first.
        done.send((1, b"second".as_slice().into())).unwrap();
        done.send((0, b"first".as_slice().into())).unwrap();
        drop(done);
        let progress = WriteProgress::new();
        let mut out = Vec::new();
        write_replies(&mut out, replies, &progress).unwrap();
        assert_eq!((out, flushed(&progress)), (b"firstsecond".to_vec(), 2));
    }

    #[test]
    fn ready_order_writes_a_finished_reply_at_once() {
        let progress = WriteProgress::new();
        let error = std::sync::Mutex::new(None);
        let mut out = Vec::new();
        {
            let direct = DirectReplies::new(&mut out, &progress, &error);
            let replies = Replies::Direct(&direct);
            replies.send(1, b"second".as_slice().into()).unwrap();
            replies.send(0, b"first".as_slice().into()).unwrap();
        }
        assert_eq!(flushed(&progress), 2);
        assert_eq!(out, b"secondfirst".to_vec());
    }

    struct Broken;

    impl std::io::Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("pipe closed"))
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn a_failed_direct_write_wakes_admission_and_stops_later_writes() {
        let progress = WriteProgress::new();
        let error = std::sync::Mutex::new(None);
        let mut broken = Broken;
        let direct = DirectReplies::new(&mut broken, &progress, &error);
        assert!(direct.write(&b"a".as_slice().into()).is_err());
        assert!(progress.wait_until(1).is_err());
        assert!(direct.write(&b"b".as_slice().into()).is_err());
        // The writer's own error is kept for `serve` to return.
        assert!(
            error
                .lock()
                .unwrap()
                .as_ref()
                .is_some_and(|error| error.kind() == std::io::ErrorKind::Other)
        );
    }

    fn serve(input: &[u8], order: ReplyOrder) -> Vec<Vec<u8>> {
        let mut output = Vec::new();
        super::super::serve(
            &mut BufReader::new(Cursor::new(input)),
            &mut output,
            false,
            order,
        )
        .unwrap();
        let mut reader = Cursor::new(output);
        let mut bodies = Vec::new();
        while let Some(body) = read_frame(&mut reader).unwrap() {
            bodies.push(body);
        }
        bodies
    }

    fn id(body: &[u8]) -> Value {
        serde_json::from_slice::<Value>(body).unwrap()["id"].clone()
    }

    /// Project requests still wait for every earlier reply, and each reply
    /// is the frame request order writes.
    #[test]
    fn ready_order_keeps_project_barriers_and_reply_bytes() {
        let input = frames(&[
            initialize_request(),
            open_project_request(2, Value::Null, serde_json::json!({})),
            transform_request(3, "<template><p>{{ 1 }}</p></template>"),
            transform_request(4, "<script setup lang=\"ts\">const a = 1</script>"),
            transform_request(5, "<template><div v-if=\"true\" /></template>"),
            close_project_request(6),
            transform_request(7, "<template>closed</template>"),
        ]);
        let ordered = serve(&input, ReplyOrder::Request);
        let ready = serve(&input, ReplyOrder::Ready);
        let ids = |bodies: &[Vec<u8>]| bodies.iter().map(|body| id(body)).collect::<Vec<_>>();
        let (ordered_ids, ready_ids) = (ids(&ordered), ids(&ready));
        assert_eq!(ordered_ids, (1..=7).map(Value::from).collect::<Vec<_>>());
        assert_eq!(ready_ids.get(..2), ordered_ids.get(..2));
        assert_eq!(ready_ids.get(5..), ordered_ids.get(5..));
        let mut sorted = ready;
        sorted.sort_by_key(|body| id(body).as_u64());
        assert_eq!(sorted, ordered);
    }
}
