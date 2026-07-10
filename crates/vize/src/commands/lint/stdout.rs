use std::io::{self, Write};
use std::time::Duration;
use vize_carton::cstr;
use vize_patina::format_summary;

pub(super) fn write(bytes: &[u8]) {
    let mut stdout = io::stdout().lock();
    if let Err(error) = write_all_retry(&mut stdout, bytes) {
        handle_stdout_error(error);
    }
}

pub(super) fn write_text_summary(
    total_errors: usize,
    total_warnings: usize,
    file_count: usize,
    elapsed: Duration,
    cross_file_tree: Option<&str>,
) {
    write(
        cstr!(
            "\n{}\n",
            format_summary(total_errors, total_warnings, file_count)
        )
        .as_bytes(),
    );
    write(cstr!("Linted {} files in {:.4?}\n", file_count, elapsed).as_bytes());
    if let Some(tree) = cross_file_tree {
        write(cstr!("\n{tree}\n").as_bytes());
    }
}

fn write_all_retry<W: Write>(writer: &mut W, bytes: &[u8]) -> io::Result<()> {
    const MAX_CONSECUTIVE_WOULD_BLOCK: usize = 1024;

    let mut written = 0;
    let mut would_block_count = 0;
    while written < bytes.len() {
        match writer.write(&bytes[written..]) {
            Ok(0) => {
                return Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "failed to write lint output",
                ));
            }
            Ok(count) => {
                written += count;
                would_block_count = 0;
            }
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::Interrupted | io::ErrorKind::WouldBlock
                ) =>
            {
                would_block_count += usize::from(error.kind() == io::ErrorKind::WouldBlock);
                if would_block_count > MAX_CONSECUTIVE_WOULD_BLOCK {
                    return Err(error);
                }
                std::thread::sleep(Duration::from_millis(1));
            }
            Err(error) => return Err(error),
        }
    }
    Ok(())
}

fn handle_stdout_error(error: io::Error) -> ! {
    if error.kind() == io::ErrorKind::BrokenPipe {
        std::process::exit(0);
    }
    eprintln!(
        "\x1b[31mError:\x1b[0m failed to write lint output: {}",
        error
    );
    std::process::exit(1);
}

#[cfg(test)]
mod tests {
    use super::write_all_retry;
    use std::io::{self, Write};

    #[test]
    fn stdout_writer_retries_temporary_would_block() {
        struct WouldBlockOnce {
            attempts: usize,
            bytes: Vec<u8>,
        }

        impl Write for WouldBlockOnce {
            fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
                self.attempts += 1;
                if self.attempts == 1 {
                    return Err(io::Error::from(io::ErrorKind::WouldBlock));
                }
                self.bytes.extend_from_slice(buf);
                Ok(buf.len())
            }

            fn flush(&mut self) -> io::Result<()> {
                Ok(())
            }
        }

        let mut writer = WouldBlockOnce {
            attempts: 0,
            bytes: Vec::new(),
        };

        write_all_retry(&mut writer, b"lint output").unwrap();

        assert_eq!(writer.bytes, b"lint output");
        assert_eq!(writer.attempts, 2);
    }
}
