//! Interpret terminal evidence only after the owned editor transport is closed.

use corsa_lsp::process::{ProcessObserver, ProcessReceipt};
use std::io::Write;
use vize_l0::{String, cstr};

fn terminal_complete(receipt: &ProcessReceipt) -> bool {
    receipt.completed
        && receipt.stderr_complete
        && receipt.stderr_error.is_none()
        && receipt.process_error.is_none()
        && receipt.reaped
        && !receipt.timed_out
        && !receipt.kill_requested
        && receipt.exit_status.is_some_and(|status| status.success())
}

pub(super) fn report(
    observer: &ProcessObserver,
    graceful_acknowledged: bool,
    require_clean_exit: bool,
) -> Result<(), String> {
    report_to(
        &observer.receipts(),
        graceful_acknowledged,
        require_clean_exit,
        &mut std::io::stderr().lock(),
    )
}

fn report_to(
    receipts: &[ProcessReceipt],
    graceful_acknowledged: bool,
    require_clean_exit: bool,
    output: &mut impl Write,
) -> Result<(), String> {
    let complete = receipts.len() == 1 && receipts.first().is_some_and(terminal_complete);
    for receipt in receipts {
        // The authenticated native EOF lifecycle finishes without stderr.
        // Unknown servers and wrappers retain every byte, even a byte-for-byte
        // copy of a previously observed native Exit error line.
        output.write_all(&receipt.stderr).map_err(|error| {
            cstr!("Failed to retain native editor process stderr: {error}; {receipts:?}")
        })?;
    }
    if require_clean_exit && !(graceful_acknowledged && complete) {
        let error = cstr!("Native editor process did not finish cleanly: {receipts:?}");
        tracing::warn!(error = %error, "native editor termination was incomplete");
        return Err(error);
    }
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    const NORMAL_EXIT_STDERR: &[u8] = b"error handling method 'exit': EOF\n";
    use std::{os::unix::process::ExitStatusExt, process::ExitStatus, time::Duration};

    fn receipt(stderr: &[u8], exit_code: i32) -> ProcessReceipt {
        ProcessReceipt {
            pid: 41,
            completed: true,
            stderr: stderr.to_vec(),
            stderr_complete: true,
            stderr_error: None,
            exit_status: Some(ExitStatus::from_raw(exit_code << 8)),
            process_error: None,
            timed_out: false,
            kill_requested: false,
            reaped: true,
            elapsed: Duration::ZERO,
        }
    }

    #[test]
    fn even_acknowledged_terminal_stderr_is_retained_without_normalization() {
        for (bytes, acknowledged, expected) in [
            (NORMAL_EXIT_STDERR, true, NORMAL_EXIT_STDERR),
            (NORMAL_EXIT_STDERR, false, NORMAL_EXIT_STDERR),
            (
                b"error handling method 'exit': EOF",
                true,
                b"error handling method 'exit': EOF".as_slice(),
            ),
            (
                b"error handling method 'exit': EOF\n\0\xff",
                true,
                b"error handling method 'exit': EOF\n\0\xff".as_slice(),
            ),
            (
                b"unexpected native error\n",
                true,
                b"unexpected native error\n".as_slice(),
            ),
        ] {
            let mut output = Vec::new();
            let result = report_to(&[receipt(bytes, 0)], acknowledged, true, &mut output);
            assert_eq!(result.is_ok(), acknowledged);
            assert_eq!(output, expected);
        }
    }

    #[test]
    fn unsuccessful_or_incomplete_native_termination_retains_every_original_byte() {
        let mut cases = vec![receipt(NORMAL_EXIT_STDERR, 2)];
        for change in 0..8 {
            let mut row = receipt(NORMAL_EXIT_STDERR, 0);
            match change {
                0 => row.completed = false,
                1 => row.stderr_complete = false,
                2 => row.stderr_error = Some("reader failed".into()),
                3 => row.process_error = Some("PID mismatch".into()),
                4 => row.timed_out = true,
                5 => row.kill_requested = true,
                6 => row.reaped = false,
                7 => row.exit_status = None,
                _ => unreachable!(),
            }
            cases.push(row);
        }
        let mut verdicts = Vec::new();
        for row in cases {
            let mut output = Vec::new();
            let result = report_to(&[row], true, true, &mut output);
            verdicts.push(result.is_err());
            assert_eq!(output, NORMAL_EXIT_STDERR);
        }
        assert_eq!(verdicts, vec![true; 9]);
        for rows in [
            vec![],
            vec![receipt(NORMAL_EXIT_STDERR, 0), receipt(b"\xff\0", 0)],
        ] {
            let mut output = Vec::new();
            assert!(report_to(&rows, true, true, &mut output).is_err());
            assert_eq!(
                output,
                rows.iter()
                    .flat_map(|row| row.stderr.iter().copied())
                    .collect::<Vec<_>>()
            );
        }
    }
}
