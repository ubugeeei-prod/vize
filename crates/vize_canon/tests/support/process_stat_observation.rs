//! Decode an owned Linux proc-stat read without hiding observation failures.

use std::io;

pub(super) fn observed(result: io::Result<String>) -> Option<String> {
    match result {
        Ok(stat) => Some(stat),
        // Linux /proc retirement can return ESRCH (3) instead of ENOENT.
        Err(error)
            if error.kind() == io::ErrorKind::NotFound || error.raw_os_error() == Some(3) =>
        {
            None
        }
        Err(error) => panic!("cannot observe owned native life: {error}"),
    }
}

#[cfg(test)]
mod tests {
    use super::observed;
    use std::{io, panic::catch_unwind};

    #[test]
    fn retired_proc_reads_accept_only_missing_path_or_missing_process() {
        for errno in [2, 3] {
            assert_eq!(observed(Err(io::Error::from_raw_os_error(errno))), None);
        }
    }

    #[test]
    fn observable_live_and_zombie_records_keep_every_byte() {
        for stat in ["42 (native) S 1 2 3\n", "42 (native) Z 1 2 3\n"] {
            assert_eq!(observed(Ok(stat.into())), Some(stat.into()));
        }
    }

    #[test]
    fn unexpected_observation_errors_remain_fatal() {
        for errno in [4, 5, 13, 22] {
            assert!(catch_unwind(|| observed(Err(io::Error::from_raw_os_error(errno)))).is_err());
        }
        assert!(catch_unwind(|| observed(Err(io::Error::other("unknown failure")))).is_err());
    }
}
