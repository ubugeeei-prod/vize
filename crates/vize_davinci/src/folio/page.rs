//! The runtime half of `#[derive(Folio)]`: the derived-page printer and the
//! parse driver the generated code calls into.
//!
//! The derived page format is documented in
//! `davinci-road/plan/folio-format.md` ("Derived pages"). In short: a
//! `[page]` header section carrying `name=value` scalar lines in field
//! declaration order, then one `[page.field]` section per `Vec` (entries in
//! order) or `FxHashMap` field (entries sorted by printed key). Empty
//! sections are omitted, every printed section is followed by exactly one
//! blank line, lines are LF, and parse errors carry 1-based line numbers.
//!
//! Parse is lenient exactly where print normalizes - scalar-line order,
//! section order, extra blank lines, non-canonical value spellings - and
//! strict everywhere else, mirroring the croquis parser's discipline. Every
//! rejection message here is part of the contract and tested exactly.

use core::fmt;
use core::hash::Hash;

use alloc::vec::Vec;

use vize_s0::{FxHashMap, String, cstr};

use super::FolioError;
use super::value::FolioValue;

// --- printing ---------------------------------------------------------------

/// Prints one derived page in canonical form.
pub struct PagePrinter<'w, W: fmt::Write> {
    w: &'w mut W,
    page: &'static str,
}

impl<'w, W: fmt::Write> PagePrinter<'w, W> {
    /// A printer for `page` writing into `w`.
    pub fn new(w: &'w mut W, page: &'static str) -> Self {
        Self { w, page }
    }

    /// Write the `[page]` header line.
    ///
    /// # Errors
    ///
    /// Propagates the writer's failure (as do all printing methods below).
    pub fn open(&mut self) -> fmt::Result {
        writeln!(self.w, "[{}]", self.page)
    }

    /// Write one `name=value` scalar line.
    pub fn scalar<T: FolioValue>(&mut self, name: &str, value: &T) -> fmt::Result {
        write!(self.w, "{name}=")?;
        value.print_value(self.w)?;
        self.w.write_char('\n')
    }

    /// Close the header section with its blank line.
    pub fn close_header(&mut self) -> fmt::Result {
        self.w.write_char('\n')
    }

    /// Write a `[page.name]` list section, one entry per line in order.
    /// An empty list is omitted entirely.
    pub fn list<T: FolioValue>(&mut self, name: &str, items: &[T]) -> fmt::Result {
        if items.is_empty() {
            return Ok(());
        }
        writeln!(self.w, "[{}.{name}]", self.page)?;
        for item in items {
            item.print_value(self.w)?;
            self.w.write_char('\n')?;
        }
        self.w.write_char('\n')
    }

    /// Write a `[page.name]` map section, `key=value` per line, sorted by
    /// printed key (lexicographic, byte order - normalization rule 1). An
    /// empty map is omitted entirely.
    pub fn map<K: FolioValue, V: FolioValue>(
        &mut self,
        name: &str,
        map: &FxHashMap<K, V>,
    ) -> fmt::Result {
        if map.is_empty() {
            return Ok(());
        }
        let mut entries: Vec<(String, &V)> = Vec::with_capacity(map.len());
        for (key, value) in map {
            let mut printed = String::default();
            key.print_value(&mut printed)?;
            entries.push((printed, value));
        }
        entries.sort_unstable_by(|a, b| a.0.as_str().cmp(b.0.as_str()));
        writeln!(self.w, "[{}.{name}]", self.page)?;
        for (key, value) in entries {
            self.w.write_str(key.as_str())?;
            self.w.write_char('=')?;
            value.print_value(self.w)?;
            self.w.write_char('\n')?;
        }
        self.w.write_char('\n')
    }
}

// --- parsing ----------------------------------------------------------------

/// What one physical line means to the generated parser.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineEvent<'a> {
    /// Blank line or a consumed section header - nothing for the caller.
    Skip,
    /// A scalar line inside the `[page]` header section.
    Field,
    /// A `[page.<name>]` header; the caller matches `<name>` against its
    /// known sections and calls [`ParseState::enter_section`].
    Section(&'a str),
    /// An entry line inside the section entered with this index.
    Entry(usize),
}

/// Where in the page the line-by-line scan currently is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Cursor {
    BeforeHeader,
    Header,
    Section(usize),
}

/// Parse driver state for one derived page.
///
/// Tracks the header, the current section, and which sections were already
/// seen (at most 64 per page, enforced by the derive).
pub struct ParseState {
    page: &'static str,
    cursor: Cursor,
    seen: u64,
}

impl ParseState {
    /// Fresh state for `page`.
    #[must_use]
    pub fn new(page: &'static str) -> Self {
        Self {
            page,
            cursor: Cursor::BeforeHeader,
            seen: 0,
        }
    }

    /// Classify one physical line.
    ///
    /// # Errors
    ///
    /// Rejects content before the header, a second `[page]` header, an
    /// unknown top-level section, and (via the messages here) everything
    /// the format doc calls strict.
    pub fn classify<'a>(
        &mut self,
        line: &'a str,
        line_no: usize,
    ) -> Result<LineEvent<'a>, FolioError> {
        if let Some(name) = line
            .strip_prefix('[')
            .and_then(|rest| rest.strip_suffix(']'))
        {
            if self.cursor == Cursor::BeforeHeader {
                if name == self.page {
                    self.cursor = Cursor::Header;
                    return Ok(LineEvent::Skip);
                }
                return Err(FolioError::new(
                    line_no,
                    cstr!("first section must be [{}]", self.page),
                ));
            }
            if name == self.page {
                return Err(FolioError::new(
                    line_no,
                    cstr!("duplicate section [{}]", self.page),
                ));
            }
            if let Some(sub) = name
                .strip_prefix(self.page)
                .and_then(|rest| rest.strip_prefix('.'))
            {
                return Ok(LineEvent::Section(sub));
            }
            return Err(FolioError::new(line_no, cstr!("unknown section [{name}]")));
        }
        if line.is_empty() {
            return Ok(LineEvent::Skip);
        }
        match self.cursor {
            Cursor::BeforeHeader => Err(FolioError::new(
                line_no,
                cstr!("content before the [{}] header", self.page),
            )),
            Cursor::Header => Ok(LineEvent::Field),
            Cursor::Section(index) => Ok(LineEvent::Entry(index)),
        }
    }

    /// Enter section `index` (the derive's declaration-order index for the
    /// field), rejecting a duplicate.
    ///
    /// # Errors
    ///
    /// Returns `duplicate section [page.name]` when the section was already
    /// entered.
    pub fn enter_section(
        &mut self,
        index: usize,
        name: &str,
        line_no: usize,
    ) -> Result<(), FolioError> {
        let bit = 1u64 << index;
        if self.seen & bit != 0 {
            return Err(FolioError::new(
                line_no,
                cstr!("duplicate section [{}.{name}]", self.page),
            ));
        }
        self.seen |= bit;
        self.cursor = Cursor::Section(index);
        Ok(())
    }

    /// The rejection for a `[page.<name>]` the type does not declare.
    #[must_use]
    pub fn unknown_section(&self, name: &str, line_no: usize) -> FolioError {
        FolioError::new(line_no, cstr!("unknown section [{}.{name}]", self.page))
    }

    /// Reject the whole input when the `[page]` header never appeared.
    ///
    /// # Errors
    ///
    /// Returns `missing [page] header` attributed to no line.
    pub fn require_header(&self) -> Result<(), FolioError> {
        if self.cursor == Cursor::BeforeHeader {
            return Err(FolioError::new(0, cstr!("missing [{}] header", self.page)));
        }
        Ok(())
    }
}

// --- generated-code helpers ---------------------------------------------------

/// Split a header line at its first `=` into `(name, value)`.
///
/// # Errors
///
/// Rejects a line with no `=` at all.
pub fn split_field(line: &str, line_no: usize) -> Result<(&str, &str), FolioError> {
    line.split_once('=')
        .ok_or_else(|| FolioError::new(line_no, cstr!("field line is missing `=`")))
}

/// Store a parsed scalar into its slot, rejecting a second occurrence.
///
/// # Errors
///
/// Returns `duplicate field` for a repeat, or the value's own parse error.
pub fn set_scalar<T: FolioValue>(
    slot: &mut Option<T>,
    name: &str,
    value: &str,
    line_no: usize,
) -> Result<(), FolioError> {
    if slot.is_some() {
        return Err(FolioError::new(line_no, cstr!("duplicate field `{name}`")));
    }
    *slot = Some(T::parse_value(value, line_no)?);
    Ok(())
}

/// The rejection for a header line naming a field the type does not have.
#[must_use]
pub fn unknown_field(name: &str, line_no: usize) -> FolioError {
    FolioError::new(line_no, cstr!("unknown field `{name}`"))
}

/// Unwrap a required scalar after the scan, rejecting an absent one.
///
/// # Errors
///
/// Returns `missing field` attributed to no line.
pub fn require_scalar<T>(slot: Option<T>, name: &str) -> Result<T, FolioError> {
    slot.ok_or_else(|| FolioError::new(0, cstr!("missing field `{name}`")))
}

/// Parse one `key=value` map entry line and insert it, rejecting a
/// duplicate key.
///
/// # Errors
///
/// Returns `map entry line is missing =`, `duplicate map key`, or the key
/// or value's own parse error.
pub fn map_insert<K, V>(
    map: &mut FxHashMap<K, V>,
    line: &str,
    line_no: usize,
) -> Result<(), FolioError>
where
    K: FolioValue + Eq + Hash,
    V: FolioValue,
{
    let Some((key_text, value_text)) = line.split_once('=') else {
        return Err(FolioError::new(
            line_no,
            cstr!("map entry line is missing `=`"),
        ));
    };
    let key = K::parse_value(key_text, line_no)?;
    if map.contains_key(&key) {
        return Err(FolioError::new(
            line_no,
            cstr!("duplicate map key `{key_text}`"),
        ));
    }
    map.insert(key, V::parse_value(value_text, line_no)?);
    Ok(())
}
