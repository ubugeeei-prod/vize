//! Per-entry line parsers for the croquis folio sections.

use alloc::vec::Vec;

use vize_s0::{String, cstr};

use super::super::{
    BindingGroup, CroquisFolio, ErrorEntry, ExternEntry, MacroEntry, PropEntry, ScopeEntry,
    ScopeRef, SurfaceEntry, TypeEntry, TypeExportMark,
};
use crate::folio::FolioError;

pub(super) fn err(line: usize, message: String) -> FolioError {
    FolioError::new(line, message)
}

fn parse_u32(s: &str) -> Option<u32> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

fn parse_u64(s: &str) -> Option<u64> {
    if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    s.parse().ok()
}

/// `{start}:{end}` with both halves decimal.
fn parse_span(s: &str) -> Option<(u32, u32)> {
    let (start, end) = s.split_once(':')?;
    Some((parse_u32(start)?, parse_u32(end)?))
}

/// Does this physical line end with a ` @{start}:{end}` span tail?
///
/// Used by the `[macros]` accumulator to find the end of a (possibly
/// multi-line) entry.
pub(super) fn tail_is_span(line: &str) -> bool {
    line.rfind(" @")
        .is_some_and(|pos| parse_span(&line[pos + 2..]).is_some())
}

/// `{prefix}{index}` where prefix is `~`, `!`, or `#`.
fn parse_ref(s: &str) -> Option<ScopeRef> {
    let mut chars = s.chars();
    let prefix = chars.next()?;
    if !matches!(prefix, '~' | '!' | '#') {
        return None;
    }
    let index = parse_u64(chars.as_str())?;
    Some(ScopeRef { prefix, index })
}

/// Comma-separated names with no surrounding whitespace; every name
/// non-empty.
fn parse_name_list(s: &str, line: usize) -> Result<Vec<String>, FolioError> {
    s.split(',')
        .map(|name| {
            if name.is_empty() {
                Err(err(line, cstr!("empty name in list")))
            } else {
                Ok(String::from(name))
            }
        })
        .collect()
}

/// `script_setup=` / `scopes=` / `bindings=` inside `[vir]`.
pub(super) fn parse_header(
    line: &str,
    line_no: usize,
    folio: &mut CroquisFolio,
    header_seen: &mut u8,
) -> Result<(), FolioError> {
    let (key, value) = line
        .split_once('=')
        .ok_or_else(|| err(line_no, cstr!("expected key=value in [vir]")))?;
    let bit = match key {
        "script_setup" => {
            folio.script_setup = match value {
                "true" => true,
                "false" => false,
                _ => return Err(err(line_no, cstr!("script_setup must be true or false"))),
            };
            1
        }
        "scopes" => {
            folio.scope_count =
                parse_u64(value).ok_or_else(|| err(line_no, cstr!("scopes must be an integer")))?;
            2
        }
        "bindings" => {
            folio.binding_count = parse_u64(value)
                .ok_or_else(|| err(line_no, cstr!("bindings must be an integer")))?;
            4
        }
        _ => return Err(err(line_no, cstr!("unknown [vir] key {key}"))),
    };
    if *header_seen & bit != 0 {
        return Err(err(line_no, cstr!("duplicate [vir] key {key}")));
    }
    *header_seen |= bit;
    Ok(())
}

/// `{name}{!|?}[:{type}][=]`
pub(super) fn parse_prop(line: &str, line_no: usize) -> Result<PropEntry, FolioError> {
    let (pos, req) = line
        .char_indices()
        .find(|(_, c)| matches!(c, '!' | '?'))
        .ok_or_else(|| err(line_no, cstr!("prop line is missing a !/? marker")))?;
    if pos == 0 {
        return Err(err(line_no, cstr!("prop line has an empty name")));
    }
    let name = String::from(&line[..pos]);
    let rest = &line[pos + req.len_utf8()..];
    let (ty, has_default) = if rest.is_empty() {
        (None, false)
    } else if rest == "=" {
        (None, true)
    } else if let Some(ty) = rest.strip_prefix(':') {
        match ty.strip_suffix('=') {
            Some(ty) => (Some(String::from(ty)), true),
            None => (Some(String::from(ty)), false),
        }
    } else {
        return Err(err(line_no, cstr!("malformed prop line")));
    };
    Ok(PropEntry {
        name,
        required: req == '!',
        ty,
        has_default,
    })
}

/// A `@{start}:{end}` line becomes a span fallback; anything else is
/// verbatim argument text.
pub(super) fn parse_surface_entry(line: &str) -> SurfaceEntry {
    line.strip_prefix('@').and_then(parse_span).map_or_else(
        || SurfaceEntry::Args(String::from(line)),
        |(start, end)| SurfaceEntry::Span { start, end },
    )
}

/// `@{name}[<{type_args}>] @{start}:{end}` - `text` may span physical
/// lines; the driver guarantees it ends with the span tail.
pub(super) fn parse_macro(text: &str, line_no: usize) -> Result<MacroEntry, FolioError> {
    let body = text
        .strip_prefix('@')
        .ok_or_else(|| err(line_no, cstr!("macro line must start with @")))?;
    let pos = body
        .rfind(" @")
        .ok_or_else(|| err(line_no, cstr!("macro line is missing a span")))?;
    let (start, end) =
        parse_span(&body[pos + 2..]).ok_or_else(|| err(line_no, cstr!("malformed macro span")))?;
    let head = &body[..pos];
    let (name, type_args) = match head.find('<') {
        Some(lt) => {
            let ty = head[lt..]
                .strip_prefix('<')
                .and_then(|t| t.strip_suffix('>'))
                .ok_or_else(|| err(line_no, cstr!("unterminated macro type arguments")))?;
            (&head[..lt], Some(String::from(ty)))
        }
        None => (head, None),
    };
    if name.is_empty() {
        return Err(err(line_no, cstr!("macro line has an empty name")));
    }
    Ok(MacroEntry {
        name: String::from(name),
        type_args,
        start,
        end,
    })
}

/// `{source}[^][ {{a,b}}]`
pub(super) fn parse_extern(line: &str, line_no: usize) -> Result<ExternEntry, FolioError> {
    let (head, bindings) = match line.strip_suffix('}') {
        Some(rest) => {
            let pos = rest
                .rfind(" {")
                .ok_or_else(|| err(line_no, cstr!("malformed extern binding list")))?;
            (&rest[..pos], parse_name_list(&rest[pos + 2..], line_no)?)
        }
        None => (line, Vec::new()),
    };
    let (source, type_only) = match head.strip_suffix('^') {
        Some(source) => (source, true),
        None => (head, false),
    };
    if source.is_empty() {
        return Err(err(line_no, cstr!("extern line has an empty source")));
    }
    Ok(ExternEntry {
        source: String::from(source),
        type_only,
        bindings,
    })
}

/// `{name}[^]{t|i}@{start}:{end}`
pub(super) fn parse_type(line: &str, line_no: usize) -> Result<TypeEntry, FolioError> {
    let pos = line
        .rfind('@')
        .ok_or_else(|| err(line_no, cstr!("type line is missing a span")))?;
    let (start, end) =
        parse_span(&line[pos + 1..]).ok_or_else(|| err(line_no, cstr!("malformed type span")))?;
    let head = &line[..pos];
    let (head, kind) = match head.strip_suffix('t') {
        Some(head) => (head, TypeExportMark::Type),
        None => match head.strip_suffix('i') {
            Some(head) => (head, TypeExportMark::Interface),
            None => return Err(err(line_no, cstr!("type line is missing a t/i kind mark"))),
        },
    };
    let (name, hoisted) = match head.strip_suffix('^') {
        Some(name) => (name, true),
        None => (head, false),
    };
    if name.is_empty() {
        return Err(err(line_no, cstr!("type line has an empty name")));
    }
    Ok(TypeEntry {
        name: String::from(name),
        hoisted,
        kind,
        start,
        end,
    })
}

/// `{code}:{name,name,...}`
pub(super) fn parse_binding_group(line: &str, line_no: usize) -> Result<BindingGroup, FolioError> {
    let (code, names) = line
        .split_once(':')
        .ok_or_else(|| err(line_no, cstr!("binding group is missing a : separator")))?;
    if code.is_empty() {
        return Err(err(line_no, cstr!("binding group has an empty code")));
    }
    Ok(BindingGroup {
        code: String::from(code),
        names: parse_name_list(names, line_no)?,
    })
}

/// `{id} {name} @{start}:{end}[ [a,b]][ < p, q]`
pub(super) fn parse_scope(line: &str, line_no: usize) -> Result<ScopeEntry, FolioError> {
    let mut rest = line;

    let mut parents = Vec::new();
    if let Some(pos) = rest.rfind(" < ") {
        for r in rest[pos + 3..].split(", ") {
            let r = parse_ref(r)
                .ok_or_else(|| err(line_no, cstr!("malformed scope parent reference")))?;
            parents.push(r);
        }
        rest = &rest[..pos];
    }

    let mut bindings = Vec::new();
    if let Some(inner) = rest.strip_suffix(']') {
        let pos = inner
            .rfind(" [")
            .ok_or_else(|| err(line_no, cstr!("malformed scope binding list")))?;
        bindings = parse_name_list(&inner[pos + 2..], line_no)?;
        rest = &inner[..pos];
    }

    let pos = rest
        .rfind(" @")
        .ok_or_else(|| err(line_no, cstr!("scope line is missing a span")))?;
    let (start, end) =
        parse_span(&rest[pos + 2..]).ok_or_else(|| err(line_no, cstr!("malformed scope span")))?;
    rest = &rest[..pos];

    let (id, name) = rest
        .split_once(' ')
        .ok_or_else(|| err(line_no, cstr!("scope line is missing a name")))?;
    let id = parse_ref(id).ok_or_else(|| err(line_no, cstr!("malformed scope id")))?;
    if name.is_empty() {
        return Err(err(line_no, cstr!("scope line has an empty name")));
    }
    Ok(ScopeEntry {
        id,
        name: String::from(name),
        start,
        end,
        bindings,
        parents,
    })
}

/// `{name}={kind}@{start}:{end}`
pub(super) fn parse_error(line: &str, line_no: usize) -> Result<ErrorEntry, FolioError> {
    let (name, rest) = line
        .split_once('=')
        .ok_or_else(|| err(line_no, cstr!("error line is missing a = separator")))?;
    if name.is_empty() {
        return Err(err(line_no, cstr!("error line has an empty name")));
    }
    let pos = rest
        .rfind('@')
        .ok_or_else(|| err(line_no, cstr!("error line is missing a span")))?;
    let (start, end) =
        parse_span(&rest[pos + 1..]).ok_or_else(|| err(line_no, cstr!("malformed error span")))?;
    let kind = &rest[..pos];
    if kind.is_empty() {
        return Err(err(line_no, cstr!("error line has an empty kind")));
    }
    Ok(ErrorEntry {
        name: String::from(name),
        kind: String::from(kind),
        start,
        end,
    })
}
