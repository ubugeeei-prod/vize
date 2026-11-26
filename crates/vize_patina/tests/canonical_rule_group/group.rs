use super::*;

pub(super) fn requested(
    lint: &NativeCanonicalLint<'_, '_>,
    messages: &impl MessageLookup,
) -> (Answer, Answer) {
    let result = lint.accessibility(messages);
    (result.tabindex_no_positive, result.img_alt)
}

pub(super) fn preorder<'s, 'a>(
    artifact: &'s vize_l2::artifact::L2Template<'a>,
    visit: &mut impl FnMut(vize_l0::id::NodeId, vize_l2::walk::NodeRef<'s, 'a>),
) -> Result<(), vize_l2::walk::NodeLimit> {
    artifact.visit_markup_preorder(visit)
}
