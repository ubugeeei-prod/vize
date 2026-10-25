//! Immutable JSX component values proved by the existing semantic program.

use oxc_ast::ast::JSXElementName;
use vize_l0::{FxHashSet, profiler::global_profiler};

use super::Lowerer;

impl Lowerer<'_, '_, '_> {
    pub(super) fn is_stable_component_identifier(&mut self, name: &JSXElementName<'_>) -> bool {
        if self.current_output_mode != crate::JsxOutputMode::Vapor {
            return false;
        }
        let JSXElementName::IdentifierReference(reference) = name else {
            return false;
        };
        let Some(scoping) = self.scoping.as_ref() else {
            return false;
        };
        let Some(symbol) = reference
            .reference_id
            .get()
            .and_then(|reference| scoping.get_reference(reference).symbol_id())
        else {
            return false;
        };
        let profiler = global_profiler();
        let profiling = profiler.is_enabled();
        if self.stable_component_bindings.is_none() {
            let mut stable = FxHashSet::default();
            let mut symbols = 0;
            let mut references = 0;
            vize_l0::profile!("atelier.jsx.lower.component_stability", {
                for candidate in scoping.symbol_ids() {
                    if profiling {
                        symbols += 1;
                    }
                    if !scoping.symbol_flags(candidate).is_const_variable() {
                        continue;
                    }
                    // `symbol_is_mutated` assumes const assignments are absent;
                    // inspect the real references so erroneous writes cannot mint
                    // stability authority either. Imports retain live semantics.
                    let mut written = false;
                    for reference in scoping.get_resolved_references(candidate) {
                        if profiling {
                            references += 1;
                        }
                        written |= reference.is_write();
                    }
                    if !written {
                        stable.insert(candidate);
                    }
                }
            });
            if profiling {
                profiler.record_counter_enabled("atelier.jsx.lower.component_stability_builds", 1);
                profiler.record_counter_enabled(
                    "atelier.jsx.lower.component_stability_symbols",
                    symbols,
                );
                profiler.record_counter_enabled(
                    "atelier.jsx.lower.component_stability_references",
                    references,
                );
            }
            self.stable_component_bindings = Some(stable);
        }
        let stable = self
            .stable_component_bindings
            .as_ref()
            .is_some_and(|bindings| bindings.contains(&symbol));
        if profiling {
            profiler.record_counter_enabled("atelier.jsx.lower.component_stability_queries", 1);
            profiler.record_counter_enabled(
                "atelier.jsx.lower.stable_component_values",
                u64::from(stable),
            );
        }
        stable
    }
}
