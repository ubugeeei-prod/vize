//! Canonical printer for [`CroquisFolio`].
//!
//! `Full` mode reproduces `Croquis::to_vir()` byte-for-byte for canonical
//! content; `Display` mode elides spans and default markers. Both modes
//! normalize on the fly (sorted map-derived lists, sequential scope ids),
//! so printing never mutates the value.

use core::fmt::{Result, Write};

use alloc::vec::Vec;

use vize_s0::FxHashMap;

use super::{CroquisFolio, ScopeEntry, ScopeRef, SurfaceEntry};
use crate::folio::FolioMode;

/// Compute the sequential-per-prefix renumbering for scope entries, keyed
/// by `(prefix, old index)`.
pub(super) fn scope_id_remap(scopes: &[ScopeEntry]) -> FxHashMap<(char, u64), u64> {
    let mut counters: FxHashMap<char, u64> = FxHashMap::default();
    let mut remap = FxHashMap::default();
    for scope in scopes {
        let counter = counters.entry(scope.id.prefix).or_insert(0);
        remap.insert((scope.id.prefix, scope.id.index), *counter);
        *counter += 1;
    }
    remap
}

/// Apply the renumbering to one reference; unresolved references (only
/// reachable from hand-built values) are kept verbatim.
pub(super) fn remap_ref(remap: &FxHashMap<(char, u64), u64>, r: ScopeRef) -> ScopeRef {
    match remap.get(&(r.prefix, r.index)) {
        Some(&index) => ScopeRef {
            prefix: r.prefix,
            index,
        },
        None => r,
    }
}

fn sorted(names: &[vize_s0::String]) -> Vec<&str> {
    let mut out: Vec<&str> = names.iter().map(|n| n.as_str()).collect();
    out.sort_unstable();
    out
}

fn write_joined<W: Write>(w: &mut W, items: &[&str], sep: &str) -> Result {
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            w.write_str(sep)?;
        }
        w.write_str(item)?;
    }
    Ok(())
}

pub(super) fn print<W: Write>(folio: &CroquisFolio, w: &mut W, mode: FolioMode) -> Result {
    let full = mode == FolioMode::Full;

    writeln!(w, "[vir]")?;
    writeln!(w, "script_setup={}", folio.script_setup)?;
    writeln!(w, "scopes={}", folio.scope_count)?;
    writeln!(w, "bindings={}", folio.binding_count)?;
    writeln!(w)?;

    if !folio.props.is_empty() {
        writeln!(w, "[surface.props]")?;
        for prop in &folio.props {
            let req = if prop.required { '!' } else { '?' };
            write!(w, "{}{}", prop.name, req)?;
            if let Some(ty) = &prop.ty {
                write!(w, ":{ty}")?;
            }
            if full && prop.has_default {
                w.write_char('=')?;
            }
            writeln!(w)?;
        }
        writeln!(w)?;
    }

    for (header, lines) in [
        ("[surface.emits]", &folio.emits),
        ("[surface.models]", &folio.models),
    ] {
        if !lines.is_empty() {
            writeln!(w, "{header}")?;
            for line in lines {
                writeln!(w, "{line}")?;
            }
            writeln!(w)?;
        }
    }

    for (header, entries) in [
        ("[surface.expose]", &folio.expose),
        ("[surface.slots]", &folio.slots),
    ] {
        let printable = |entry: &&SurfaceEntry| full || matches!(entry, SurfaceEntry::Args(_));
        if entries.iter().any(|entry| printable(&entry)) {
            writeln!(w, "{header}")?;
            for entry in entries.iter().filter(printable) {
                match entry {
                    SurfaceEntry::Args(args) => writeln!(w, "{args}")?,
                    SurfaceEntry::Span { start, end } => writeln!(w, "@{start}:{end}")?,
                }
            }
            writeln!(w)?;
        }
    }

    if !folio.macros.is_empty() {
        writeln!(w, "[macros]")?;
        for call in &folio.macros {
            write!(w, "@{}", call.name)?;
            if let Some(ty) = &call.type_args {
                write!(w, "<{ty}>")?;
            }
            if full {
                write!(w, " @{}:{}", call.start, call.end)?;
            }
            writeln!(w)?;
        }
        writeln!(w)?;
    }

    if !folio.reactivity.is_empty() {
        writeln!(w, "[reactivity]")?;
        for line in &folio.reactivity {
            writeln!(w, "{line}")?;
        }
        writeln!(w)?;
    }

    if !folio.externs.is_empty() {
        writeln!(w, "[extern]")?;
        for ext in &folio.externs {
            let type_only = if ext.type_only { "^" } else { "" };
            write!(w, "{}{}", ext.source, type_only)?;
            if !ext.bindings.is_empty() {
                w.write_str(" {")?;
                write_joined(w, &sorted(&ext.bindings), ",")?;
                w.write_char('}')?;
            }
            writeln!(w)?;
        }
        writeln!(w)?;
    }

    if !folio.types.is_empty() {
        writeln!(w, "[types]")?;
        for te in &folio.types {
            let hoist = if te.hoisted { "^" } else { "" };
            write!(w, "{}{}{}", te.name, hoist, te.kind.as_char())?;
            if full {
                write!(w, "@{}:{}", te.start, te.end)?;
            }
            writeln!(w)?;
        }
        writeln!(w)?;
    }

    if !folio.bindings.is_empty() {
        writeln!(w, "[bindings]")?;
        for group in &folio.bindings {
            write!(w, "{}:", group.code)?;
            write_joined(w, &sorted(&group.names), ",")?;
            writeln!(w)?;
        }
        writeln!(w)?;
    }

    if !folio.scopes.is_empty() {
        let remap = scope_id_remap(&folio.scopes);
        writeln!(w, "[scopes]")?;
        for scope in &folio.scopes {
            let id = remap_ref(&remap, scope.id);
            write!(w, "{}{} {}", id.prefix, id.index, scope.name)?;
            if full {
                write!(w, " @{}:{}", scope.start, scope.end)?;
            }
            if !scope.bindings.is_empty() {
                w.write_str(" [")?;
                write_joined(w, &sorted(&scope.bindings), ",")?;
                w.write_char(']')?;
            }
            for (i, parent) in scope.parents.iter().enumerate() {
                let parent = remap_ref(&remap, *parent);
                let sep = if i == 0 { " < " } else { ", " };
                write!(w, "{}{}{}", sep, parent.prefix, parent.index)?;
            }
            writeln!(w)?;
        }
        writeln!(w)?;
    }

    if !folio.errors.is_empty() {
        writeln!(w, "[errors]")?;
        for err in &folio.errors {
            write!(w, "{}={}", err.name, err.kind)?;
            if full {
                write!(w, "@{}:{}", err.start, err.end)?;
            }
            writeln!(w)?;
        }
        writeln!(w)?;
    }

    Ok(())
}
