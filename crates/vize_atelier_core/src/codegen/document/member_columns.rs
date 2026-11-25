//! Static member positions for render maps composed into Vue files.
//!
//! Only the SFC assembly composes a render map into a component's, and only
//! when it asked for a map, so the request is held for the compile it wraps
//! rather than passed down every backend's signature: a compile without a
//! map never reads it, and a standalone compiler keeps its complete map.

use std::cell::Cell;

use super::{EmitDocument, SourceMapBuilder, rewritten_identifier_spans};
use oxc_syntax::identifier::is_identifier_name;
use vize_l0::Span;

std::thread_local! {
    static COMPOSING_SFC_MAP: Cell<bool> = const { Cell::new(false) };
}

/// Run `compile`, a template compile whose render map the SFC assembly
/// composes into the component's: every render map it serializes on this
/// thread also carries the static member columns.
pub fn composing_sfc_render_map<R>(compile: impl FnOnce() -> R) -> R {
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            COMPOSING_SFC_MAP.set(self.0);
        }
    }
    let _restore = Restore(COMPOSING_SFC_MAP.replace(true));
    compile()
}

/// Whether the render map being serialized is composed into a Vue file.
pub(super) fn composing_sfc_map() -> bool {
    COMPOSING_SFC_MAP.get()
}

impl EmitDocument {
    /// Add the static properties of expressions whose complete rewrite we
    /// can verify, for a render map composed into a Vue file.
    #[cold]
    #[inline(never)]
    pub(super) fn write_member_columns(&self, builder: &mut SourceMapBuilder, source: &str) {
        for link in self.links().iter().filter(|link| link.name.is_none()) {
            let Some(authored) =
                source.get(link.authored.start as usize..link.authored.end as usize)
            else {
                continue;
            };
            let Some(emitted) = self
                .as_str()
                .get(link.generated.start as usize..link.generated.end as usize)
            else {
                continue;
            };
            // Text and static attribute links can cover only the contents of a
            // generated string. A dot there belongs to the literal value.
            if emitted == authored && self.is_string_content(link.generated) {
                continue;
            }
            visit_static_member_spans(authored, emitted, link.authored.start, |out, at| {
                builder.add_raw((link.generated.start + out.start) as usize, at.start);
            });
        }
    }

    fn is_string_content(&self, span: Span) -> bool {
        let text = self.as_str().as_bytes();
        let before = (span.start as usize)
            .checked_sub(1)
            .and_then(|i| text.get(i));
        matches!(before, Some(b'\'' | b'"' | b'`')) && before == text.get(span.end as usize)
    }
}

/// Visit property names in a verified expression rewrite. A rewritten receiver
/// can grow a context prefix or an unref call. Its entire static access suffix
/// must survive byte for byte, including optional chaining and whitespace.
fn visit_static_member_spans(
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
        || (emitted != authored && rewritten_identifier_spans(authored, emitted).is_none())
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
    use super::{EmitDocument, visit_static_member_spans};
    use vize_l0::Span;

    #[test]
    fn a_literal_with_a_dot_keeps_its_existing_map() {
        let text = "photo.description";
        let span = Span::new(0, text.len() as u32);
        for quote in ['\'', '"', '`'] {
            let mut doc = EmitDocument::default();
            doc.push_char(quote);
            doc.push_linked(text, span);
            doc.push_char(quote);
            assert_eq!(
                super::composing_sfc_render_map(|| doc.source_map("a.vue", text)),
                doc.source_map("a.vue", text)
            );
        }
        let mut read = EmitDocument::default();
        read.push_expression(text, span, text);
        assert_ne!(
            super::composing_sfc_render_map(|| read.source_map("a.vue", text)),
            read.source_map("a.vue", text)
        );
    }

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
