//! Content-mapper per-prop checks for values the props literal also carries.
//!
//! The upstream content-mapper host reports every virtual diagnostic, so the
//! mapper ignores the rest of a per-prop assignment whose value the props
//! literal carries. The literal types an undeclared attr as `unknown`, so such
//! an assignment keeps only the contract the literal leaves unchecked on its
//! name (#12458). `satisfies` keeps the value contextually typed by the full
//! contract, and the value's own type reaches the annotation, so the message
//! is the per-prop one. Native checks dedupe after mapping and keep the plain
//! assignment.

use super::component_props::{ComponentPropSource, has_per_prop_value_check};
use super::props_literal::{literal_carries_value, named_entry_marks};
use crate::virtual_ts::helpers::{
    component_prop_key, push_ts_string_literal, to_safe_identifier_fragment,
};
use vize_carton::{String, append};
use vize_croquis::croquis::{ComponentUsage, PassedProp};

/// Opening of every owned per-prop annotation. The content mapper appends
/// [`attr_value_declaration`] once when it finds one.
pub(crate) const ATTR_VALUE_REFERENCE: &str = "$AttrValue<";

/// Vue's public attrs, which the literal's tail types for every child.
macro_rules! public_attr_keys {
    () => {
        "keyof import(\"vue\").VNodeProps | keyof import(\"vue\").AllowedComponentProps | keyof import(\"vue\").ComponentCustomProps"
    };
}

/// Only a required fallthrough prop is typed by the literal; testing the one
/// key avoids resolving every required key of the root's attrs per prop.
macro_rules! attr_value {
    ($listener_keys:literal, $fallthrough:literal) => {
        concat!(
            "type $AttrValue<P, C, K, V> = K extends (P extends unknown ? keyof P : never) | ",
            $listener_keys,
            " | ",
            public_attr_keys!(),
            " ? unknown : ",
            $fallthrough,
            " extends infer F ? K extends keyof F ? {} extends Pick<F, K> ? V : unknown : V : V;\n"
        )
    };
}

const SHARED_ATTR_VALUE: &str = attr_value!(
    "keyof import(\"vize:virtual-ts\").$EmitListeners<C>",
    "import(\"vize:virtual-ts\").$FallthroughProps<C>"
);

/// Standalone output keeps the listener and fallthrough helpers local to the
/// template, so they are spelled out here.
const STANDALONE_ATTR_VALUE: &str = attr_value!(
    "(C extends { __vizeEmitProps?: infer E } ? 0 extends (1 & E) ? never : keyof NonNullable<E> : never)",
    "(C extends { readonly __vizeComponentMarker: true; readonly __vizeHasFallthroughProps: true; readonly __vizeFallthroughProps?: infer A } ? 0 extends (1 & A) ? {} : NonNullable<A> : {})"
);

/// `$AttrValue<P, C, K, V>` is the per-prop contract `V` unless the literal
/// types `K`: one of child `C`'s props `P`, its emitted listeners, Vue's
/// public attrs or its required fallthrough props.
pub(crate) const fn attr_value_declaration(shared_virtual_module: bool) -> &'static str {
    if shared_virtual_module {
        SHARED_ATTR_VALUE
    } else {
        STANDALONE_ATTR_VALUE
    }
}

/// The per-prop assignments of one usage whose value the literal carries.
pub(crate) struct OwnedPropValues<'a> {
    owned: Vec<bool>,
    component: String,
    component_ref: &'a str,
    idx: usize,
}

impl<'a> OwnedPropValues<'a> {
    /// Decide the rows from the literal's own entry marks.
    pub(crate) fn new(
        (usage, idx, component_ref): (&ComponentUsage, usize, &'a str),
        source_context: ComponentPropSource<'_>,
    ) -> Self {
        let owned = if source_context.unchecked_prop_values {
            // Merge flags only change `class`/`style` rows, which never own here.
            let marks = named_entry_marks(usage, (false, false));
            usage
                .props
                .iter()
                .enumerate()
                .map(|(index, prop)| {
                    has_per_prop_value_check(prop)
                        && literal_carries_value(usage, &marks, index)
                        && !writes_model(usage, prop)
                })
                .collect()
        } else {
            Vec::new()
        };
        Self {
            owned,
            component: to_safe_identifier_fragment(usage.name.as_str()),
            component_ref,
            idx,
        }
    }

    pub(crate) fn owns(&self, index: usize) -> bool {
        self.owned.get(index).copied().unwrap_or(false)
    }

    /// `: $AttrValue<props, typeof child, key, contract> = `
    pub(crate) fn append_type(
        &self,
        ts: &mut crate::virtual_ts::TextBuf,
        prop: &PassedProp,
        value: &str,
    ) {
        let (component, idx, component_ref) =
            (self.component.as_str(), self.idx, self.component_ref);
        let name = to_safe_identifier_fragment(prop.name.as_str());
        append!(
            *ts,
            ": {ATTR_VALUE_REFERENCE}__{component}_Props_{idx}, typeof {component_ref}, "
        );
        push_ts_string_literal(ts, component_prop_key(prop.name.as_str()).as_str());
        append!(*ts, ", __{component}_{idx}_prop_{name}> = ");
        if needs_parentheses(value) {
            ts.push('(');
        }
    }

    /// ` satisfies contract` after the value.
    pub(crate) fn append_tail(
        &self,
        ts: &mut crate::virtual_ts::TextBuf,
        prop: &PassedProp,
        value: &str,
    ) {
        let (component, idx) = (self.component.as_str(), self.idx);
        let name = to_safe_identifier_fragment(prop.name.as_str());
        if needs_parentheses(value) {
            ts.push(')');
        }
        append!(*ts, " satisfies __{component}_{idx}_prop_{name}");
    }
}

/// Whether `satisfies` written after `value` could bind to less than all of
/// it: a line break ends the declaration first (ASI, TS1434), and a top-level
/// operator weaker than `satisfies` keeps only its right operand. Anything
/// this scan cannot classify, such as a regex or a template, is wrapped.
fn needs_parentheses(value: &str) -> bool {
    let bytes = value.as_bytes();
    let mut depth = 0usize;
    let mut quote = None;
    let mut index = 0;
    while let Some(&byte) = bytes.get(index) {
        index += 1;
        if let Some(open) = quote {
            match byte {
                b'\\' => index += 1,
                b'\n' | b'\r' => return true,
                _ if byte == open => quote = None,
                _ => {}
            }
            continue;
        }
        match byte {
            b'"' | b'\'' => quote = Some(byte),
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => match depth.checked_sub(1) {
                Some(outer) => depth = outer,
                None => return true,
            },
            // `?.` is a member access unless a digit follows (`a ?.5 : b`).
            b'?' if bytes.get(index) == Some(&b'.')
                && !bytes.get(index + 1).is_some_and(u8::is_ascii_digit) =>
            {
                index += 1;
            }
            b'\n' | b'\r' | b'`' | b'/' => return true,
            b'=' | b'&' | b'|' | b'^' | b'?' | b',' if depth == 0 => return true,
            _ => {}
        }
    }
    depth != 0
        || quote.is_some()
        || !bytes.last().is_some_and(|last| {
            last.is_ascii_alphanumeric()
                || matches!(last, b'_' | b'$' | b')' | b']' | b'}' | b'"' | b'\'' | b'!')
        })
}

/// A `v-model` value and its modifiers keep their write-back owners.
fn writes_model(usage: &ComponentUsage, prop: &PassedProp) -> bool {
    usage.events.iter().any(|event| {
        event.start == prop.start && event.end == prop.end && event.name.starts_with("update:")
    })
}

#[cfg(test)]
mod tests {
    use super::needs_parentheses;

    #[test]
    fn only_values_satisfies_could_split_are_wrapped() {
        let plain = [
            "{}",
            "1",
            "-1",
            "'a, b'",
            "\"x ? y\"",
            "foo",
            "foo.bar?.baz",
            "list?.[0]",
            "call(a, b ? c : d)",
            "{ a: 1, b: c ?? d }",
            "[a, b]",
            "a + b * c",
            "a < b",
            "!flag",
            "value!",
            "typeof x",
        ];
        for value in plain {
            assert!(!needs_parentheses(value), "{value}");
        }
        let wrapped = [
            "a ? b : c",
            "a ?.5 : b",
            "a ?? b",
            "a || b",
            "a && b",
            "a | b",
            "a & b",
            "a ^ b",
            "a == b",
            "a !== b",
            "x = 1",
            "() => 1",
            "a, b",
            "a / b",
            "/x/.test(a)",
            "`t`",
            "{}\n",
            "\n  cond\n    ? 1\n    : {}\n",
            "foo.",
            "'open",
            "(a",
            "a)",
            "",
        ];
        for value in wrapped {
            assert!(needs_parentheses(value), "{value:?}");
        }
    }
}
