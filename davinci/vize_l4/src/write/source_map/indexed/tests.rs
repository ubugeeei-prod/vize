use super::*;
use crate::write::SpanLink;
use vize_l0::Span;

#[test]
fn every_byte_and_checkpoint_agrees_with_the_original_position_convention() {
    for ending in ["\r", "\n", "\r\n", "\u{2028}", "\u{2029}"] {
        for pad in [0, STRIDE - 3, STRIDE - 2, STRIDE - 1, STRIDE, STRIDE + 1] {
            let input = alloc::format!(
                "{}🦀𐀀雪é{ending}ab{ending}日{}",
                "x".repeat(pad),
                "🌿x".repeat(80)
            );
            let index = SourceMapIndex::new(&input);
            let stock = super::super::line_start_table(&input);
            for byte in 0..input.len() + 5 {
                assert_eq!(
                    index.position(byte as u32),
                    super::super::resolve_in_table(&input, &stock, byte),
                    "{ending:?} {pad} {byte}"
                );
            }
        }
    }
}

#[test]
fn arbitrary_link_order_duplicates_names_and_escaping_keep_complete_serialized_bytes() {
    for code in ["", "one line", "a\r\n雪🦀\u{2028}\u{2029}b"] {
        for text in ["", "abc", "\u{0}\t\r\n\"\\雪🦀\u{2029}z"] {
            let index = SourceMapIndex::new(text);
            let mut links = Vec::new();
            for g in (0..code.len() + 3).rev() {
                for s in (0..text.len() + 3).rev() {
                    links.push(SpanLink {
                        generated: Span::new(g as u32, g as u32),
                        authored: Span::new(s as u32, s as u32),
                        name: match (g + s) % 3 {
                            0 => None,
                            1 => Some("雪\"".into()),
                            _ => Some("\t🦀".into()),
                        },
                        segment: (g + s) % 7 != 0,
                    });
                }
            }
            let doc = EmitDocument::from_parts(code.into(), links);
            assert_eq!(
                index.source_map(&doc, "A\"雪.vue"),
                doc.source_map("A\"雪.vue", text)
            );
        }
    }
}
