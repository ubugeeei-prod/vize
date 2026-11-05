//! One constructor `$props` tuple per component, shared by its listener arrays.
//!
//! `typeof Child extends { new (...): { $props: infer __P } }` does not depend
//! on the event. Distributing that conditional again inside every
//! `_prop_arrays` alias re-instantiates it. The shared alias stays a one-tuple
//! (`[__P]`, or `never` when the box is empty) so each listener still unwraps
//! member by member: union constituents and `$IsAny` do not collapse, and the
//! handler annotation keeps the per-event alias name.

use std::cell::RefCell;

use vize_carton::{FxHashMap, String, append, cstr};

use super::super::context::{ComponentContractKey, EventContract, EventContractKey};

/// Not an authored event name. Marks the shared props tuple in the contract map.
const PROPS_MEMBERS_SENTINEL: &str = "\u{0}props_members";

pub(super) struct Emit<'a> {
    pub(super) contracts: &'a RefCell<FxHashMap<EventContractKey, u32>>,
    pub(super) component: &'a ComponentContractKey,
    pub(super) scope_id: u32,
    pub(super) indent: &'a str,
    pub(super) is_any: &'a str,
    pub(super) prop_key: &'a str,
    pub(super) arrays_type: &'a str,
    /// Already one props object (`resolved_props`). `None` reads `props_box`.
    pub(super) direct_props: Option<&'a str>,
    pub(super) props_box: &'a str,
    pub(super) props_members: &'a str,
}

pub(super) fn emit(ts: &mut String, emit: Emit<'_>) {
    let Emit {
        contracts,
        component,
        scope_id,
        indent,
        is_any,
        prop_key,
        arrays_type,
        direct_props,
        props_box,
        props_members,
    } = emit;
    // `any` props infer `__H` as `any`, and `Extract<any, …>` is `any`.
    // Unioning that into the handler annotation would type `(v)` as explicit
    // `any` and hide TS7006 on a loose `:is`.
    let arrays = cstr!(
        "{is_any}<__P> extends true ? never : __P extends {{ {prop_key}?: infer __H }} ? Extract<NonNullable<__H>, readonly unknown[]> : never"
    );
    if let Some(props) = direct_props {
        append!(
            *ts,
            "{indent}type {arrays_type} = {props} extends infer __P ? {arrays} : never;\n",
        );
        return;
    }
    if claim_members(contracts, component, scope_id) {
        append!(
            *ts,
            "{indent}type {props_members} = {props_box} extends infer __B ? __B extends [infer __P] ? [__P] : never : never;\n",
        );
    }
    append!(
        *ts,
        "{indent}type {arrays_type} = {props_members} extends infer __B ? __B extends [infer __P] ? {arrays} : never : never;\n",
    );
}

fn claim_members(
    contracts: &RefCell<FxHashMap<EventContractKey, u32>>,
    component: &ComponentContractKey,
    scope_id: u32,
) -> bool {
    let key = EventContractKey {
        component: component.clone(),
        event: Some(EventContract {
            name: String::from(PROPS_MEMBERS_SENTINEL),
            typed_handler: false,
        }),
    };
    let mut contracts = contracts.borrow_mut();
    if contracts.contains_key(&key) {
        return false;
    }
    contracts.insert(key, scope_id);
    true
}

#[cfg(test)]
mod tests {
    use super::PROPS_MEMBERS_SENTINEL;
    use crate::virtual_ts::generator::generate_virtual_ts;
    use vize_croquis::{Analyzer, AnalyzerOptions};

    const SCRIPT: &str = r#"import Child from './Child.vue'
import Other from './Other.vue'
const onSelect = (value: number) => value
const onClose = () => undefined
const onSave = (value: string) => value
const onOpen = () => undefined
"#;

    const TEMPLATE: &str = r#"<div>
  <Child @select="onSelect" @close="onClose" />
  <Child @select="onSelect" @close="onClose" />
  <Other @save="onSave" @open="onOpen" />
</div>"#;

    fn generate() -> crate::virtual_ts::types::VirtualTsOutput {
        let allocator = vize_carton::Allocator::new();
        let (root, errors) = vize_armature::parse(&allocator, TEMPLATE);
        assert!(errors.is_empty(), "{errors:?}");
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(SCRIPT);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();
        generate_virtual_ts(&summary, Some(SCRIPT), Some(&root), 0)
    }

    #[test]
    fn listeners_share_one_constructor_props_tuple() {
        let output = generate();
        let code = output.code.as_str();
        assert!(
            !code.contains(PROPS_MEMBERS_SENTINEL),
            "the contract sentinel must not be emitted: {code}"
        );
        for component in ["Child", "Other"] {
            let conditional = vize_carton::cstr!("typeof {component} extends {{ new (");
            assert_eq!(
                code.matches(conditional.as_str()).count(),
                1,
                "{component} constructor props conditional: {code}"
            );
        }
        let members: Vec<&str> = code
            .lines()
            .filter(|line| line.contains("_props_members ="))
            .collect();
        assert_eq!(members.len(), 2, "one tuple per component: {code}");
        for line in &members {
            assert!(
                line.contains(
                    "extends infer __B ? __B extends [infer __P] ? [__P] : never : never;"
                ),
                "shared alias keeps the distributive one-tuple: {line}"
            );
            assert!(
                !line.contains("typeof"),
                "the shared tuple must not repeat the constructor conditional: {line}"
            );
        }
        let arrays: Vec<&str> = code
            .lines()
            .filter(|line| line.contains("_prop_arrays ="))
            .collect();
        assert_eq!(
            arrays.len(),
            4,
            "select, close, save and open each declare one array alias: {code}"
        );
        for line in &arrays {
            assert!(
                line.contains(
                    "_props_members extends infer __B ? __B extends [infer __P] ? $IsAny<__P>"
                ),
                "array alias must unwrap the shared tuple, not the constructor: {line}"
            );
            assert!(!line.contains("typeof"), "{line}");
            assert!(!line.contains("$props"), "{line}");
        }
        let child_members = members_alias(
            arrays
                .iter()
                .find(|line| line.contains("_select_prop_arrays"))
                .expect("select arrays"),
        );
        let close_members = members_alias(
            arrays
                .iter()
                .find(|line| line.contains("_close_prop_arrays"))
                .expect("close arrays"),
        );
        assert_eq!(child_members, close_members, "{code}");
        assert_eq!(
            code.matches(child_members).count(),
            3,
            "declaration plus select and close: {code}"
        );
        let other_members = members_alias(
            arrays
                .iter()
                .find(|line| line.contains("_save_prop_arrays"))
                .expect("save arrays"),
        );
        assert_ne!(child_members, other_members);
        assert_eq!(
            code.matches("_select_prop_arrays =").count(),
            1,
            "the repeated Child usage must reuse the first array alias: {code}"
        );
        let annotations: Vec<&str> = code
            .lines()
            .filter(|line| line.contains("_select_handler |"))
            .collect();
        assert_eq!(annotations.len(), 2, "{code}");
        let alias = annotations[0]
            .split("_select_handler | ")
            .nth(1)
            .and_then(|rest| rest.split(" | null").next())
            .expect("annotation alias");
        assert!(alias.contains("_select_prop_arrays"), "{alias}");
        assert!(annotations[1].contains(alias), "{annotations:?}");
        let maps: Vec<&str> = code
            .lines()
            .filter(|line| line.contains("// @vize-map: handler -> "))
            .collect();
        let expected = handler_ranges();
        assert_eq!(maps.len(), expected.len(), "{maps:?}");
        for (line, range) in maps.iter().zip(expected) {
            let marked =
                vize_carton::cstr!("// @vize-map: handler -> {}:{}\n", range.start, range.end);
            assert!(
                line.contains(marked.trim_end()),
                "handler anchor moved: {line} != {marked}"
            );
        }
        for event in ["select", "close", "save", "open"] {
            let name_ranges = event_name_ranges(event);
            let mapped = output
                .mapping
                .spans()
                .iter()
                .flat_map(|span| span.sub_spans.iter())
                .filter(|sub| name_ranges.contains(&sub.src_range))
                .count();
            assert_eq!(
                mapped,
                name_ranges.len(),
                "{event} diagnostic anchors: {name_ranges:?}"
            );
        }
    }

    fn members_alias(line: &str) -> &str {
        line.split(" = ")
            .nth(1)
            .and_then(|rhs| rhs.split(" extends ").next())
            .expect("members alias")
            .trim()
    }

    fn handler_ranges() -> Vec<std::ops::Range<usize>> {
        let mut from = 0usize;
        [
            "onSelect", "onClose", "onSelect", "onClose", "onSave", "onOpen",
        ]
        .into_iter()
        .map(|expr| {
            let (start, text) = TEMPLATE
                .match_indices(expr)
                .find(|(index, _)| *index >= from)
                .expect(expr);
            let end = start + text.len();
            from = end;
            start..end
        })
        .collect()
    }

    fn event_name_ranges(event: &str) -> Vec<std::ops::Range<usize>> {
        let needle = vize_carton::cstr!("@{event}");
        TEMPLATE
            .match_indices(needle.as_str())
            .map(|(index, text)| {
                let start = index + 1;
                start..start + (text.len() - 1)
            })
            .collect()
    }
}
