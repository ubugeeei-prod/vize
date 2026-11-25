//! External-consumer coverage for the compatibility-preview template facade.

#![expect(
    deprecated,
    reason = "the supported old names remain usable for one minor release"
)]

fn through_both_old_facades<'a>(
    value: vize_patina::L2Template<'a>,
) -> vize_patina::LintL2Template<'a> {
    let value: vize_patina::markup::L2Template<'a> = value;
    value
}

#[test]
fn the_deprecated_template_facades_retain_the_canonical_owned_type() {
    let _: for<'a> fn(vize_patina::LintL2Template<'a>) -> vize_patina::LintL2Template<'a> =
        through_both_old_facades;
}
