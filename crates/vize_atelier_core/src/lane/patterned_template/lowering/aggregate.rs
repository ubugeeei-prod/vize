use vize_armature::patterns::{MatchPattern, PatternKind, PatternProperty, PatternRest};
use vize_s0::cstr;

use super::Selector;

impl Selector<'_> {
    pub(super) fn object(
        &mut self,
        properties: &[PatternProperty],
        rest: Option<&PatternRest>,
        value: &str,
    ) {
        self.tests.push(cstr!("if ({value} == null) return null;"));
        for property in properties {
            let key = &property.key.text;
            self.tests
                .push(cstr!("if (!({key} in Object({value}))) return null;"));
            if !matches!(property.pattern.kind, PatternKind::Wildcard) {
                let child = self.temp();
                self.tests.push(cstr!("const {child} = {value}[{key}];"));
                self.emit(&property.pattern, &child);
            }
        }
        if let Some(binding) = rest.and_then(|rest| rest.binding.as_ref()) {
            let copy = self.temp();
            let key = self.temp();
            let excluded = properties
                .iter()
                .map(|property| property.key.text.as_str())
                .collect::<std::vec::Vec<_>>()
                .join(", ");
            // Copy own enumerable keys without rereading excluded getters or
            // invoking a __proto__ setter. Symbols keep their original identity.
            self.copies.push(cstr!(
                "const {copy} = {{}}; for (const {key} of Object.getOwnPropertyNames(Object({value})).concat(Object.getOwnPropertySymbols(Object({value})))) {{ if (![{excluded}].includes({key}) && Object.prototype.propertyIsEnumerable.call({value}, {key})) Object.defineProperty({copy}, {key}, {{ value: {value}[{key}], enumerable: true, configurable: true, writable: true }}); }}"
            ));
            self.bindings
                .push(cstr!("const {} = {copy};", binding.name));
        }
    }

    pub(super) fn array(
        &mut self,
        elements: &[MatchPattern],
        rest: Option<&PatternRest>,
        value: &str,
    ) {
        let operator = if rest.is_some() { ">=" } else { "===" };
        let count = elements.len();
        self.tests.push(cstr!(
            "if (!Array.isArray({value}) || !({value}.length {operator} {count})) return null;"
        ));
        for (index, pattern) in elements.iter().enumerate() {
            if !matches!(pattern.kind, PatternKind::Wildcard) {
                let child = self.temp();
                self.tests.push(cstr!("const {child} = {value}[{index}];"));
                self.emit(pattern, &child);
            }
        }
        if let Some(binding) = rest.and_then(|rest| rest.binding.as_ref()) {
            let copy = self.temp();
            // Array.from copies sparse entries without consulting slice/species.
            self.copies.push(cstr!("const {copy} = Array.from({{ length: {value}.length - {count} }}, (_, i) => {value}[i + {count}]);"));
            self.bindings
                .push(cstr!("const {} = {copy};", binding.name));
        }
    }
}
