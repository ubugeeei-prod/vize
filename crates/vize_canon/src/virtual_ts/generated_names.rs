//! Names the generator writes where the SFC's type parameters are bound.
//!
//! `generic="Props"` binds `Props` in `__setup` and in every generic signature
//! the module declares, so a generated reference spelled `Props` would resolve
//! to the parameter instead (#11485). The generator therefore never spells a
//! name an authored parameter binds:
//!
//! - a module declaration it owns takes the first `{name}_{n}` no parameter
//!   binds, and public names are re-exported from it (an authored type of the
//!   same name is the SFC's own declaration and keeps its spelling);
//! - a lib type is reached through `globalThis`, which no parameter can bind;
//! - a shared helper is reached through a module alias with a fresh name.
//!
//! Without such a parameter every name is spelled as before.

use core::fmt;
use std::cell::RefCell;

use vize_carton::{String, append, cstr};
use vize_l0::ts_emit::{EmitHole, EmitLen, TsBuf};

use super::generic_parameters::GenericParameters;
use vize_croquis::Croquis;

#[derive(Default)]
struct Projection {
    bound: Vec<String>,
    authored: Vec<String>,
    /// Helper forwarding aliases `(fresh, helper, constraints)` emitted once.
    forwarded: Vec<(String, &'static str, &'static [&'static str])>,
}

thread_local! {
    /// Projections are synchronous per thread, like the bindings table.
    static CURRENT: RefCell<Projection> = RefCell::new(Projection::default());
}

/// Restores an outer projection's names on return and unwinding.
pub(crate) struct Binding(Option<Projection>);

impl Drop for Binding {
    fn drop(&mut self) {
        if let Some(outer) = self.0.take() {
            CURRENT.with_borrow_mut(|current| *current = outer);
        }
    }
}

/// Bind the authored type parameter names for one projection, beside the
/// module types the SFC declares itself.
pub(crate) fn bind(parameters: Option<&GenericParameters>, summary: &Croquis) -> Binding {
    let authored = summary.type_exports.iter();
    bind_names(parameters, authored.map(|export| export.name.as_str()))
}

fn bind_names<'a>(
    parameters: Option<&GenericParameters>,
    authored_types: impl IntoIterator<Item = &'a str>,
) -> Binding {
    let bound: Vec<String> = parameters
        .into_iter()
        .flat_map(GenericParameters::parameter_names)
        .map(String::from)
        .collect();
    let authored = if bound.is_empty() {
        Vec::new()
    } else {
        authored_types.into_iter().map(String::from).collect()
    };
    let projection = Projection {
        bound,
        authored,
        forwarded: Vec::new(),
    };
    Binding(Some(CURRENT.with_borrow_mut(|current| {
        std::mem::replace(current, projection)
    })))
}

fn fresh(projection: &Projection, base: &str) -> Option<String> {
    let bound = |name: &str| projection.bound.iter().any(|bound| bound == name);
    let authored = |name: &str| projection.authored.iter().any(|authored| authored == name);
    if !bound(base) || authored(base) {
        return None;
    }
    (1usize..)
        .map(|suffix| cstr!("{base}_{suffix}"))
        .find(|candidate| !bound(candidate) && !authored(candidate))
}

/// A module declaration the generator owns: its declaration and every
/// reference use this spelling.
pub(crate) fn declared(base: &'static str) -> String {
    CURRENT
        .with_borrow(|projection| fresh(projection, base))
        .unwrap_or(String::const_new(base))
}

/// [`declared`] for a computed name.
pub(crate) fn declared_dynamic(base: &str) -> String {
    CURRENT
        .with_borrow(|projection| fresh(projection, base))
        .unwrap_or_else(|| String::from(base))
}

/// Head of an exported generated declaration: `export type Props` or, when a
/// parameter binds the public name, `type Props_1`. Close the declaration
/// with [`reexport`].
pub(crate) fn export_type(public: &'static str) -> ExportHead {
    let local = declared(public);
    ExportHead {
        exported: local == public,
        local,
    }
}

pub(crate) struct ExportHead {
    exported: bool,
    local: String,
}

/// Head of a module-local generated declaration: `type $ResolvedProps`.
pub(crate) fn local_type(base: &'static str) -> ExportHead {
    ExportHead {
        exported: false,
        local: declared(base),
    }
}

impl ExportHead {
    fn keyword(&self) -> &'static str {
        if self.exported {
            "export type "
        } else {
            "type "
        }
    }
}

impl fmt::Display for ExportHead {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.keyword())?;
        formatter.write_str(&self.local)
    }
}

impl EmitLen for ExportHead {
    fn hole_len(&self) -> usize {
        self.keyword().len() + self.local.len()
    }
}

impl<B: TsBuf> EmitHole<B> for ExportHead {
    fn emit_hole(&self, buffer: &mut B) {
        buffer.push_ts_str(self.keyword());
        buffer.push_ts_str(&self.local);
    }
}

/// Re-export a renamed public declaration under its public name.
pub(crate) fn reexport(ts: &mut crate::virtual_ts::TextBuf, public: &'static str) {
    let local = declared(public);
    if local != public {
        append!(*ts, "export type {{ {local} as {public} }};\n");
    }
}

/// A lib type, global in every program.
pub(crate) fn lib(name: &'static str) -> String {
    if CURRENT.with_borrow(|projection| projection.bound.iter().any(|bound| bound == name)) {
        cstr!("globalThis.{name}")
    } else {
        String::const_new(name)
    }
}

/// The awaited return of `__setup` instantiated with `arguments` (`<T>` or
/// empty): every module alias derived from setup reads it.
pub(crate) fn setup_return(arguments: &str) -> SetupReturn<'_> {
    SetupReturn {
        awaited: lib("Awaited"),
        return_type: lib("ReturnType"),
        arguments,
    }
}

pub(crate) struct SetupReturn<'a> {
    awaited: String,
    return_type: String,
    arguments: &'a str,
}

impl SetupReturn<'_> {
    fn pieces(&self) -> [&str; 6] {
        [
            &self.awaited,
            "<",
            &self.return_type,
            "<typeof __setup",
            self.arguments,
            ">>",
        ]
    }
}

impl fmt::Display for SetupReturn<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.pieces()
            .iter()
            .try_for_each(|piece| formatter.write_str(piece))
    }
}

impl EmitLen for SetupReturn<'_> {
    fn hole_len(&self) -> usize {
        self.pieces().iter().map(|piece| piece.len()).sum()
    }
}

impl<B: TsBuf> EmitHole<B> for SetupReturn<'_> {
    fn emit_hole(&self, buffer: &mut B) {
        for piece in self.pieces() {
            buffer.push_ts_str(piece);
        }
    }
}

/// A shared helper type; `constraints` spell its parameters' constraints (``
/// for none). Helpers are global in batch projections and module-local in
/// standalone ones, so a bound name is forwarded through a module alias
/// declared by [`emit_forwarded_helpers`].
pub(crate) fn helper(name: &'static str, constraints: &'static [&'static str]) -> String {
    CURRENT.with_borrow_mut(|projection| {
        let Some(local) = fresh(projection, name) else {
            return String::const_new(name);
        };
        if !projection
            .forwarded
            .iter()
            .any(|(forwarded, _, _)| *forwarded == local)
        {
            projection
                .forwarded
                .push((local.clone(), name, constraints));
        }
        local
    })
}

/// The line prefix of a template use projected onto its setup binding.
pub(crate) const TEMPLATE_USE_PREFIX: &str = "/* __vize_template_use */ typeof ";

/// The line prefix of a template use's unwrapped, const-preserving hover proxy.
pub(crate) const TEMPLATE_HOVER_PREFIX: &str = "/* __vize_template_hover */ const ";

/// Declare the helper aliases requested so far, outside every generic scope.
pub(crate) fn emit_forwarded_helpers(ts: &mut crate::virtual_ts::TextBuf) {
    CURRENT.with_borrow(|projection| {
        for (local, helper, constraints) in &projection.forwarded {
            if constraints.is_empty() {
                append!(*ts, "type {local} = {helper};\n");
                continue;
            }
            let (mut parameters, mut arguments) = (String::default(), String::default());
            for (index, constraint) in constraints.iter().enumerate() {
                let separator = if index > 0 { ", " } else { "" };
                append!(arguments, "{separator}__A{index}");
                append!(parameters, "{separator}__A{index}");
                if !constraint.is_empty() {
                    append!(parameters, " extends {constraint}");
                }
            }
            append!(*ts, "type {local}<{parameters}> = {helper}<{arguments}>;\n");
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unbound_names_keep_their_spelling() {
        let parameters = GenericParameters::parse("T extends string");
        let _binding = bind_names(Some(&parameters), []);
        assert_eq!(declared("Props"), "Props");
        assert_eq!(cstr!("{}", export_type("Props")), "export type Props");
        assert_eq!(
            cstr!("{}", setup_return("<T>")),
            "Awaited<ReturnType<typeof __setup<T>>>"
        );
        assert_eq!(lib("Record"), "Record");
        assert_eq!(helper("$PublicEmit", &[""]), "$PublicEmit");
        let mut ts = crate::virtual_ts::TextBuf::default();
        reexport(&mut ts, "Props");
        emit_forwarded_helpers(&mut ts);
        assert_eq!(ts, "");
    }

    #[test]
    fn bound_names_take_fresh_spellings() {
        let parameters =
            GenericParameters::parse("Props, Props_1, Record, $PublicEmit, $NativeElement");
        let _binding = bind_names(Some(&parameters), ["Props_3"]);
        assert_eq!(declared("Props"), "Props_2");
        assert_eq!(cstr!("{}", export_type("Props")), "type Props_2");
        assert_eq!(lib("Record"), "globalThis.Record");
        assert_eq!(helper("$PublicEmit", &[""]), "$PublicEmit_1");
        assert_eq!(
            helper("$NativeElement", &["PropertyKey"]),
            "$NativeElement_1"
        );
        let mut ts = crate::virtual_ts::TextBuf::default();
        reexport(&mut ts, "Props");
        emit_forwarded_helpers(&mut ts);
        assert_eq!(
            ts,
            "export type { Props_2 as Props };\ntype $PublicEmit_1<__A0> = $PublicEmit<__A0>;\ntype $NativeElement_1<__A0 extends PropertyKey> = $NativeElement<__A0>;\n"
        );
    }

    #[test]
    fn bindings_restore_the_outer_projection() {
        let outer = GenericParameters::parse("Emits");
        let _outer = bind_names(Some(&outer), []);
        {
            let _inner = bind_names(None, []);
            assert_eq!(declared("Emits"), "Emits");
        }
        assert_eq!(declared("Emits"), "Emits_1");
    }

    #[test]
    fn authored_types_keep_their_spelling() {
        // `Emits` is the SFC's own declaration; no generated alias exists to rename.
        let parameters = GenericParameters::parse("Emits, Props");
        let _binding = bind_names(Some(&parameters), ["Emits", "Props_1"]);
        assert_eq!(declared("Emits"), "Emits");
        assert_eq!(declared("Props"), "Props_2");
    }
}
