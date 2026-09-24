use vize_s0::Allocator;
use vize_s1::pug::{PugError, PugErrorCode, parse_pug};

use super::{PugRendering, derive_template_with};

/// An S1 tree reporting `SourceTooLarge` derives no template: one error
/// diagnostic, empty HTML, no emitted nodes. (A 4 GiB input is impractical,
/// so the S1 error is supplied beside a small tree.)
#[test]
fn an_oversized_source_is_refused_before_emission() {
    let allocator = Allocator::default();
    let (tree, _) = parse_pug(&allocator, "p hello");
    let errors = [PugError {
        code: PugErrorCode::SourceTooLarge,
        offset: 0,
    }];
    let template = derive_template_with(&tree, &errors, PugRendering::Pug);
    assert!(template.html.is_empty());
    assert!(template.has_errors());
    assert_eq!(template.diagnostics.len(), 1);
    assert_eq!(
        template.first_error().map(|error| error.message.as_str()),
        Some(PugErrorCode::SourceTooLarge.message())
    );
}
