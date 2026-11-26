const REF_FACTORY_NAMES: &[&str] = &[
    "ref",
    "computed",
    "shallowRef",
    "toRef",
    "customRef",
    "defineModel",
    "useTemplateRef",
];

pub(super) fn source_may_contain_ref_operand(source: &str) -> bool {
    if !REF_FACTORY_NAMES
        .iter()
        .any(|factory| source.contains(factory))
    {
        return false;
    }
    if !source.is_ascii() {
        return true;
    }

    // The operand scan below only looks at the names `collect_direct_ref_bindings`
    // proves are refs, so it may skip a block only when that list is complete:
    // every factory call site is a plain `name = factory(...)` declaration and
    // no factory is reachable under another name.
    let bindings = collect_direct_ref_bindings(source);
    if bindings.is_empty()
        || source.contains("* as")
        || factory_is_aliased(source)
        || factory_call_sites(source) > bindings.len()
    {
        return true;
    }

    // `emit('event', value)` is an ordinary call to the byte prefilter, but a
    // captured `defineEmits()` binding makes every payload argument a value
    // position. Keep these blocks so the AST rule can resolve both bindings.
    if source.contains("defineEmits") {
        return true;
    }

    bindings
        .into_iter()
        .any(|binding| identifier_may_be_ref_operand(source, binding))
}

fn collect_direct_ref_bindings(source: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let bytes = source.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'=' || is_equality_or_arrow(bytes, index) {
            continue;
        }
        let rest = source.get(index + 1..).unwrap_or_default().trim_start();
        if !starts_with_ref_factory_call(rest) {
            continue;
        }
        if let Some(name) = identifier_before_assignment(source, index) {
            names.push(name);
        }
    }
    names
}

/// Occurrences of `factory(` / `factory<` as a whole identifier, including
/// member calls such as `Vue.ref(`. Each one may create a ref.
fn factory_call_sites(source: &str) -> usize {
    let bytes = source.as_bytes();
    REF_FACTORY_NAMES
        .iter()
        .map(|factory| {
            source
                .match_indices(factory)
                .filter(|(start, _)| {
                    let end = start + factory.len();
                    has_identifier_boundaries(bytes, *start, end)
                        && source
                            .get(end..)
                            .and_then(|rest| rest.trim_start().as_bytes().first().copied())
                            .is_some_and(|byte| byte == b'(' || byte == b'<')
                })
                .count()
        })
        .sum()
}

/// Whether some factory is renamed (`import { ref as vueRef }`), so its calls
/// are invisible to the name-based scan.
fn factory_is_aliased(source: &str) -> bool {
    let bytes = source.as_bytes();
    REF_FACTORY_NAMES.iter().any(|factory| {
        source.match_indices(factory).any(|(start, _)| {
            let end = start + factory.len();
            has_identifier_boundaries(bytes, start, end)
                && source.get(end..).is_some_and(|rest| {
                    let trimmed = rest.trim_start();
                    trimmed.len() < rest.len()
                        && trimmed.strip_prefix("as").is_some_and(|after| {
                            after
                                .as_bytes()
                                .first()
                                .is_some_and(u8::is_ascii_whitespace)
                        })
                })
        })
    })
}

fn is_equality_or_arrow(bytes: &[u8], index: usize) -> bool {
    matches!(
        bytes.get(index.wrapping_sub(1)),
        Some(b'!' | b'=' | b'<' | b'>')
    ) || matches!(bytes.get(index + 1), Some(b'=' | b'>'))
}

fn starts_with_ref_factory_call(source: &str) -> bool {
    REF_FACTORY_NAMES.iter().any(|factory| {
        source
            .strip_prefix(factory)
            .and_then(|rest| rest.trim_start().as_bytes().first().copied())
            .is_some_and(|byte| byte == b'(' || byte == b'<')
    })
}

fn identifier_before_assignment(source: &str, equals_index: usize) -> Option<&str> {
    let bytes = source.as_bytes();
    let mut end = equals_index;
    while end > 0 && bytes.get(end - 1).is_some_and(u8::is_ascii_whitespace) {
        end -= 1;
    }
    let mut start = end;
    while start > 0
        && bytes
            .get(start - 1)
            .copied()
            .is_some_and(is_ascii_identifier_continue)
    {
        start -= 1;
    }
    source.get(start..end).filter(|name| !name.is_empty())
}

fn identifier_may_be_ref_operand(source: &str, name: &str) -> bool {
    let mut search_start = 0;
    while let Some(relative) = source.get(search_start..).and_then(|rest| rest.find(name)) {
        let start = search_start + relative;
        let end = start + name.len();
        search_start = end;
        if !has_identifier_boundaries(source.as_bytes(), start, end) {
            continue;
        }
        if occurrence_is_ref_initializer(source, start, end)
            || next_non_ws_is_value(source, end)
            || next_non_ws_is_object_key(source, end)
        {
            continue;
        }
        if occurrence_is_obviously_passed_ref(source, start, end) {
            continue;
        }
        return true;
    }
    false
}

fn has_identifier_boundaries(bytes: &[u8], start: usize, end: usize) -> bool {
    !bytes
        .get(start.wrapping_sub(1))
        .is_some_and(|byte| is_ascii_identifier_continue(*byte))
        && !bytes
            .get(end)
            .is_some_and(|byte| is_ascii_identifier_continue(*byte))
}

fn occurrence_is_ref_initializer(source: &str, start: usize, end: usize) -> bool {
    let before = source.get(..start).unwrap_or_default().trim_end();
    let after = source.get(end..).unwrap_or_default().trim_start();
    after.starts_with('=')
        && (before.ends_with("const") || before.ends_with("let") || before.ends_with("var"))
}

fn next_non_ws_is_value(source: &str, end: usize) -> bool {
    source
        .get(end..)
        .unwrap_or_default()
        .trim_start()
        .starts_with(".value")
}

fn next_non_ws_is_object_key(source: &str, end: usize) -> bool {
    source
        .get(end..)
        .unwrap_or_default()
        .trim_start()
        .starts_with(':')
}

/// An occurrence passed as a whole value to a call, `f(name)` or `f(a, name)`.
/// Only the previous non-whitespace byte decides: `(` of a plain call or a `,`
/// argument separator. Anything else before a `)` or `,` (`=== name)`,
/// `2 * name)`, `typeof name,`) is an operand position.
fn occurrence_is_obviously_passed_ref(source: &str, start: usize, end: usize) -> bool {
    let before = source.get(..start).unwrap_or_default().trim_end();
    let after = source.get(end..).unwrap_or_default().trim_start();
    if !matches!(after.as_bytes().first(), Some(b')' | b',')) {
        return false;
    }
    match before.as_bytes().last() {
        Some(b'(') => paren_starts_plain_call(before),
        Some(b',') => true,
        _ => false,
    }
}

fn paren_starts_plain_call(before: &str) -> bool {
    let without_paren = before.trim_end_matches('(').trim_end();
    let Some((start, _)) = without_paren
        .char_indices()
        .rev()
        .find(|(_, ch)| !ch.is_ascii_alphanumeric() && *ch != '_' && *ch != '$')
    else {
        return !is_control_keyword(without_paren);
    };
    let callee = without_paren.get(start + 1..).unwrap_or_default().trim();
    !callee.is_empty() && !is_control_keyword(callee)
}

fn is_control_keyword(value: &str) -> bool {
    matches!(
        value,
        "if" | "while"
            | "for"
            | "switch"
            | "typeof"
            | "void"
            | "delete"
            | "in"
            | "instanceof"
            | "case"
    )
}

fn is_ascii_identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

#[cfg(test)]
mod tests {
    use super::source_may_contain_ref_operand;

    #[test]
    fn skips_value_only_refs() {
        let source = "const count = ref(0)\ncount.value++\nwatch(count, () => {})";
        assert!(!source_may_contain_ref_operand(source));
    }

    #[test]
    fn keeps_operand_shapes() {
        for source in [
            "let count = ref(0)\ncount++",
            "let count = ref(0)\n--count",
            "const count = ref(0)\ncount + 1",
            "const count = ref(0)\nconst x = 1 + count",
            "const flag = ref(false)\nif (flag) {}",
            "const flag = ref(false)\nwhile (flag) {}",
            "const flag = ref(false)\nconst x = flag ? 1 : 2",
        ] {
            assert!(source_may_contain_ref_operand(source), "{source}");
        }
    }

    #[test]
    fn keeps_unicode_identifier_shapes_conservative() {
        assert!(source_may_contain_ref_operand("const 値 = ref(0)\n値++"));
    }

    #[test]
    fn keeps_template_literal_interpolation_operands() {
        assert!(source_may_contain_ref_operand(
            "const count = ref(0)\nconst text = `${count}`"
        ));
    }

    #[test]
    fn keeps_define_emits_payloads() {
        assert!(source_may_contain_ref_operand(
            "const emit = defineEmits(['change'])\nconst active = computed(() => true)\nemit('change', active)"
        ));
    }

    #[test]
    fn keeps_operands_before_a_closing_paren_or_comma() {
        for source in [
            "const root = useTemplateRef('container')\nif (event.target === root) {}",
            "const count = ref(1)\nconst double = computed(() => (2 * count))",
            "const el = useTemplateRef('el')\nlog(typeof el, 1)",
            "const el = useTemplateRef('el')\nlog(typeof(el))",
            "const flag = ref(false)\nlog(!flag)",
        ] {
            assert!(source_may_contain_ref_operand(source), "{source}");
        }
    }

    #[test]
    fn skips_refs_passed_as_call_arguments() {
        for source in [
            "const count = ref(0)\nwatch(count, () => {})",
            "const count = ref(0)\nconst total = ref(0)\nsync(total, count)\ntotal.value++",
        ] {
            assert!(!source_may_contain_ref_operand(source), "{source}");
        }
    }

    #[test]
    fn keeps_blocks_whose_ref_bindings_the_scan_cannot_list() {
        for source in [
            "import { ref, useTemplateRef as utr } from 'vue'\nconst count = ref(0)\ncount.value++\nconst el = utr('el')\nif (el) {}",
            "import * as Vue from 'vue'\nimport { ref } from 'vue'\nconst count = ref(0)\ncount.value++\nconst el = Vue.useTemplateRef('el')\nif (el) {}",
            "const count = ref(0)\ncount.value++\nconst el: Readonly<ShallowRef<HTMLInputElement | null>> = useTemplateRef('el')\nif (el) {}",
            "const count = ref(0)\ncount.value++\nconst el = (useTemplateRef<HTMLInputElement>('el'))\nif (el) {}",
        ] {
            assert!(source_may_contain_ref_operand(source), "{source}");
        }
    }

    #[test]
    fn keeps_define_model_and_template_ref_operands() {
        for source in [
            "const open = defineModel('open')\nconst label = open ? 'open' : 'closed'",
            "const input = useTemplateRef('input')\nif (!input) console.log('missing')",
            "const count = ref(0)\nconst open = defineModel('open')\ncount.value++\nconst label = open ? 'open' : 'closed'",
        ] {
            assert!(source_may_contain_ref_operand(source), "{source}");
        }
    }
}
