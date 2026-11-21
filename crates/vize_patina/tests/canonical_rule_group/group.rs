use super::*;

pub(super) fn requested(
    lint: &NativeCanonicalLint<'_, '_>,
    messages: &impl MessageLookup,
) -> (Answer, Answer) {
    let result = lint.accessibility(messages);
    (result.tabindex_no_positive, result.img_alt)
}
