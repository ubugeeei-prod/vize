//! The editor projection leaves Options API props unprojected; those props are
//! unknown, not absent, so the published `Props` contract stays open.

use super::receipt::authenticate_successor_receipt;

pub(super) fn slots(predecessor: &'static str) -> &'static str {
    let predecessor = super::component_metadata::slots(predecessor);
    let successor = include_str!(
        "../fixtures/public-instance-options-api-slots/successor-unextracted-options-props-13693/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-options-api-slots/successor-unextracted-options-props-13693/receipt.json"
        ),
        "6e473bae4a2db794df2ee5cf8bc9c27c41bc2a5828798cba03a34aa082e503e9",
        13693,
        predecessor,
        successor,
    );
    successor
}
