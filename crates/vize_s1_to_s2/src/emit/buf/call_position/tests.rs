use oxc_syntax::identifier::is_identifier_part;

use super::super::super::helper::Helper;
use super::{AliasPositions, quoted_end};

/// The per-alias rescan the single scan replaced: the exact-equality
/// oracle for [`AliasPositions`].
fn helper_call_position(text: &str, alias: &str) -> Option<usize> {
    let bytes = text.as_bytes();
    let alias = alias.as_bytes();
    let mut position = 0;
    while position < bytes.len() {
        match bytes[position] {
            b'\'' | b'"' | b'`' => position = quoted_end(bytes, position),
            b'/' if bytes.get(position + 1) == Some(&b'/') => {
                position = bytes[position + 2..]
                    .iter()
                    .position(|byte| *byte == b'\n')
                    .map_or(bytes.len(), |end| position + 2 + end);
            }
            b'/' if bytes.get(position + 1) == Some(&b'*') => {
                position = bytes[position + 2..]
                    .windows(2)
                    .position(|pair| pair == b"*/")
                    .map_or(bytes.len(), |end| position + 4 + end);
            }
            _ if bytes[position..].starts_with(alias)
                && text[..position]
                    .chars()
                    .next_back()
                    .is_none_or(|ch| !is_identifier_part(ch) && ch != '.') =>
            {
                let after = position + alias.len();
                if bytes[after..]
                    .iter()
                    .find(|byte| !byte.is_ascii_whitespace())
                    == Some(&b'(')
                {
                    return Some(position);
                }
                position = after;
            }
            _ => position += 1,
        }
    }
    None
}

fn oracle(chunks: &[&str], helper: Helper) -> Option<usize> {
    let mut offset = 0;
    for chunk in chunks {
        if let Some(position) = helper_call_position(chunk, helper.alias()) {
            return Some(offset + position);
        }
        offset += chunk.len();
    }
    None
}

const CASES: &[&[&str]] = &[
    &["return (_openBlock(), _createElementBlock(\"div\", null, _toDisplayString(_ctx.a), 1))"],
    &["_createVNodeX(1) _createVNode (2) x._createVNode(3) $_createVNode(4) _createVNode\n(5)"],
    &["'_createVNode(' \"_toDisplayString(\" `_renderList(${a})` _renderList(b)"],
    &["// _openBlock(\n/* _createTextVNode( */ _createTextVNode(\"x\")"],
    &[
        "é_normalizeClass(a) _normalizeClassé(b) _normalizeClass(c)",
        "_mergeProps(d)",
    ],
    &[
        "[_normalizeProps(_guardReactiveProps(p))]",
        "_mergeProps(a, b)",
        "_normalizeProps(c)",
    ],
    &["_withCtx(() => [_createTextVNode(\"a\\\"b(\"), _withDirectives(x, [[_vShow, s]])])"],
    &["_createSlots({ _: 2 }, [_renderList(l, (i) => ({ name: i, fn: _withCtx(() => []) }))])"],
    &["unterminated '_openBlock(", "_openBlock()"],
    &["/* open", "_openBlock() */ _openBlock()"],
    &[
        "_resolveComponent(\"A\")_resolveDirective(\"b\")",
        "",
        "_toHandlers(h)",
    ],
];

#[test]
fn single_scan_matches_the_per_alias_rescan() {
    for chunks in CASES {
        let positions = AliasPositions::scan(chunks.iter().copied());
        for helper in Helper::ALL {
            assert_eq!(
                positions.first(helper),
                oracle(chunks, helper),
                "{} in {chunks:?}",
                helper.alias()
            );
        }
    }
}
