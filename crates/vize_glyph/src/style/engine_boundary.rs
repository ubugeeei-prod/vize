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

#[cfg(test)]
mod tests {
    use super::catch;
    use crate::FormatError;

    #[test]
    fn successful_engine_output_is_preserved() {
        assert_eq!(
            catch(|| ".box { color: red; }").unwrap(),
            ".box { color: red; }"
        );
    }

    #[test]
    fn engine_defects_return_style_errors_without_poisoning_the_next_request() {
        for owned_payload in [false, true] {
            let result = catch(|| {
                if owned_payload {
                    std::panic::panic_any(std::string::String::from("upstream defect"));
                }
                panic!("upstream defect");
            });
            let Err(FormatError::StyleFormatError(message)) = result else {
                panic!("engine panic must become a style formatting error");
            };
            assert!(message.contains("lightningcss panic"));
            assert!(message.contains("#3295"));
            assert_eq!(catch(|| 42).unwrap(), 42);
        }
    }
}
