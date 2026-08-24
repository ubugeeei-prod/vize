//! Parser for the croquis folio (`[vir]`) text format.
//!
//! Accepts canonical output byte-for-byte and is lenient exactly where the
//! printer normalizes: section order, blank-line placement, unsorted name
//! lists, and non-sequential scope ids. Everything else is strict - a line
//! that does not match its section's grammar is a [`FolioError`] with a
//! 1-based line number.
//!
//! The format is line-oriented, but source-derived fields can embed
//! newlines. `[macros]` entries therefore accumulate physical lines until
//! the ` @{start}:{end}` span tail appears, and the verbatim sections
//! (`[surface.emits]`, `[surface.models]`, `[surface.expose]`,
//! `[surface.slots]`, `[reactivity]`) keep every interior physical line -
//! only trailing blank lines are treated as the section separator.

use alloc::vec::Vec;

use vize_s0::{FxHashSet, String, cstr};

use super::{CroquisFolio, SurfaceEntry};
use crate::folio::FolioError;

mod entry;

use entry::{err, tail_is_span};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Section {
    Vir,
    Props,
    Emits,
    Models,
    Expose,
    Slots,
    Macros,
    Reactivity,
    Extern,
    Types,
    Bindings,
    Scopes,
    Errors,
}

impl Section {
    fn from_name(name: &str) -> Option<Self> {
        Some(match name {
            "vir" => Self::Vir,
            "surface.props" => Self::Props,
            "surface.emits" => Self::Emits,
            "surface.models" => Self::Models,
            "surface.expose" => Self::Expose,
            "surface.slots" => Self::Slots,
            "macros" => Self::Macros,
            "reactivity" => Self::Reactivity,
            "extern" => Self::Extern,
            "types" => Self::Types,
            "bindings" => Self::Bindings,
            "scopes" => Self::Scopes,
            "errors" => Self::Errors,
            _ => return None,
        })
    }

    const fn bit(self) -> u16 {
        1 << (self as u16)
    }

    /// Sections whose entries are kept as verbatim physical lines.
    const fn is_verbatim(self) -> bool {
        matches!(
            self,
            Self::Emits | Self::Models | Self::Expose | Self::Slots | Self::Reactivity
        )
    }
}

struct Parser {
    folio: CroquisFolio,
    current: Option<Section>,
    seen: u16,
    /// Bits: 1 = script_setup, 2 = scopes, 4 = bindings.
    header_seen: u8,
    /// Accumulated multi-line `[macros]` entry and its first line number.
    pending_macro: Option<(String, usize)>,
    /// Physical lines of the active verbatim section.
    verbatim: Vec<String>,
}

pub(super) fn parse(input: &str) -> Result<CroquisFolio, FolioError> {
    let mut parser = Parser {
        folio: CroquisFolio::default(),
        current: None,
        seen: 0,
        header_seen: 0,
        pending_macro: None,
        verbatim: Vec::new(),
    };

    for (idx, line) in input.split('\n').enumerate() {
        parser.line(line, idx + 1)?;
    }
    parser.finish()
}

impl Parser {
    fn line(&mut self, line: &str, line_no: usize) -> Result<(), FolioError> {
        if let Some(name) = line.strip_prefix('[').and_then(|r| r.strip_suffix(']'))
            && let Some(section) = Section::from_name(name)
        {
            return self.enter(section, name, line_no);
        }
        match self.current {
            None => {
                if line.is_empty() {
                    Ok(())
                } else {
                    Err(err(line_no, cstr!("content before the [vir] header")))
                }
            }
            Some(Section::Macros) => self.macro_line(line, line_no),
            Some(section) if section.is_verbatim() => {
                self.verbatim.push(String::from(line));
                Ok(())
            }
            Some(section) => {
                if line.is_empty() {
                    return Ok(());
                }
                self.entry_line(section, line, line_no)
            }
        }
    }

    fn enter(&mut self, section: Section, name: &str, line_no: usize) -> Result<(), FolioError> {
        self.flush()?;
        if self.seen == 0 && section != Section::Vir {
            return Err(err(line_no, cstr!("first section must be [vir]")));
        }
        if self.seen & section.bit() != 0 {
            return Err(err(line_no, cstr!("duplicate section [{name}]")));
        }
        self.seen |= section.bit();
        self.current = Some(section);
        Ok(())
    }

    fn macro_line(&mut self, line: &str, line_no: usize) -> Result<(), FolioError> {
        match self.pending_macro.take() {
            None => {
                if line.is_empty() {
                    return Ok(());
                }
                if tail_is_span(line) {
                    self.folio.macros.push(entry::parse_macro(line, line_no)?);
                } else {
                    self.pending_macro = Some((String::from(line), line_no));
                }
            }
            Some((mut joined, start_line)) => {
                joined.push('\n');
                joined.push_str(line);
                if tail_is_span(line) {
                    self.folio
                        .macros
                        .push(entry::parse_macro(joined.as_str(), start_line)?);
                } else {
                    self.pending_macro = Some((joined, start_line));
                }
            }
        }
        Ok(())
    }

    fn entry_line(&mut self, section: Section, line: &str, no: usize) -> Result<(), FolioError> {
        let folio = &mut self.folio;
        match section {
            Section::Vir => return entry::parse_header(line, no, folio, &mut self.header_seen),
            Section::Props => folio.props.push(entry::parse_prop(line, no)?),
            Section::Extern => folio.externs.push(entry::parse_extern(line, no)?),
            Section::Types => folio.types.push(entry::parse_type(line, no)?),
            Section::Bindings => folio.bindings.push(entry::parse_binding_group(line, no)?),
            Section::Scopes => folio.scopes.push(entry::parse_scope(line, no)?),
            Section::Errors => folio.errors.push(entry::parse_error(line, no)?),
            Section::Emits
            | Section::Models
            | Section::Expose
            | Section::Slots
            | Section::Reactivity
            | Section::Macros => unreachable!("handled by the driver"),
        }
        Ok(())
    }

    /// Close the active section: reject an unterminated `[macros]` entry
    /// and move the verbatim buffer (minus its trailing-blank separator)
    /// into the folio.
    fn flush(&mut self) -> Result<(), FolioError> {
        if let Some((_, start_line)) = self.pending_macro.take() {
            return Err(err(start_line, cstr!("macro line is missing a span")));
        }
        let Some(section) = self.current else {
            return Ok(());
        };
        if !section.is_verbatim() {
            return Ok(());
        }
        let mut lines = core::mem::take(&mut self.verbatim);
        while lines.last().is_some_and(|line| line.is_empty()) {
            lines.pop();
        }
        match section {
            Section::Emits => self.folio.emits = lines,
            Section::Models => self.folio.models = lines,
            Section::Reactivity => self.folio.reactivity = lines,
            Section::Expose | Section::Slots => {
                let entries: Vec<SurfaceEntry> = lines
                    .into_iter()
                    .map(|line| entry::parse_surface_entry(line.as_str()))
                    .collect();
                if section == Section::Expose {
                    self.folio.expose = entries;
                } else {
                    self.folio.slots = entries;
                }
            }
            _ => unreachable!("verbatim sections are matched above"),
        }
        Ok(())
    }

    fn finish(mut self) -> Result<CroquisFolio, FolioError> {
        self.flush()?;
        if self.seen & Section::Vir.bit() == 0 {
            return Err(err(0, cstr!("missing [vir] header")));
        }
        if self.header_seen != 0b111 {
            return Err(err(0, cstr!("incomplete [vir] header")));
        }
        validate_scopes(&self.folio)?;
        Ok(self.folio)
    }
}

/// Scope entry ids must be unique and every parent reference must resolve
/// to an entry.
fn validate_scopes(folio: &CroquisFolio) -> Result<(), FolioError> {
    let mut ids: FxHashSet<(char, u64)> = FxHashSet::default();
    for scope in &folio.scopes {
        if !ids.insert((scope.id.prefix, scope.id.index)) {
            return Err(err(
                0,
                cstr!("duplicate scope id {}{}", scope.id.prefix, scope.id.index),
            ));
        }
    }
    for scope in &folio.scopes {
        for parent in &scope.parents {
            if !ids.contains(&(parent.prefix, parent.index)) {
                return Err(err(
                    0,
                    cstr!(
                        "unresolved scope reference {}{}",
                        parent.prefix,
                        parent.index
                    ),
                ));
            }
        }
    }
    Ok(())
}
