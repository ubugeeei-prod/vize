//! Existing semantic scope maps preserve unsupported values as opaque roots.
use super::{LocalSymbol, Namespace, collect::Collector, declarations::Syntax};
use oxc_syntax::scope::ScopeId;
use vize_carton::String;

impl<'s, 'a> Collector<'s, 'a> {
    pub(super) fn note_global(&mut self, name: &str, local: LocalSymbol) {
        if self
            .global_names
            .as_ref()
            .is_some_and(|names| !names.contains(name))
        {
            return;
        }
        if self.global_symbols.insert(local) {
            self.facts.globals.push((String::from(name), local));
            self.pending.push_back(local);
        }
    }

    pub(super) fn global_scope(&mut self, scope: ScopeId) {
        let semantic = self.semantic;
        let scoping = semantic.scoping();
        let symbols = if let Some(names) = &self.global_names {
            names
                .iter()
                .filter_map(|name| scoping.get_binding(scope, name.as_str().into()))
                .collect::<Vec<_>>()
        } else {
            scoping
                .get_bindings(scope)
                .values()
                .copied()
                .collect::<Vec<_>>()
        };
        for symbol in symbols {
            let flags = scoping.symbol_flags(symbol);
            if !flags.is_value() && !flags.is_namespace_module() {
                continue;
            }
            let local = LocalSymbol {
                symbol,
                namespace: Namespace::Value,
            };
            // An unsupported merged value must not inherit an annotation's
            // provider evidence. Missing syntax already lowers to Opaque.
            if flags.is_class()
                || flags.is_enum()
                || flags.is_namespace_module()
                || flags.is_value_module()
            {
                self.binding(
                    scoping.symbol_name(symbol),
                    Some(symbol),
                    Namespace::Value,
                    Syntax::Opaque,
                    false,
                );
            }
            self.note_global(scoping.symbol_name(symbol), local);
        }
    }
}
