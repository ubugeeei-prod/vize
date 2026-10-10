//! Isolate engine defects while keeping the formatter's existing error policy.

use std::panic::{AssertUnwindSafe, catch_unwind};

use crate::error::FormatError;

pub(super) fn catch<T>(operation: impl FnOnce() -> T) -> Result<T, FormatError> {
    catch_unwind(AssertUnwindSafe(operation)).map_err(|_| {
        FormatError::StyleFormatError(
            "the CSS engine hit an internal defect (upstream lightningcss panic; see vize issue #3295)"
                .into(),
        )
    })
}
