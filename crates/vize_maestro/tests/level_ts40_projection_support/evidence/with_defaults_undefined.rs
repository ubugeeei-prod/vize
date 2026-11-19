//! #13179: `$DefaultNativeType` gains `undefined`, on top of the #13113
//! successors.

use super::receipt::authenticate_successor_receipt;

pub(super) fn slots(predecessor: &'static str) -> &'static str {
    let predecessor = super::authored_component_infer::slots(predecessor);
    let successor = include_str!(
        "../fixtures/public-instance-options-api-slots/successor-with-defaults-undefined-13179/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-options-api-slots/successor-with-defaults-undefined-13179/receipt.json"
        ),
        "2e9cfdb5bbe1415d773945b4a8a674341b0af7d35f341caeceb8fceb8b56a765",
        13179,
        predecessor,
        successor,
    );
    successor
}

pub(super) fn mixed_vue2(predecessor: &'static str) -> &'static str {
    let predecessor = super::authored_component_infer::mixed_vue2(predecessor);
    let successor = include_str!(
        "../fixtures/public-instance-mixed-vue2/successor-with-defaults-undefined-13179/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-with-defaults-undefined-13179/receipt.json"
        ),
        "7aca83c77736a63e32029ef8cda0d43bd88697906919c0cfd272d33995f02d92",
        13179,
        predecessor,
        successor,
    );
    successor
}
