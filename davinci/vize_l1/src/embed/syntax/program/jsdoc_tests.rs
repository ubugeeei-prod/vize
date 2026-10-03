use super::*;
use vize_l0::Span;

#[test]
fn native_js_ts_program_retains_original_jsdoc_receipt_after_owner_move() {
    let arena = Allocator::default();
    let source = "/** @type {import('vue').Ref<number>} */ let value=1;";
    let origin = 137;
    for lang in [Lang::Js, Lang::Ts] {
        let original = parse_program_once(
            &arena,
            EmbedSource::authored(source, Span::new(origin, origin + source.len() as u32)).unwrap(),
            ProgramOptions::module(lang),
        );
        assert!(original.hole().is_none());
        let ast = original.admitted_program().unwrap().program() as *const _;
        let source_pointer = original.source().text().as_ptr();
        let moved = original;
        let admitted = moved.admitted_program().unwrap();
        assert!(admitted.has_jsdoc_comments());
        assert_eq!(admitted.program() as *const _, ast);
        assert_eq!(admitted.source().as_ptr(), source_pointer);
        assert_eq!(admitted.source(), source);
        assert_eq!(
            admitted.source_type(),
            ProgramOptions::module(lang).source_type()
        );
        assert!(core::ptr::eq(admitted.program(), moved.program().unwrap()));
        assert_eq!(moved.comments().count(), 1);
        let comment = moved.comments().next().unwrap();
        let end = source.find("*/").unwrap() as u32 + 2;
        assert_eq!(comment.decoded_span().unwrap(), Span::new(0, end));
        assert_eq!(
            comment.authored_span().unwrap(),
            Span::new(origin, origin + end)
        );
        assert_eq!(comment.text().unwrap(), &source[..end as usize]);
        assert_eq!(moved.diagnostics().count(), 0);
    }
}
