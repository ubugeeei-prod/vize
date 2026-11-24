//! Source positions for static property reads in a mapped template expression.

use oxc_syntax::identifier::is_identifier_name;

use crate::Span;

/// Visit property names in a verified expression rewrite. A rewritten receiver
/// can grow a context prefix or an unref call. Its entire static access suffix
/// must survive byte for byte, including optional chaining and whitespace.
pub fn visit_static_member_spans(
    authored: &str,
    emitted: &str,
    source_start: u32,
    mut visit: impl FnMut(Span, Span),
) {
    let mut parts = authored.split_inclusive('.');
    let Some(receiver) = parts.next().filter(|part| part.ends_with('.')) else {
        return;
    };
    let root = member_name(receiver);
    let Some(suffix) = authored.strip_prefix(root) else {
        return;
    };
    if !is_identifier_name(root)
        || authored.ends_with('.')
        || !emitted.ends_with(suffix)
        || !parts
            .clone()
            .all(|part| is_identifier_name(member_name(part)))
    {
        return;
    }

    let mut offset = receiver.len();
    for part in parts {
        let start = offset + part.len() - part.trim_start().len();
        let len = member_name(part).len() as u32;
        let out = (emitted.len() - (authored.len() - start)) as u32;
        let at = source_start + start as u32;
        visit(Span::new(out, out + len), Span::new(at, at + len));
        offset += part.len();
    }
}

fn member_name(part: &str) -> &str {
    match part.strip_suffix('.') {
        Some(before_dot) => before_dot.strip_suffix('?').unwrap_or(before_dot).trim(),
        None => part.trim(),
    }
}

#[cfg(test)]
mod tests {
    use super::visit_static_member_spans;
    use crate::Span;
    use alloc::vec::Vec;

    #[test]
    fn properties_use_distinct_offsets_after_a_rewritten_receiver() {
        let authored = "旅 ?. hôtel.hôtel";
        let emitted = "_unref(旅) ?. hôtel.hôtel";
        let mut spans = Vec::new();
        visit_static_member_spans(authored, emitted, 10, |out, at| spans.push((out, at)));
        assert_eq!(
            spans,
            [
                (Span::new(15, 21), Span::new(17, 23)),
                (Span::new(22, 28), Span::new(24, 30)),
            ]
        );
        for (source, output) in [
            ("x.y + z", "_ctx.x.y + _ctx.z"),
            ("'a.b'", "'a.b'"),
            ("x.y", "_ctx.x.z"),
        ] {
            visit_static_member_spans(source, output, 0, |_, _| panic!("unexpected member"));
        }
    }
}
