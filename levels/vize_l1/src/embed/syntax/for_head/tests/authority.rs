use super::{EmbedHole, ForHeadHole, ForHeadPart, Lang, parse};
use vize_l0::{Allocator, Span};

#[test]
fn dense_handoff_preserves_both_owners_on_actual_strict_formal_refusals() {
    let allocator = Allocator::default();
    for lang in [Lang::Js, Lang::Ts] {
        for spelling in ["arguments", "eval", "yield", "\\u0065val", "\\u0079ield"] {
            let text = vize_l0::cstr!("{} /*alias*/ in items /*collection*/", spelling);
            let head = parse(&allocator, &text, lang);
            assert_eq!(
                head.hole(),
                Some(ForHeadHole::AliasesUnavailable(
                    EmbedHole::InvalidParameterContext
                ))
            );
            assert_eq!(head.source().text(), text);
            assert!(head.dense().is_none());
            let aliases = head.aliases().unwrap().unwrap();
            assert!(aliases.admitted_parameters().is_none());
            assert!(aliases.parameters().is_none() && aliases.rest().is_none());
            assert_eq!(aliases.diagnostics().count(), 0);
            let comment = aliases.comments().next().unwrap();
            assert_eq!(comment.text().unwrap(), "/*alias*/");
            let start = spelling.len() as u32 + 1;
            assert_eq!(comment.authored_span(), Ok(Span::new(start, start + 9)));
            let collection = head.collection().unwrap().unwrap();
            assert_eq!(collection.hole(), None);
            assert!(collection.expression().is_some());
            assert!(collection.admitted_expression().is_some());
            assert_eq!(
                collection.comments().next().unwrap().text().unwrap(),
                "/*collection*/"
            );
            let mut comments = head.comments().map(|(part, _)| part);
            assert_eq!(comments.next(), Some(ForHeadPart::Aliases));
            assert_eq!(comments.next(), Some(ForHeadPart::Collection));
            assert_eq!(comments.next(), None);
        }
    }
}

#[test]
fn generic_parameter_proof_does_not_claim_dense_name_uniqueness_or_file_origin() {
    let allocator = Allocator::default();
    let head = parse(&allocator, "(item,item) in items", Lang::Js);
    assert_eq!(head.hole(), None);
    assert!(
        head.aliases()
            .unwrap()
            .unwrap()
            .admitted_parameters()
            .is_some()
    );
    assert_eq!(head.dense().unwrap().parameters().len(), 2);
    // Individual binding IDs and duplicate refusal remain the canonical
    // constructor's existing one-enumeration preflight, not this syntax proof.
}
