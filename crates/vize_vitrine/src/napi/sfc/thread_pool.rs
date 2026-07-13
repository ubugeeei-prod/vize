use napi::{Error, Result, Status};
use std::sync::{Mutex, OnceLock};

/// Bound both a single request and all live explicit pools in the process while
/// leaving ample headroom for large build hosts.
pub(super) const MAX_BATCH_THREADS: u32 = 256;

struct ThreadAdmission {
    capacity: u32,
    live_threads: Mutex<u32>,
}

impl ThreadAdmission {
    fn new(capacity: u32) -> Self {
        Self {
            capacity,
            live_threads: Mutex::new(0),
        }
    }

    fn reserve(&self, threads: u32) -> Result<ThreadReservation<'_>> {
        debug_assert!(threads <= self.capacity);
        let mut live_threads = self
            .live_threads
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let available = self.capacity.saturating_sub(*live_threads);
        if threads > available {
            return Err(Error::new(
                Status::WouldDeadlock,
                format!(
                    "batch thread capacity exhausted: requested {threads}, available {available}, process-wide limit {}; retry after another explicit batch finishes or omit threads to use the shared pool",
                    self.capacity
                ),
            ));
        }
        *live_threads += threads;
        Ok(ThreadReservation {
            admission: self,
            threads,
        })
    }
}

struct ThreadReservation<'a> {
    admission: &'a ThreadAdmission,
    threads: u32,
}

impl Drop for ThreadReservation<'_> {
    fn drop(&mut self) {
        let mut live_threads = self
            .admission
            .live_threads
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        debug_assert!(*live_threads >= self.threads);
        *live_threads = live_threads.saturating_sub(self.threads);
    }
}

fn batch_thread_admission() -> &'static ThreadAdmission {
    static ADMISSION: OnceLock<ThreadAdmission> = OnceLock::new();
    ADMISSION.get_or_init(|| ThreadAdmission::new(MAX_BATCH_THREADS))
}

/// A per-call Rayon pool when the caller requests an explicit thread count.
/// Calls without an override continue to use Rayon's shared global pool.
pub(super) struct BatchThreadPool {
    pool: Option<rayon::ThreadPool>,
    _reservation: Option<ThreadReservation<'static>>,
}

impl BatchThreadPool {
    pub(super) fn new(threads: Option<u32>) -> Result<Self> {
        let Some(threads) = threads else {
            return Ok(Self {
                pool: None,
                _reservation: None,
            });
        };

        if !(1..=MAX_BATCH_THREADS).contains(&threads) {
            return Err(Error::new(
                Status::InvalidArg,
                format!("threads must be between 1 and {MAX_BATCH_THREADS}; received {threads}"),
            ));
        }

        let reservation = batch_thread_admission().reserve(threads)?;
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(threads as usize)
            .build()
            .map_err(|error| {
                Error::new(
                    Status::GenericFailure,
                    format!("failed to create a {threads}-thread batch pool: {error}"),
                )
            })?;
        Ok(Self {
            pool: Some(pool),
            _reservation: Some(reservation),
        })
    }

    pub(super) fn install<Operation, Output>(&self, operation: Operation) -> Output
    where
        Operation: FnOnce() -> Output + Send,
        Output: Send,
    {
        match &self.pool {
            Some(pool) => pool.install(operation),
            None => operation(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{BatchThreadPool, MAX_BATCH_THREADS, ThreadAdmission};
    use std::sync::{Arc, Barrier};

    fn observed_threads(threads: u32) -> usize {
        BatchThreadPool::new(Some(threads))
            .unwrap()
            .install(rayon::current_num_threads)
    }

    #[test]
    fn explicit_thread_counts_are_call_scoped_in_both_orders() {
        let global_threads = rayon::current_num_threads();
        assert_eq!(observed_threads(1), 1);
        assert_eq!(observed_threads(4), 4);
        assert_eq!(observed_threads(4), 4);
        assert_eq!(observed_threads(1), 1);
        assert_eq!(rayon::current_num_threads(), global_threads);

        let default = BatchThreadPool::new(None).unwrap();
        assert_eq!(default.install(rayon::current_num_threads), global_threads);
    }

    #[test]
    fn concurrent_overrides_do_not_interfere() {
        let barrier = Arc::new(Barrier::new(2));
        std::thread::scope(|scope| {
            let single_barrier = Arc::clone(&barrier);
            let single = scope.spawn(move || {
                BatchThreadPool::new(Some(1)).unwrap().install(|| {
                    single_barrier.wait();
                    rayon::current_num_threads()
                })
            });
            let parallel_barrier = Arc::clone(&barrier);
            let parallel = scope.spawn(move || {
                BatchThreadPool::new(Some(3)).unwrap().install(|| {
                    parallel_barrier.wait();
                    rayon::current_num_threads()
                })
            });
            assert_eq!(single.join().unwrap(), 1);
            assert_eq!(parallel.join().unwrap(), 3);
        });
    }

    #[test]
    fn aggregate_admission_fails_fast_and_releases_capacity() {
        let admission = ThreadAdmission::new(3);
        let first = admission.reserve(2).unwrap();

        let saturated = admission.reserve(2).err().unwrap();
        assert_eq!(saturated.status, napi::Status::WouldDeadlock);
        assert!(saturated.reason.contains("requested 2"));
        assert!(saturated.reason.contains("available 1"));
        assert!(saturated.reason.contains("process-wide limit 3"));

        drop(first);
        let full_capacity = admission.reserve(3).unwrap();
        drop(full_capacity);
        assert!(admission.reserve(3).is_ok());
    }

    #[test]
    fn invalid_thread_counts_fail_before_pool_creation() {
        let zero = BatchThreadPool::new(Some(0)).err().unwrap();
        assert_eq!(zero.status, napi::Status::InvalidArg);
        assert!(zero.reason.contains("between 1 and 256"));

        let excessive = BatchThreadPool::new(Some(MAX_BATCH_THREADS + 1))
            .err()
            .unwrap();
        assert_eq!(excessive.status, napi::Status::InvalidArg);
        assert!(excessive.reason.contains("received 257"));
    }
}
