//! Primitive Vue builtin facts recorded by the existing declarator AST walk.
//! These are source-owned editor facts, independent of native checker answers.

use super::TypeResolver;
use crate::reactivity::ReactiveKind;
use vize_carton::{CompactString, FxHashMap};

/// The private editor table lives on the heap only when a fact is proven.
#[derive(Default)]
pub(super) struct BuiltinReactiveTypes {
    facts: FxHashMap<CompactString, BuiltinReactiveType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Primitive {
    Boolean,
    Number,
    String,
}

/// A proven builtin's primitive value and exact authored identifier span.
/// Only the script AST producer can construct or insert these facts.
#[derive(Debug, Clone)]
pub struct BuiltinReactiveType {
    pub(crate) name: CompactString,
    pub(crate) span: (u32, u32),
    pub(crate) kind: ReactiveKind,
    pub(crate) value: Primitive,
}

impl BuiltinReactiveType {
    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn span(&self) -> (u32, u32) {
        self.span
    }

    pub fn kind(&self) -> ReactiveKind {
        self.kind
    }

    pub fn value_type(&self) -> &'static str {
        match self.value {
            Primitive::Boolean => "boolean",
            Primitive::Number => "number",
            Primitive::String => "string",
        }
    }
}

impl TypeResolver {
    pub fn builtin_reactive_types(&self) -> impl Iterator<Item = &BuiltinReactiveType> {
        self.builtin_reactive
            .iter()
            .flat_map(|table| table.facts.values())
    }

    pub(crate) fn builtin_reactive_type(&self, name: &str) -> Option<&BuiltinReactiveType> {
        self.builtin_reactive.as_deref()?.facts.get(name)
    }

    pub(crate) fn clear_builtin_reactive_type(&mut self, name: &str) {
        if let Some(facts) = self.builtin_reactive.as_deref_mut() {
            facts.facts.remove(name);
        }
    }

    pub(crate) fn clear_builtin_reactive_types(&mut self) {
        self.builtin_reactive = None;
    }

    pub(crate) fn record_builtin_reactive_type(&mut self, fact: BuiltinReactiveType) {
        self.builtin_reactive
            .get_or_insert_with(Default::default)
            .facts
            .insert(fact.name.clone(), fact);
    }

    pub(super) fn merge_builtin_reactive_types(
        &mut self,
        other: Option<Box<BuiltinReactiveTypes>>,
    ) {
        if let Some(other) = other {
            let facts = self.builtin_reactive.get_or_insert_with(Default::default);
            for (name, fact) in other.facts {
                facts.facts.entry(name).or_insert(fact);
            }
        }
    }

    pub(crate) fn shift_builtin_reactive_types(&mut self, delta: u32) {
        for fact in self
            .builtin_reactive
            .iter_mut()
            .flat_map(|table| table.facts.values_mut())
        {
            fact.span.0 = fact.span.0.saturating_add(delta);
            fact.span.1 = fact.span.1.saturating_add(delta);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn builtin_types_default_analysis_keeps_whole_output_without_fact_storage() {
        let source = "import { ref, computed } from 'vue'; const count = ref(0); const doubled = computed(() => count.value * 2);";
        let ordinary = crate::script_parser::parse_script_setup(source);
        let demanded = crate::script_parser::parse_script_setup_for_unused::<true>(
            source, None, false, false, false,
        );
        assert!(ordinary.types.builtin_reactive.is_none());
        assert!(demanded.types.builtin_reactive.is_some());
        assert_eq!(format!("{ordinary:?}"), format!("{demanded:?}"));
        let allocator = oxc_allocator::Allocator::default();
        let parsed = crate::script_parser::parse_program_for_analysis(
            &allocator,
            source,
            oxc_span::SourceType::ts(),
        );
        let parse_free =
            crate::script_parser::analyze_script_setup_program(&parsed.program, source, None);
        assert!(parse_free.types.builtin_reactive.is_none());
        assert_eq!(format!("{ordinary:?}"), format!("{parse_free:?}"));
        let mut drawer = crate::Drawer::with_options(crate::DrawerOptions::full());
        drawer.draw_script_setup(source);
        let ordinary = drawer.finish();
        let mut drawer = crate::Drawer::with_options(crate::DrawerOptions::full());
        drawer.draw_script_setup_with_builtin_types(source);
        let demanded = drawer.finish();
        assert!(ordinary.types.builtin_reactive.is_none());
        assert!(demanded.types.builtin_reactive.is_some());
        assert_eq!(format!("{ordinary:?}"), format!("{demanded:?}"));
    }
}
