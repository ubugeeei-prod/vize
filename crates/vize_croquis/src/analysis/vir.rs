//! VIR (Vize Intermediate Representation) text format output.
//!
//! Generates a TOML-like human-readable representation of the analysis
//! for debugging and inspection purposes.

use super::{Croquis, TypeExportKind};
use crate::macros::MacroKind;
use std::fmt::Write;
use vize_carton::FxHashMap;
use vize_carton::String;
use vize_relief::BindingType;

impl Croquis {
    /// Convert analysis summary to VIR (Vize Intermediate Representation) text format.
    ///
    /// This generates a TOML-like human-readable representation of the analysis.
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

    fn write_surface(&self, output: &mut String) {
        let has_surface = !self.macros.props().is_empty()
            || !self.macros.emits().is_empty()
            || !self.macros.models().is_empty()
            || self
                .macros
                .all_calls()
                .iter()
                .any(|c| matches!(c.kind, MacroKind::DefineExpose | MacroKind::DefineSlots));

        if !has_surface {
            return;
        }

        // [surface.props]
        if !self.macros.props().is_empty() {
            writeln!(output, "[surface.props]").ok();
            for prop in self.macros.props() {
                let req = if prop.required { "!" } else { "?" };
                let def = if prop.default_value.is_some() {
                    "="
                } else {
                    ""
                };
                if let Some(ref ty) = prop.prop_type {
                    writeln!(output, "{}{}:{}{}", prop.name, req, ty, def).ok();
                } else {
                    writeln!(output, "{}{}{}", prop.name, req, def).ok();
                }
            }
            writeln!(output).ok();
        }

        // [surface.emits]
        if !self.macros.emits().is_empty() {
            writeln!(output, "[surface.emits]").ok();
            for emit in self.macros.emits() {
                if let Some(ref ty) = emit.payload_type {
                    writeln!(output, "{}:{}", emit.name, ty).ok();
                } else {
                    writeln!(output, "{}", emit.name).ok();
                }
            }
            writeln!(output).ok();
        }

        // [surface.models]
        if !self.macros.models().is_empty() {
            writeln!(output, "[surface.models]").ok();
            for model in self.macros.models() {
                let name = if model.name.is_empty() {
                    "modelValue"
                } else {
                    model.name.as_str()
                };
                if let Some(ref ty) = model.model_type {
                    writeln!(output, "{}:{}", name, ty).ok();
                } else {
                    writeln!(output, "{}", name).ok();
                }
            }
            writeln!(output).ok();
        }

        // [surface.expose]
        let expose_calls: Vec<_> = self
            .macros
            .all_calls()
            .iter()
            .filter(|c| c.kind == MacroKind::DefineExpose)
            .collect();
        if !expose_calls.is_empty() {
            writeln!(output, "[surface.expose]").ok();
            for call in &expose_calls {
                if let Some(args) = &call.runtime_args {
                    writeln!(output, "{}", args).ok();
                } else {
                    writeln!(output, "@{}:{}", call.start, call.end).ok();
                }
            }
            writeln!(output).ok();
        }

        // [surface.slots]
        let slots_calls: Vec<_> = self
            .macros
            .all_calls()
            .iter()
            .filter(|c| c.kind == MacroKind::DefineSlots)
            .collect();
        if !slots_calls.is_empty() {
            writeln!(output, "[surface.slots]").ok();
            for call in &slots_calls {
                if let Some(type_args) = &call.type_args {
                    writeln!(output, "{}", type_args).ok();
                } else {
                    writeln!(output, "@{}:{}", call.start, call.end).ok();
                }
            }
            writeln!(output).ok();
        }
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

    fn write_scopes(&self, output: &mut String) {
        if self.scopes.is_empty() {
            return;
        }

        // Build a map from scope ID -> prefixed display ID
        // Separate counters for ~, !, # prefixes
        let mut prefix_counters: FxHashMap<&str, u32> = FxHashMap::default();
        let mut id_to_display: FxHashMap<u32, String> = FxHashMap::default();

        // Helper to determine effective prefix by checking parent chain
        // If any ancestor is ClientOnly, child scopes should also be !
        // If any ancestor is server-only, child scopes should also be #
        let get_effective_prefix = |scope: &crate::scope::Scope| -> &'static str {
            // First check the scope's own prefix
            let own_prefix = scope.kind.prefix();
            if own_prefix != "~" {
                return own_prefix;
            }

            // Check parent chain for client-only or server-only context
            let mut visited: vize_carton::SmallVec<[crate::scope::ScopeId; 8]> =
                vize_carton::SmallVec::new();
            let mut queue: vize_carton::SmallVec<[crate::scope::ScopeId; 8]> =
                scope.parents.iter().copied().collect();

            while let Some(parent_id) = queue.pop() {
                if visited.contains(&parent_id) {
                    continue;
                }
                visited.push(parent_id);

                if let Some(parent) = self.scopes.get_scope(parent_id) {
                    let parent_prefix = parent.kind.prefix();
                    if parent_prefix == "!" {
                        return "!"; // Client-only context propagates down
                    }
                    if parent_prefix == "#" {
                        return "#"; // Server-only context propagates down
                    }
                    // Add grandparents to queue
                    for &gp in &parent.parents {
                        if !visited.contains(&gp) {
                            queue.push(gp);
                        }
                    }
                }
            }

            "~" // Default to universal
        };

        for scope in self.scopes.iter() {
            let prefix = get_effective_prefix(scope);
            let counter = prefix_counters.entry(prefix).or_insert(0);
            #[allow(clippy::disallowed_macros)]
            let display_id = format!("{}{}", prefix, *counter);
            id_to_display.insert(scope.id.as_u32(), display_id.into());
            *counter += 1;
        }

        writeln!(output, "[scopes]").ok();
        for scope in self.scopes.iter() {
            let bd_count = scope.bindings().count();

            // Get scope display ID with prefix
            let scope_id_display = id_to_display
                .get(&scope.id.as_u32())
                .map(|s| s.as_str())
                .unwrap_or("?");

            // Build parent references from the parents list using display IDs
            let par = if scope.parents.is_empty() {
                String::default()
            } else {
                let refs: Vec<_> = scope
                    .parents
                    .iter()
                    .filter_map(|p| id_to_display.get(&p.as_u32()))
                    .map(|s| s.as_str())
                    .collect();
                if refs.is_empty() {
                    String::default()
                } else {
                    {
                        #[allow(clippy::disallowed_macros)]
                        let s = format!(" < {}", refs.join(", "));
                        s.into()
                    }
                }
            };

            if bd_count > 0 {
                let bd: Vec<_> = scope.bindings().map(|(n, _)| n).collect();
                writeln!(
                    output,
                    "{} {} @{}:{} [{}]{}",
                    scope_id_display,
                    scope.display_name(),
                    scope.span.start,
                    scope.span.end,
                    bd.join(","),
                    par
                )
                .ok();
            } else {
                writeln!(
                    output,
                    "{} {} @{}:{}{}",
                    scope_id_display,
                    scope.display_name(),
                    scope.span.start,
                    scope.span.end,
                    par
                )
                .ok();
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
