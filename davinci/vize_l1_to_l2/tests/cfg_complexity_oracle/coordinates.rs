//! Reconstruct boundary pairs from the original file and the folio's decoded
//! spelling. The oracle never consults the retained AST or its coordinate map.

use vize_l0::Span;
use vize_l1::markup::entity::{EntityContext, decode_one};

pub struct Coordinates {
    start: u32,
    boundaries: Option<Vec<(usize, usize)>>,
}

impl Coordinates {
    pub fn from_folio(
        file: &str,
        span: Span,
        decoded: &str,
        context: EntityContext,
    ) -> Option<Self> {
        let raw = file.get(span.start as usize..span.end as usize)?;
        if raw == decoded {
            return Some(Self {
                start: span.start,
                boundaries: None,
            });
        }
        let mut rebuilt = String::new();
        let mut boundaries = vec![(0, 0)];
        let mut offset = 0;
        while offset < raw.len() {
            let rest = raw.get(offset..)?;
            if let Some((entity, consumed)) = decode_one(rest.as_bytes(), context) {
                entity.for_each(|ch| rebuilt.push(ch));
                offset += consumed;
            } else {
                let scalar = rest.chars().next()?;
                rebuilt.push(scalar);
                offset += scalar.len_utf8();
            }
            boundaries.push((rebuilt.len(), offset));
        }
        (rebuilt == decoded).then_some(Self {
            start: span.start,
            boundaries: Some(boundaries),
        })
    }

    pub fn offset(&self, decoded: u32) -> Option<u32> {
        let local = if let Some(boundaries) = &self.boundaries {
            let index = boundaries
                .binary_search_by_key(&(decoded as usize), |&(at, _)| at)
                .ok()?;
            u32::try_from(boundaries[index].1).ok()?
        } else {
            decoded
        };
        self.start.checked_add(local)
    }
}

#[test]
fn byte_ranges_cover_whole_references_and_respect_their_html_context() {
    let raw = "π:a &amp;&amp; b";
    let map = Coordinates::from_folio(raw, Span::new(3, 17), "a && b", EntityContext::Attribute)
        .expect("decoded");
    assert_eq!((map.offset(0), map.offset(6)), (Some(3), Some(17)));
    let multi = "&NotEqualTilde;";
    let map = Coordinates::from_folio(multi, Span::new(0, 15), "≂\u{338}", EntityContext::Text)
        .expect("two scalars");
    assert_eq!(map.offset(3), None);
    assert_eq!(map.offset(5), Some(15));
    let legacy = "&notit;";
    assert!(
        Coordinates::from_folio(legacy, Span::new(0, 7), "¬it;", EntityContext::Text).is_some()
    );
    assert!(
        Coordinates::from_folio(legacy, Span::new(0, 7), "¬it;", EntityContext::Attribute)
            .is_none()
    );
}

#[test]
fn historical_html_expressions_keep_the_authored_range_of_each_operator() {
    let fixture = include_str!(
        "../../../../tests/_fixtures/differential/compiler/expression-character-references/Entities.vue"
    );
    let mut report = super::runner::Report::default();
    super::runner::run_source(fixture, "Entities.vue", &mut report);
    report
        .verdict("original character references")
        .expect("all original authored spans agree");
    let source = r#"<i :value="label === 'p-emptylabel' ? '&nbsp;' : (label ?? 'empty')" />"#;
    let (independent, naive) = super::runner::both_template(source);
    assert_eq!(independent, naive);
    let row = naive
        .rows
        .iter()
        .find(|row| row.kind == "logical")
        .expect("coalescing decision");
    assert_eq!(
        source.get(row.start as usize..row.end as usize),
        Some("label ?? 'empty'")
    );
}
