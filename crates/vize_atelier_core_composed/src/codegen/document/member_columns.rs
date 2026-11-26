// DO NOT EDIT. Generated from crates/vize_atelier_core/src/codegen/document/member_columns.rs by `node tools/commands/generate/composed-crates.ts`.
// Edit the source and run the command again.
//! Static member positions for render maps composed into Vue files.

use super::{EmitDocument, SourceMapBuilder};
use oxc_syntax::identifier::is_identifier_name;
use vize_l0::{Span, String};

impl EmitDocument {
    /// Serialize the render map the SFC assembly composes into a Vue file:
    /// [`Self::source_map`] plus the static properties of expressions whose
    /// complete rewrite we can verify.
    pub fn composed_source_map(&self, filename: &str, source: &str) -> String {
        let mut builder = SourceMapBuilder::new();
        self.write_segments(&mut builder);
        self.write_member_columns(&mut builder, source);
        builder.finish(self.as_str(), filename, source)
    }

    /// Add the static properties of expressions whose complete rewrite we
    /// can verify, for a render map composed into a Vue file.
    fn write_member_columns(&self, builder: &mut SourceMapBuilder, source: &str) {
        let links = self.links();
        for (index, link) in links.iter().enumerate() {
            if link.name.is_some() {
                continue;
            }
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
            // An expression whose rewrite was verified when it was linked
            // carries its rewritten identifiers' named links right after it.
            let verified = emitted == authored
                || links.get(index + 1).is_some_and(|next| {
                    next.name.is_some()
                        && link.generated.start <= next.generated.start
                        && next.generated.end <= link.generated.end
                });
            visit_static_member_spans(
                authored,
                emitted,
                link.authored.start,
                verified,
                |out, at| {
                    builder.add_raw((link.generated.start + out.start) as usize, at.start);
                },
            );
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
    verified: bool,
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
        || !verified
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
    use super::super::super::rewrite_spans::rewritten_identifier_spans;
    use super::{EmitDocument, visit_static_member_spans};
    use vize_l0::Span;

    fn verified(authored: &str, emitted: &str) -> bool {
        authored == emitted || rewritten_identifier_spans(authored, emitted).is_some()
    }

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
                doc.composed_source_map("a.vue", text),
                doc.source_map("a.vue", text)
            );
        }
        let mut read = EmitDocument::default();
        read.push_expression(text, span, text);
        assert_ne!(
            read.composed_source_map("a.vue", text),
            read.source_map("a.vue", text)
        );
    }

    #[test]
    fn a_rewritten_receiver_keeps_its_member_columns() {
        let authored = "photo.description";
        let mut read = EmitDocument::default();
        read.push_expression("_ctx.photo.description", Span::new(0, 17), authored);
        assert_ne!(
            read.composed_source_map("a.vue", authored),
            read.source_map("a.vue", authored)
        );
    }

    #[test]
    fn properties_use_distinct_offsets_after_a_rewritten_receiver() {
        let authored = "旅 ?. hôtel.hôtel";
        let emitted = "_unref(旅) ?. hôtel.hôtel";
        let mut spans = Vec::new();
        let checked = verified(authored, emitted);
        visit_static_member_spans(authored, emitted, 10, checked, |out, at| {
            spans.push((out, at));
        });
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
            let checked = verified(source, output);
            visit_static_member_spans(source, output, 0, checked, |_, _| {
                panic!("unexpected member")
            });
        }
    }
}
