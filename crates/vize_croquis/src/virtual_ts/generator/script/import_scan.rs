//! Import statement scanning for virtual TypeScript generation.
//!
//! Virtual TypeScript is emitted line by line, so imports have to be located
//! without a full parse. These helpers group the lines of one import statement,
//! track which lines start in code context, and locate relative module
//! specifiers, all while ignoring comments, strings, and template literals.

use vize_carton::{String, ToCompactString};

pub(super) use self::context::code_line_starts;

mod context;

/// Report the last line index of the import statement starting at `start`.
///
/// Script content is processed line by line, so a wrapped import such as
/// `import {\n  a,\n} from "./x.ts";` must be recognized as one statement.
/// Emitting only its first line at module scope would leave the remaining
/// lines inside the setup body, making the virtual TypeScript unparseable and
/// erasing every type the file declares.
///
/// `code_starts` marks which lines begin in code context, so `import`-looking
/// text inside a template literal or a block comment is never mistaken for a
/// statement. Continuation lines are consumed only while the named-binding
/// braces stay open, or when the next line opens the trailing `from` clause,
/// so a line that merely looks like an import can never swallow the rest of
/// the script.
///
/// Returns `None` when the line does not begin an import statement.
pub(super) fn import_statement_end(
    lines: &[&str],
    code_starts: &[bool],
    start: usize,
) -> Option<usize> {
    if !code_starts.get(start).copied().unwrap_or(false) {
        return None;
    }
    if !lines.get(start)?.trim().starts_with("import ") {
        return None;
    }

    let mut statement = String::with_capacity(128);
    for (offset, line) in lines[start..].iter().enumerate() {
        if offset > 0 {
            statement.push('\n');
        }
        statement.push_str(line.trim());

        let state = scan_import_statement(&statement);
        if state.is_complete() {
            return Some(start + offset);
        }
        if state.is_open() {
            continue;
        }
        // The named-binding clause is balanced but no module specifier has
        // been seen: `import { a }\nfrom "./x";` is one statement, while
        // anything else ends it here rather than eating unrelated code.
        if opens_from_clause(lines.get(start + offset + 1)) {
            continue;
        }
        return Some(start + offset);
    }

    Some(start)
}

/// Whether a line opens the trailing `from "…"` clause of an import.
fn opens_from_clause(line: Option<&&str>) -> bool {
    let Some(rest) = line.and_then(|line| line.trim_start().strip_prefix("from")) else {
        return false;
    };
    rest.starts_with([' ', '\t', '"', '\''])
}

/// Join the lines of one import statement into a single logical statement.
///
/// Line structure is preserved: collapsing the lines into one would move any
/// trailing `//` comment in front of the bindings that follow it and comment
/// out the rest of the statement.
pub(super) fn join_statement_lines(lines: &[&str]) -> String {
    if let [line] = lines {
        return line.to_compact_string();
    }

    let mut statement = String::with_capacity(128);
    for (offset, line) in lines.iter().enumerate() {
        if offset > 0 {
            statement.push('\n');
        }
        statement.push_str(line);
    }
    statement
}

/// Brace, string, and specifier state of an accumulated import statement.
struct ImportStatementState {
    /// Unbalanced `{` count of the named-binding clause.
    depth: i32,
    /// Whether a string literal is still open.
    quote_open: bool,
    /// Whether a closed quoted module specifier has been seen.
    has_specifier: bool,
}

impl ImportStatementState {
    /// A statement is complete once its braces balance, no string stays open,
    /// and a quoted module specifier has been seen.
    fn is_complete(&self) -> bool {
        !self.quote_open && self.depth <= 0 && self.has_specifier
    }

    /// A statement continues onto the next line only while its named-binding
    /// clause is still open.
    fn is_open(&self) -> bool {
        !self.quote_open && self.depth > 0
    }
}

/// Scan an accumulated import statement for brace, string, and comment state.
///
/// Comments are skipped so that a `}` or a quote inside `// …` or `/* … */`
/// never ends the statement early. The whole accumulated statement is rescanned
/// on every added line, so comment state spanning lines resolves naturally.
fn scan_import_statement(statement: &str) -> ImportStatementState {
    let bytes = statement.as_bytes();
    let mut depth: i32 = 0;
    let mut quote: Option<u8> = None;
    let mut has_specifier = false;
    let mut line_comment = false;
    let mut block_comment = false;
    let mut index = 0;

    while index < bytes.len() {
        let byte = bytes[index];

        if line_comment {
            if byte == b'\n' {
                line_comment = false;
            }
            index += 1;
            continue;
        }

        if block_comment {
            if byte == b'*' && bytes.get(index + 1) == Some(&b'/') {
                block_comment = false;
                index += 2;
                continue;
            }
            index += 1;
            continue;
        }

        match quote {
            Some(open) => {
                if byte == b'\\' {
                    index += 2;
                    continue;
                }
                if byte == open {
                    quote = None;
                    has_specifier = true;
                }
            }
            None => match byte {
                b'/' if bytes.get(index + 1) == Some(&b'/') => {
                    line_comment = true;
                    index += 2;
                    continue;
                }
                b'/' if bytes.get(index + 1) == Some(&b'*') => {
                    block_comment = true;
                    index += 2;
                    continue;
                }
                b'"' | b'\'' | b'`' => quote = Some(byte),
                b'{' => depth += 1,
                b'}' => depth -= 1,
                _ => {}
            },
        }
        index += 1;
    }

    ImportStatementState {
        depth,
        quote_open: quote.is_some(),
        has_specifier,
    }
}

/// Find a relative `from "./..."` import range with a byte scanner.
///
/// Script virtual-TS generation calls this for every import-looking line. A
/// regex was measurably expensive in that loop, and the grammar we need is
/// intentionally narrow: skip to `from`, require whitespace, accept a quoted
/// path that begins with `.`, then return the path slice boundaries.
pub(super) fn find_relative_import_from_range(line: &str) -> Option<(usize, usize, usize, usize)> {
    let bytes = line.as_bytes();
    let mut search_start = 0;

    while let Some(offset) = line[search_start..].find("from") {
        let from_start = search_start + offset;
        let mut cursor = from_start + 4;

        if !bytes
            .get(cursor)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            search_start = cursor;
            continue;
        }

        while bytes
            .get(cursor)
            .is_some_and(|byte| byte.is_ascii_whitespace())
        {
            cursor += 1;
        }

        let Some(quote @ (b'\'' | b'"')) = bytes.get(cursor).copied() else {
            search_start = cursor.saturating_add(1);
            continue;
        };
        let path_start = cursor + 1;
        if bytes.get(path_start) != Some(&b'.') {
            search_start = path_start;
            continue;
        }

        let mut path_end = path_start + 1;
        while bytes.get(path_end).is_some_and(|byte| *byte != quote) {
            path_end += 1;
        }
        if bytes.get(path_end) != Some(&quote) {
            return None;
        }

        return Some((from_start, path_start, path_end, path_end));
    }

    None
}

#[cfg(test)]
mod tests;
