//! The emit-document page (`emit-document-page@1`): generated text plus
//! the generated↔authored links of a P3-9 `EmitDocument`.
//!
//! ```text
//! [emit-document]
//! schema_version=1
//!
//! [emit-document.links]
//! <gen-start>:<gen-end> <src-start>:<src-end> <name> <true|false>
//!
//! [emit-document.text]
//! <the generated text, verbatim to the end of the page>
//! ```
//!
//! The text section is terminal, so any generated text round-trips byte
//! for byte. `<name>` is `-` when the link has no authored symbol.

use core::fmt::{Result as FmtResult, Write};

use vize_davinci::folio::{Folio, FolioError, FolioMode};
use vize_s0::{String, cstr};

use crate::contract::Span;

/// One generated↔authored link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmitLink {
    pub generated: Span,
    pub authored: Span,
    pub name: Option<String>,
    pub segment: bool,
}

/// Generated text plus its links, in producer order.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct EmitDocument {
    pub text: String,
    pub links: Vec<EmitLink>,
}

impl Folio for EmitDocument {
    fn print<W: Write>(&self, w: &mut W, _mode: FolioMode) -> FmtResult {
        writeln!(
            w,
            "[emit-document]\nschema_version=1\n\n[emit-document.links]"
        )?;
        for link in &self.links {
            let name = link.name.as_deref().unwrap_or("-");
            let segment = if link.segment { "true" } else { "false" };
            writeln!(
                w,
                "{}:{} {}:{} {name} {segment}",
                link.generated.start, link.generated.end, link.authored.start, link.authored.end
            )?;
        }
        write!(w, "\n[emit-document.text]\n{}", self.text)
    }

    fn parse(input: &str) -> Result<Self, FolioError> {
        let err = |line: usize, message: String| FolioError::new(line, message);
        let Some((head, text)) = input.split_once("[emit-document.text]\n") else {
            return Err(err(0, cstr!("missing section [emit-document.text]")));
        };
        let mut document = EmitDocument {
            text: String::from(text),
            links: Vec::new(),
        };
        let mut section = "";
        for (index, line) in head.split('\n').enumerate() {
            let no = index + 1;
            match (section, line) {
                (_, "") => {}
                ("", "[emit-document]") => section = "header",
                ("", _) => return Err(err(no, cstr!("first section must be [emit-document]"))),
                ("header", "[emit-document.links]") => section = "links",
                ("header", "schema_version=1") => {}
                ("header", _) => {
                    return Err(err(
                        no,
                        cstr!("expected `schema_version=1`, found `{line}`"),
                    ));
                }
                ("links", _) => document.links.push(link(line, no)?),
                _ => return Err(err(no, cstr!("unexpected line `{line}`"))),
            }
        }
        if section != "links" {
            return Err(err(0, cstr!("missing section [emit-document.links]")));
        }
        Ok(document)
    }
}

fn link(line: &str, no: usize) -> Result<EmitLink, FolioError> {
    let bad = || FolioError::new(no, cstr!("invalid link `{line}`"));
    let mut parts = line.split(' ');
    let generated = span(parts.next().ok_or_else(bad)?, no)?;
    let authored = span(parts.next().ok_or_else(bad)?, no)?;
    let name = parts.next().ok_or_else(bad)?;
    let segment = match parts.next().ok_or_else(bad)? {
        "true" => true,
        "false" => false,
        _ => return Err(bad()),
    };
    if parts.next().is_some() || name.is_empty() {
        return Err(bad());
    }
    Ok(EmitLink {
        generated,
        authored,
        name: (name != "-").then(|| String::from(name)),
        segment,
    })
}

fn span(text: &str, no: usize) -> Result<Span, FolioError> {
    let bad = || FolioError::new(no, cstr!("invalid range `{text}`"));
    let (start, end) = text.split_once(':').ok_or_else(bad)?;
    let start = start.parse().map_err(|_| bad())?;
    let end = end.parse().map_err(|_| bad())?;
    if start > end {
        return Err(bad());
    }
    Ok(Span { start, end })
}
