//! VIR (Vize Intermediate Representation) text format output.
//!
//! Generates a TOML-like human-readable representation of a croquis
//! for debugging and inspection purposes.

use super::{Croquis, TypeExportKind};
use std::fmt::Write;
use vize_carton::{FxHashMap, String};
use vize_relief::BindingType;

mod scopes;
mod surface;

impl Croquis {
    /// Convert croquis to VIR (Vize Intermediate Representation) text format.
    ///
    /// This generates a TOML-like human-readable representation of the croquis.
    ///
    /// # Important
    ///
    /// **VIR is a display format only, not a portable representation.**
    ///
    /// - VIR output is intended for debugging and human inspection
    /// - The format may change between versions without notice
    /// - Do not parse VIR output or use it as a stable interface
    /// - For programmatic access, use the `Croquis` struct fields directly
    ///
    /// Performance: Pre-allocates buffer, uses write! macro for zero-copy formatting.
    pub fn to_vir(&self) -> String {
        // Pre-allocate with estimated capacity
        let mut output = String::with_capacity(4096);

        // [vir]
        writeln!(output, "[vir]").ok();
        writeln!(output, "script_setup={}", self.bindings.is_script_setup).ok();
        writeln!(output, "scopes={}", self.scopes.len()).ok();
        writeln!(output, "bindings={}", self.bindings.bindings.len()).ok();
        writeln!(output).ok();

        self.write_surface(&mut output);
        self.write_macros(&mut output);
        self.write_reactivity(&mut output);
        self.write_extern(&mut output);
        self.write_types(&mut output);
        self.write_bindings(&mut output);
        self.write_scopes(&mut output);
        self.write_errors(&mut output);

        output
    }

    fn write_macros(&self, output: &mut String) {
        if self.macros.all_calls().is_empty() {
            return;
        }

        writeln!(output, "[macros]").ok();
        for call in self.macros.all_calls() {
            if let Some(ref ty) = call.type_args {
                writeln!(
                    output,
                    "@{}<{}> @{}:{}",
                    call.name, ty, call.start, call.end
                )
                .ok();
            } else {
                writeln!(output, "@{} @{}:{}", call.name, call.start, call.end).ok();
            }
        }
        writeln!(output).ok();
    }

    fn write_reactivity(&self, output: &mut String) {
        if self.reactivity.count() == 0 {
            return;
        }

        writeln!(output, "[reactivity]").ok();
        for src in self.reactivity.sources() {
            writeln!(output, "{}={}", src.name, src.kind.to_display()).ok();
        }
        writeln!(output).ok();
    }

    fn write_extern(&self, output: &mut String) {
        let extern_scopes: Vec<_> = self
            .scopes
            .iter()
            .filter(|s| s.kind == crate::scope::ScopeKind::ExternalModule)
            .collect();

        if extern_scopes.is_empty() {
            return;
        }

        writeln!(output, "[extern]").ok();
        for scope in &extern_scopes {
            if let crate::scope::ScopeData::ExternalModule(data) = scope.data() {
                let type_only = if data.is_type_only { "^" } else { "" };
                let bd: Vec<_> = scope.bindings().map(|(n, _)| n).collect();
                if bd.is_empty() {
                    writeln!(output, "{}{}", data.source, type_only).ok();
                } else {
                    writeln!(output, "{}{} {{{}}}", data.source, type_only, bd.join(",")).ok();
                }
            }
        }
        writeln!(output).ok();
    }

    fn write_types(&self, output: &mut String) {
        if self.type_exports.is_empty() {
            return;
        }

        writeln!(output, "[types]").ok();
        for te in &self.type_exports {
            let hoist = if te.hoisted { "^" } else { "" };
            let kind = match te.kind {
                TypeExportKind::Type => "t",
                TypeExportKind::Interface => "i",
            };
            writeln!(
                output,
                "{}{}{}@{}:{}",
                te.name, hoist, kind, te.start, te.end
            )
            .ok();
        }
        writeln!(output).ok();
    }

    fn write_bindings(&self, output: &mut String) {
        if self.bindings.bindings.is_empty() {
            return;
        }

        writeln!(output, "[bindings]").ok();

        // Group bindings by type for compact output
        let mut by_type: FxHashMap<BindingType, Vec<&str>> = FxHashMap::default();
        for (name, bt) in &self.bindings.bindings {
            by_type.entry(*bt).or_default().push(name.as_str());
        }

        // Output in a consistent order
        let type_order = [
            BindingType::SetupConst,
            BindingType::SetupRef,
            BindingType::SetupMaybeRef,
            BindingType::SetupReactiveConst,
            BindingType::SetupLet,
            BindingType::Props,
            BindingType::PropsAliased,
            BindingType::Data,
            BindingType::Options,
            BindingType::LiteralConst,
            BindingType::JsGlobalUniversal,
            BindingType::JsGlobalBrowser,
            BindingType::JsGlobalNode,
            BindingType::JsGlobalDeno,
            BindingType::JsGlobalBun,
            BindingType::VueGlobal,
            BindingType::ExternalModule,
        ];

        for bt in type_order {
            if let Some(names) = by_type.get(&bt) {
                writeln!(output, "{}:{}", bt.to_vir(), names.join(",")).ok();
            }
        }
        writeln!(output).ok();
    }

    fn write_errors(&self, output: &mut String) {
        if self.invalid_exports.is_empty() {
            return;
        }

        writeln!(output, "[errors]").ok();
        for ie in &self.invalid_exports {
            writeln!(output, "{}={:?}@{}:{}", ie.name, ie.kind, ie.start, ie.end).ok();
        }
        writeln!(output).ok();
    }
}
