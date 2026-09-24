use vize_s0::Allocator;

use super::{PugErrorCode, check_pug_fidelity, parse_pug_within};

#[test]
fn an_oversized_source_is_one_eof_hole_with_a_size_error() {
    let allocator = Allocator::default();
    let source = "p hello";
    let (tree, errors) = parse_pug_within(&allocator, source, 3);
    assert!(tree.nodes.is_empty());
    assert_eq!(tree.eof.leading, source);
    assert_eq!(check_pug_fidelity(&tree), Ok(()));
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].code, PugErrorCode::SourceTooLarge);
    assert_eq!(errors[0].offset, 0);
}

#[test]
fn a_source_at_the_limit_parses_normally() {
    let allocator = Allocator::default();
    let source = "p hello";
    let (tree, errors) = parse_pug_within(&allocator, source, source.len());
    assert_eq!(tree.nodes.len(), 1);
    assert!(errors.is_empty());
}
