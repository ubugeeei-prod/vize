use super::parse;
use vize_l0::Allocator;
use vize_relief::errors::ErrorCode;

#[test]
fn duplicate_if_directive_is_a_parse_error() {
    let allocator = Allocator::new();
    let (_, errors) = parse(&allocator, r#"<p v-if="ok" v-if="!ok">x</p>"#);
    assert_eq!(
        errors
            .iter()
            .filter(|error| error.code == ErrorCode::DuplicateAttribute)
            .count(),
        1
    );
}
