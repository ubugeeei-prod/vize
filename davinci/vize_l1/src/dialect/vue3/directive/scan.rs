//! Iterative, allocation-free lexical boundary scan, not a JS parser.

use super::{DirectiveNameError, MAX_DIRECTIVE_DELIMITER_RUNS};

fn boundary(byte: u8) -> bool {
    byte == b'=' || byte.is_ascii_whitespace() || matches!(byte, b'/' | b'>')
}

#[derive(Clone, Copy)]
struct Run {
    delimiter: u8,
    count: usize,
}

struct Delimiters {
    runs: [Run; MAX_DIRECTIVE_DELIMITER_RUNS],
    length: usize,
}

impl Delimiters {
    fn new() -> Self {
        Self {
            runs: [Run {
                delimiter: 0,
                count: 0,
            }; MAX_DIRECTIVE_DELIMITER_RUNS],
            length: 0,
        }
    }

    fn top(&self) -> u8 {
        self.runs
            .get(self.length.saturating_sub(1))
            .map_or(0, |run| run.delimiter)
    }

    fn push(&mut self, delimiter: u8) -> Result<(), DirectiveNameError> {
        if self.length > 0
            && let Some(run) = self.runs.get_mut(self.length - 1)
            && run.delimiter == delimiter
        {
            // Every push consumes input bytes, so this counter is
            // bounded by the admitted string length plus its opener.
            run.count += 1;
            return Ok(());
        }
        let run = self
            .runs
            .get_mut(self.length)
            .ok_or(DirectiveNameError::NestingLimit)?;
        *run = Run {
            delimiter,
            count: 1,
        };
        self.length += 1;
        Ok(())
    }

    fn pop(&mut self) {
        if self.length > 0
            && let Some(run) = self.runs.get_mut(self.length - 1)
        {
            run.count -= 1;
            if run.count == 0 {
                self.length -= 1;
            }
        }
    }
}

pub(super) fn argument(input: &[u8], start: usize) -> Result<(usize, bool), DirectiveNameError> {
    let mut stack = Delimiters::new();
    stack.push(b']')?;
    let mut index = start;
    while let Some(&byte) = input.get(index) {
        if boundary(byte) {
            return Ok((index, false));
        }
        let delimiter = stack.top();
        if matches!(delimiter, b'\'' | b'"' | b'`') {
            match byte {
                b'\\' => {
                    if input.get(index + 1).is_some_and(|&next| boundary(next)) {
                        return Ok((index + 1, false));
                    }
                    index += 1;
                }
                _ if byte == delimiter => stack.pop(),
                b'$' if delimiter == b'`' && input.get(index + 1) == Some(&b'{') => {
                    stack.push(b'}')?;
                    index += 1;
                }
                _ => {}
            }
        } else {
            match byte {
                b'\'' | b'"' | b'`' => stack.push(byte)?,
                b'[' => stack.push(b']')?,
                b'(' => stack.push(b')')?,
                b'{' => stack.push(b'}')?,
                b']' | b')' | b'}' if byte == delimiter => {
                    stack.pop();
                    if stack.length == 0 {
                        return Ok((index, true));
                    }
                }
                _ => {}
            }
        }
        index += 1;
    }
    Ok((input.len(), false))
}
