use vize_carton::{String, ToCompactString};

use crate::script::ScriptCompileContext;

use super::super::super::props::extract_emit_names_from_type;
use super::props::build_user_props_decl;

/// Find the matching `}` for the first `{` in `opts`, returning `(open, close)`
/// byte indices.
fn find_object_close(opts: &str) -> Option<(usize, usize)> {
    let bytes = opts.as_bytes();
    let open = opts.find('{')?;
    let mut depth = 0i32;
    let mut i = open;
    let mut in_str: Option<u8> = None;
    while i < bytes.len() {
        let c = bytes[i];
        if let Some(q) = in_str {
            if c == b'\\' {
                i += 2;
                continue;
            }
            if c == q {
                in_str = None;
            }
            i += 1;
            continue;
        }
        match c {
            b'"' | b'\'' | b'`' => in_str = Some(c),
            b'{' | b'[' | b'(' => depth += 1,
            b'}' | b']' | b')' => {
                depth -= 1;
                if depth == 0 {
                    return Some((open, i));
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// A parsed top-level property of a defineModel options object.
struct OptionProp {
    /// byte offset of the property start within the options string
    start: usize,
    /// property key (e.g. `set`, `default`, `type`)
    key: String,
}

/// Parse the top-level members of an object-literal options string `{ ... }`.
/// Returns `None` if it contains a spread/computed key (Vue keeps such options
/// verbatim).
fn parse_option_props(opts: &str, open: usize, close: usize) -> Option<Vec<OptionProp>> {
    let bytes = opts.as_bytes();
    let mut props: Vec<OptionProp> = Vec::new();
    let mut j = open + 1;
    let mut depth = 0i32;
    let mut in_str: Option<u8> = None;
    let mut member_start: Option<usize> = None;
    let mut key_buf = String::default();
    let mut key_done = false;
    while j < close {
        let c = bytes[j];
        if let Some(q) = in_str {
            if c == b'\\' {
                j += 2;
                continue;
            }
            // collect quoted key characters while still reading the key
            if member_start.is_some() && !key_done && depth == 0 && c != q {
                key_buf.push(c as char);
            }
            if c == q {
                in_str = None;
            }
            j += 1;
            continue;
        }
        match c {
            b'.' if depth == 0 && member_start.is_none() => {
                // spread element (`...x`) -> bail
                return None;
            }
            b'"' | b'\'' | b'`' => {
                if member_start.is_none() {
                    member_start = Some(j);
                }
                in_str = Some(c);
            }
            b'[' if depth == 0 && member_start.is_none() => {
                // computed key -> bail
                return None;
            }
            b'{' | b'[' | b'(' => {
                if member_start.is_none() {
                    member_start = Some(j);
                }
                depth += 1;
            }
            b'}' | b']' | b')' => depth -= 1,
            b',' if depth == 0 => {
                if let Some(ms) = member_start.take() {
                    props.push(OptionProp {
                        start: ms,
                        key: clean_key(&key_buf),
                    });
                }
                key_buf.clear();
                key_done = false;
            }
            b':' if depth == 0 && !key_done && member_start.is_some() => {
                key_done = true;
            }
            _ => {
                if member_start.is_none() && !c.is_ascii_whitespace() {
                    member_start = Some(j);
                }
                if member_start.is_some() && !key_done && depth == 0 && !c.is_ascii_whitespace() {
                    key_buf.push(c as char);
                }
            }
        }
        j += 1;
    }
    if let Some(ms) = member_start.take() {
        props.push(OptionProp {
            start: ms,
            key: clean_key(&key_buf),
        });
    }
    Some(props)
}

fn clean_key(raw: &str) -> String {
    raw.trim()
        .trim_matches(['\'', '"', '`'])
        .to_compact_string()
}

/// Produce the prop-options string for a single defineModel, stripping the
/// runtime-only `get`/`set` accessors (which belong only on the `useModel`
/// runtime argument), matching `@vue/compiler-sfc`'s `genModelProps`.
fn strip_runtime_accessors(opts: &str) -> Option<String> {
    let (open, close) = find_object_close(opts)?;
    let props = parse_option_props(opts, open, close)?;
    // Build result by removing get/set spans from last to first, mirroring Vue's
    // `slice(0, start) + slice(end)` where `end = next ? next.start : close`.
    let mut result = opts.to_compact_string();
    for idx in (0..props.len()).rev() {
        let p = &props[idx];
        if p.key == "get" || p.key == "set" {
            let end = props.get(idx + 1).map(|n| n.start).unwrap_or(close);
            result.replace_range(p.start..end, "");
        }
    }
    Some(result)
}

/// Build model props (and combine props/emits) when defineModel is used.
pub(super) fn build_model_props_emits(
    ctx: &ScriptCompileContext,
    model_infos: &[(String, String, Option<String>)],
    is_ts: bool,
    needs_prop_type: bool,
    needs_merge_defaults: bool,
) -> Vec<u8> {
    let mut buf: Vec<u8> = Vec::new();

    // ---- props ----
    if !model_infos.is_empty() {
        // model props declaration: `{\n    "name": <opts>,\n    "nameModifiers": {},\n  }`
        let mut model_decl: Vec<u8> = Vec::new();
        model_decl.push(b'{');
        for (model_name, _binding_name, options) in model_infos {
            model_decl.extend_from_slice(b"\n    \"");
            model_decl.extend_from_slice(model_name.as_bytes());
            model_decl.extend_from_slice(b"\": ");
            if let Some(opts) = options {
                model_decl.extend_from_slice(opts.as_bytes());
            } else {
                model_decl.extend_from_slice(b"{}");
            }
            model_decl.extend_from_slice(b",\n    \"");
            if model_name == "modelValue" {
                model_decl.extend_from_slice(b"modelModifiers");
            } else {
                model_decl.extend_from_slice(model_name.as_bytes());
                model_decl.extend_from_slice(b"Modifiers");
            }
            model_decl.extend_from_slice(b"\": {},");
        }
        model_decl.extend_from_slice(b"\n  }");

        let user_props = build_user_props_decl(ctx, is_ts, needs_prop_type, needs_merge_defaults);
        buf.extend_from_slice(b"  props: ");
        if let Some(user) = user_props {
            buf.extend_from_slice(b"/*@__PURE__*/_mergeModels(");
            buf.extend_from_slice(user.as_bytes());
            buf.extend_from_slice(b", ");
            buf.extend_from_slice(&model_decl);
            buf.push(b')');
        } else {
            buf.extend_from_slice(&model_decl);
        }
        buf.extend_from_slice(b",\n");
    }

    // ---- emits ----
    // User-declared emits, kept verbatim from source (array) or extracted from type.
    let user_emits: Option<Vec<u8>> = ctx.macros.define_emits.as_ref().and_then(|m| {
        if !m.args.is_empty() {
            Some(m.args.trim().as_bytes().to_vec())
        } else if let Some(ref type_args) = m.type_args {
            let names = extract_emit_names_from_type(type_args);
            if names.is_empty() {
                None
            } else {
                let mut v = Vec::new();
                v.push(b'[');
                for (i, n) in names.iter().enumerate() {
                    if i > 0 {
                        v.extend_from_slice(b", ");
                    }
                    v.push(b'"');
                    v.extend_from_slice(n.as_bytes());
                    v.push(b'"');
                }
                v.push(b']');
                Some(v)
            }
        } else {
            None
        }
    });

    // Model emits: ["update:name", ...]
    let model_emits: Vec<u8> = {
        let mut v = Vec::new();
        v.push(b'[');
        for (i, (name, _, _)) in model_infos.iter().enumerate() {
            if i > 0 {
                v.extend_from_slice(b", ");
            }
            v.extend_from_slice(b"\"update:");
            v.extend_from_slice(name.as_bytes());
            v.push(b'"');
        }
        v.push(b']');
        v
    };

    let emits_decl: Option<Vec<u8>> = if !model_infos.is_empty() {
        Some(match user_emits {
            Some(user) => {
                let mut v = Vec::new();
                v.extend_from_slice(b"/*@__PURE__*/_mergeModels(");
                v.extend_from_slice(&user);
                v.extend_from_slice(b", ");
                v.extend_from_slice(&model_emits);
                v.push(b')');
                v
            }
            None => model_emits,
        })
    } else {
        user_emits
    };

    if let Some(decl) = emits_decl {
        buf.extend_from_slice(b"  emits: ");
        buf.extend_from_slice(&decl);
        buf.extend_from_slice(b",\n");
    }

    buf
}

/// Collect model info from defineModel calls.
///
/// Returns Vec of (model_name, binding_name, prop_options).
pub(super) fn collect_model_infos(
    ctx: &ScriptCompileContext,
) -> Vec<(String, String, Option<String>)> {
    ctx.macros
        .define_models
        .iter()
        .map(|m| {
            let args = m.args.trim();
            let has_name = args.starts_with(['\'', '"', '`']);
            let model_name = if has_name {
                args.trim_start_matches(['\'', '"', '`'])
                    .split(['\'', '"', '`'])
                    .next()
                    .unwrap_or("modelValue")
                    .to_compact_string()
            } else {
                "modelValue".to_compact_string()
            };
            let binding_name = m
                .binding_name
                .as_deref()
                .map(String::from)
                .unwrap_or_else(|| model_name.clone());

            // Locate the options argument (second arg if named, first arg otherwise).
            let raw_options: Option<&str> = if args.is_empty() {
                None
            } else if has_name {
                // skip the string literal, then the comma
                args.split_once(',').map(|(_, rest)| rest.trim())
            } else {
                Some(args)
            };
            let options = raw_options.and_then(|opts| {
                if opts.starts_with('{') {
                    // strip runtime-only get/set accessors for the prop options
                    Some(strip_runtime_accessors(opts).unwrap_or_else(|| opts.to_compact_string()))
                } else {
                    None
                }
            });
            (model_name, binding_name, options)
        })
        .collect()
}
