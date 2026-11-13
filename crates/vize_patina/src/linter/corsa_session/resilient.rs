//! Batched checker requests that answer every item the runtime can.
//!
//! The native runtime answers a batch (`getTypeAtLocations`, `getTypesOfSymbols`,
//! ...) in one response, so a panic while describing one type fails the whole
//! request. TypeScript 7.0 and 7.1 panic on the clone of the empty tuple that
//! an empty array literal gets in a tuple context (`[] as const`,
//! `Promise.all([])`, `...[]`): `newTypeResponse` reads it as a `TupleType`
//! (#12949). One such expression would leave every other expression of the
//! file untyped. A panicking batch is split and retried instead, so only the
//! items whose own answer panics stay unanswered.

use corsa::CorsaError;
use std::future::Future;

/// Panicking requests one batch may retry before the failure is taken to be
/// systematic rather than a property of a few types. Each panicking item costs
/// about `2 * log2(batch)` requests.
const PANIC_BUDGET: usize = 64;

/// Whether `error` is a panic the runtime recovered while answering one
/// request. It arrives as an RPC error over JSON-RPC and as a protocol error
/// over msgpack, with the message `panic: <value>` and the stack.
pub(super) fn is_recovered_panic(error: &CorsaError) -> bool {
    match error {
        CorsaError::Rpc(response) => response.message.starts_with("panic:"),
        CorsaError::Protocol(message) => message.starts_with("panic:"),
        _ => false,
    }
}

/// Whether `error` concerns one request rather than the session: the runtime
/// answered with an error, or its answer did not decode. A query that fails
/// this way loses only its own facts. A closed, timed-out or broken transport
/// fails every later request too, so it is reported once for the file.
pub(super) fn is_query_local(error: &CorsaError) -> bool {
    !matches!(
        error,
        CorsaError::Io(_) | CorsaError::Closed(_) | CorsaError::Timeout(_) | CorsaError::Join(_)
    )
}

/// One query's answer: `Some` when it succeeded, `None` when it failed on its
/// own ([`is_query_local`]), and the error when the session failed.
pub(super) fn local<T>(result: corsa::Result<T>) -> corsa::Result<Option<T>> {
    match result {
        Ok(value) => Ok(Some(value)),
        Err(error) if is_query_local(&error) => Ok(None),
        Err(error) => Err(error),
    }
}

/// Answer `items` through `ask`, one batch when it succeeds. A batch whose
/// answer panics is halved until the panicking items stand alone; those come
/// back `None`, everything else in input order.
pub(super) async fn answer_each<I, O, F, Fut>(
    items: Vec<I>,
    mut ask: F,
) -> corsa::Result<Vec<Option<O>>>
where
    I: Clone,
    F: FnMut(Vec<I>) -> Fut,
    Fut: Future<Output = corsa::Result<Vec<Option<O>>>>,
{
    let total = items.len();
    let first = match ask(items.clone()).await {
        Ok(answers) => return Ok(answers),
        Err(error) if is_recovered_panic(&error) && total > 0 => error,
        Err(error) => return Err(error),
    };
    let mut answers = Vec::with_capacity(total);
    answers.resize_with(total, || None);
    let mut budget = PANIC_BUDGET;
    // `(start, len)` slices still to answer, the earliest on top.
    let mut pending = vec![(0usize, total)];
    let mut failed = Some(first);
    while let Some((start, len)) = pending.pop() {
        if let Some(error) = failed.take() {
            budget = budget.saturating_sub(1);
            if budget == 0 {
                return Err(error);
            }
            if len > 1 {
                let half = len / 2;
                pending.push((start + half, len - half));
                pending.push((start, half));
            }
            continue;
        }
        let slice = items.get(start..start + len).unwrap_or_default().to_vec();
        match ask(slice).await {
            Ok(found) => {
                for (offset, answer) in found.into_iter().take(len).enumerate() {
                    if let Some(slot) = answers.get_mut(start + offset) {
                        *slot = answer;
                    }
                }
            }
            Err(error) if is_recovered_panic(&error) => {
                failed = Some(error);
                pending.push((start, len));
            }
            Err(error) => return Err(error),
        }
    }
    Ok(answers)
}

#[cfg(test)]
mod tests {
    use super::{PANIC_BUDGET, answer_each, is_recovered_panic};
    use corsa::{CorsaError, error::RpcResponseError, runtime::block_on};
    use std::cell::Cell;

    fn panic_error() -> CorsaError {
        CorsaError::Rpc(RpcResponseError {
            code: -32603,
            message: "panic: interface conversion: checker.TypeData is *checker.TypeReference, not *checker.TupleType".into(),
            data: None,
        })
    }

    /// Doubles each item; a batch holding a negative item panics.
    fn ask(
        calls: &Cell<usize>,
    ) -> impl FnMut(Vec<i32>) -> std::future::Ready<corsa::Result<Vec<Option<i32>>>> + '_ {
        move |batch| {
            calls.set(calls.get() + 1);
            std::future::ready(if batch.iter().any(|item| *item < 0) {
                Err(panic_error())
            } else {
                Ok(batch.into_iter().map(|item| Some(item * 2)).collect())
            })
        }
    }

    #[test]
    fn a_clean_batch_is_one_request() {
        let calls = Cell::new(0);
        let answers = block_on(answer_each(vec![1, 2, 3], ask(&calls))).unwrap();
        assert_eq!(answers, vec![Some(2), Some(4), Some(6)]);
        assert_eq!(calls.get(), 1);
    }

    #[test]
    fn only_the_panicking_items_go_unanswered() {
        let calls = Cell::new(0);
        let items = vec![1, -1, 3, 4, 5, 6, -7, 8];
        let answers = block_on(answer_each(items, ask(&calls))).unwrap();
        assert_eq!(
            answers,
            vec![
                Some(2),
                None,
                Some(6),
                Some(8),
                Some(10),
                Some(12),
                None,
                Some(16)
            ]
        );
        // Two panicking items in eight: the first batch, then two halvings
        // down to each of them.
        assert_eq!(calls.get(), 11);
    }

    #[test]
    fn a_systematic_panic_is_returned_after_the_budget() {
        let calls = Cell::new(0);
        let items = vec![-1; PANIC_BUDGET * 4];
        let error = block_on(answer_each(items, ask(&calls))).unwrap_err();
        assert!(is_recovered_panic(&error));
        assert_eq!(calls.get(), PANIC_BUDGET);
    }

    #[test]
    fn other_errors_are_not_retried() {
        let calls = Cell::new(0);
        let error = block_on(answer_each(vec![1, 2], |_batch: Vec<i32>| {
            calls.set(calls.get() + 1);
            std::future::ready(Err::<Vec<Option<i32>>, _>(CorsaError::Closed("stdio")))
        }))
        .unwrap_err();
        assert!(matches!(error, CorsaError::Closed(_)));
        assert_eq!(calls.get(), 1);
        assert!(is_recovered_panic(&CorsaError::Protocol(
            "panic: interface conversion".into()
        )));
        assert!(!is_recovered_panic(&CorsaError::Protocol(
            "Incomplete expression type batch".into()
        )));
    }
}
