//! The `[sfc-summary]` folio.
//!
//! Hand-written, because two decisions are semantic and the derive refuses
//! both: a schema line that is not the version this summary reads is
//! refused rather than stored, and the component signature is one
//! declaration printed as `name` / `params`. Empty facets are omitted.
//! `Display` prints the same bytes as `Full` — a summary has no spans.

use core::fmt;

use alloc::vec::Vec;

use vize_s0::{String, cstr};

use super::{Declaration, Facet, SfcSummary, finish, folio_error, insert, schema};
use crate::folio::page::{self, LineEvent, ParseState};
use crate::folio::value::FolioValue;
use crate::folio::{Folio, FolioError, FolioMode};

impl Folio for SfcSummary {
    fn print<W: fmt::Write>(&self, w: &mut W, _mode: FolioMode) -> fmt::Result {
        writeln!(w, "[sfc-summary]")?;
        writeln!(w, "schema_version={}", schema::SUMMARY)?;
        for facet in Facet::ALL {
            writeln!(w, "{}={}", facet.group(), facet.schema())?;
        }
        writeln!(w)?;
        for facet in Facet::ALL {
            print_facet(self, facet, w)?;
        }
        Ok(())
    }

    fn parse(input: &str) -> Result<Self, FolioError> {
        parse_summary(input)
    }
}

fn print_facet<W: fmt::Write>(summary: &SfcSummary, facet: Facet, w: &mut W) -> fmt::Result {
    let mut rows = summary
        .declarations
        .iter()
        .filter(|decl| decl.facet == facet);
    let Some(first) = rows.next() else {
        if facet == Facet::Signature {
            return Err(fmt::Error);
        }
        return Ok(());
    };
    writeln!(w, "[sfc-summary.{}]", facet.group())?;
    if facet == Facet::Signature {
        writeln!(w, "name={}", first.name)?;
        writeln!(w, "params={}", first.contract)?;
    } else {
        write_entry(w, first)?;
        for row in rows {
            write_entry(w, row)?;
        }
    }
    writeln!(w)
}

fn write_entry<W: fmt::Write>(w: &mut W, row: &Declaration) -> fmt::Result {
    writeln!(w, "{}={}", row.name, row.contract)
}

fn parse_summary(input: &str) -> Result<SfcSummary, FolioError> {
    let mut state = ParseState::new("sfc-summary");
    let mut seen_schema = 0u8;
    let mut signature_name: Option<String> = None;
    let mut signature_params: Option<String> = None;
    let mut declarations = Vec::new();
    let mut line_no = 0usize;
    for line in input.split('\n') {
        line_no += 1;
        match state.classify(line, line_no)? {
            LineEvent::Skip => {}
            LineEvent::Field => header_field(line, line_no, &mut seen_schema)?,
            LineEvent::Section(name) => {
                let Some(facet) = Facet::from_group_name(name) else {
                    return Err(state.unknown_section(name, line_no));
                };
                state.enter_section(usize::from(facet as u8), facet.group(), line_no)?;
            }
            LineEvent::Entry(index) => {
                let Some(facet) = Facet::ALL.get(index).copied() else {
                    return Err(FolioError::new(line_no, cstr!("unknown section index")));
                };
                if facet == Facet::Signature {
                    signature_field(line, line_no, &mut signature_name, &mut signature_params)?;
                } else {
                    let (name, contract) = page::split_field(line, line_no)?;
                    insert(
                        &mut declarations,
                        facet,
                        String::from(name),
                        String::from(contract),
                    )
                    .map_err(|err| folio_error(err, line_no))?;
                }
            }
        }
    }
    state.require_header()?;
    require_schemas(seen_schema)?;
    let name = page::require_scalar(signature_name, "name")?;
    let params = page::require_scalar(signature_params, "params")?;
    insert(&mut declarations, Facet::Signature, name, params).map_err(|err| folio_error(err, 0))?;
    Ok(finish(declarations))
}

fn header_field(line: &str, line_no: usize, seen: &mut u8) -> Result<(), FolioError> {
    let (name, value) = page::split_field(line, line_no)?;
    let (bit, expected) = if name == "schema_version" {
        (0u8, schema::SUMMARY)
    } else if let Some(facet) = Facet::from_group_name(name) {
        (facet as u8 + 1, facet.schema())
    } else {
        return Err(page::unknown_field(name, line_no));
    };
    let mask = 1u8 << bit;
    if *seen & mask != 0 {
        return Err(FolioError::new(line_no, cstr!("duplicate field `{name}`")));
    }
    let parsed = u16::parse_value(value, line_no)?;
    if parsed != expected {
        return Err(FolioError::new(
            line_no,
            cstr!("expected `{name}={expected}`, found `{line}`"),
        ));
    }
    *seen |= mask;
    Ok(())
}

fn require_schemas(seen: u8) -> Result<(), FolioError> {
    if seen & 1 == 0 {
        return Err(FolioError::new(0, cstr!("missing field `schema_version`")));
    }
    for facet in Facet::ALL {
        if seen & (1u8 << (facet as u8 + 1)) == 0 {
            return Err(FolioError::new(
                0,
                cstr!("missing field `{}`", facet.group()),
            ));
        }
    }
    Ok(())
}

fn signature_field(
    line: &str,
    line_no: usize,
    name_slot: &mut Option<String>,
    params_slot: &mut Option<String>,
) -> Result<(), FolioError> {
    let (key, value) = page::split_field(line, line_no)?;
    match key {
        "name" => {
            // Reject before storing, so a bad name never becomes a declaration.
            insert(
                &mut Vec::new(),
                Facet::Signature,
                String::from(value),
                String::from(""),
            )
            .map_err(|err| folio_error(err, line_no))?;
            page::set_scalar(name_slot, "name", value, line_no)?;
        }
        "params" => {
            if value.chars().any(|ch| ch == '\n' || ch == '\r') {
                return Err(folio_error(
                    super::SummaryError::BadContract {
                        facet: Facet::Signature,
                        name: String::from("params"),
                    },
                    line_no,
                ));
            }
            page::set_scalar(params_slot, "params", value, line_no)?;
        }
        _ => return Err(page::unknown_field(key, line_no)),
    }
    Ok(())
}
