//! #12789: a setup block's verbatim lines map as one span per run, on top of
//! the #12523 constructor successor.

use super::receipt::authenticate_successor_receipt;

pub(super) fn slots(predecessor: &'static str) -> &'static str {
    let predecessor = super::constructor_noinfer_return::slots(predecessor);
    let successor = include_str!(
        "../fixtures/public-instance-options-api-slots/successor-verbatim-runs-12789/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-options-api-slots/successor-verbatim-runs-12789/receipt.json"
        ),
        "4c6e5b674dfd3b3dae4a44e0831dc8fe13a5c0ebebeea9ea161a1b272e5081f3",
        12789,
        predecessor,
        successor,
    );
    successor
}
