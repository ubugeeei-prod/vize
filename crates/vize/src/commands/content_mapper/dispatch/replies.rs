//! The reply writer, and the admission credit it hands back to the reader.

use std::collections::BTreeMap;
use std::io::{self, Write};
use std::sync::mpsc::Receiver;
use std::sync::{Condvar, Mutex, PoisonError};

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

/// Write `(seq, frame)` replies until every sender is gone.
pub(super) fn write_replies<W: Write>(
    writer: &mut W,
    done: Receiver<(u64, Vec<u8>)>,
    progress: &WriteProgress,
    order: ReplyOrder,
) -> io::Result<()> {
    match order {
        ReplyOrder::Request => write_in_request_order(writer, done, progress),
        ReplyOrder::Ready => write_when_ready(writer, done, progress),
    }
}

fn write_in_request_order<W: Write>(
    writer: &mut W,
    done: Receiver<(u64, Vec<u8>)>,
    progress: &WriteProgress,
) -> io::Result<()> {
    let mut next = 0_u64;
    let mut pending = BTreeMap::<u64, Vec<u8>>::new();
    while let Ok((seq, bytes)) = done.recv() {
        pending.insert(seq, bytes);
        while let Some(frame) = pending.remove(&next) {
            writer.write_all(&frame)?;
            writer.flush()?;
            next += 1;
            progress.record(next);
        }
    }
    Ok(())
}

fn write_when_ready<W: Write>(
    writer: &mut W,
    done: Receiver<(u64, Vec<u8>)>,
    progress: &WriteProgress,
) -> io::Result<()> {
    let mut written = 0_u64;
    while let Ok((_, frame)) = done.recv() {
        writer.write_all(&frame)?;
        writer.flush()?;
        written += 1;
        progress.record(written);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::io::{BufReader, Cursor};
    use std::sync::mpsc;

    use serde_json::Value;

    use super::{ReplyOrder, WriteProgress, write_replies};
    use crate::commands::content_mapper::read_frame;
    use crate::commands::content_mapper::test_support::{
        close_project_request, frames, initialize_request, open_project_request, transform_request,
    };

    fn written(order: ReplyOrder) -> (Vec<u8>, u64) {
        let (done, replies) = mpsc::channel();
        // The second request finished first.
        done.send((1, b"second".to_vec())).unwrap();
        done.send((0, b"first".to_vec())).unwrap();
        drop(done);
        let progress = WriteProgress::new();
        let mut out = Vec::new();
        write_replies(&mut out, replies, &progress, order).unwrap();
        let flushed = progress
            .state
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .flushed;
        (out, flushed)
    }

    #[test]
    fn request_order_holds_a_finished_reply_behind_an_earlier_one() {
        assert_eq!(written(ReplyOrder::Request), (b"firstsecond".to_vec(), 2));
    }

    #[test]
    fn ready_order_writes_a_finished_reply_at_once() {
        assert_eq!(written(ReplyOrder::Ready), (b"secondfirst".to_vec(), 2));
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
