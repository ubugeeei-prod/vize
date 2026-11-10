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
        "ae29188e32454483d1fc5230cb1e5df17f76c616d12a49db05111af5de32b442",
        12789,
        predecessor,
        successor,
    );
    successor
}

pub(super) fn mixed_vue2(predecessor: &'static str) -> &'static str {
    let predecessor = super::constructor_noinfer_return::mixed_vue2(predecessor);
    let successor = include_str!(
        "../fixtures/public-instance-mixed-vue2/successor-verbatim-runs-12789/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-verbatim-runs-12789/receipt.json"
        ),
        "0532418b54a6a10e1d584d3b0f4e442b9e99c002a8b2be5436470ab5c6341cd6",
        12789,
        predecessor,
        successor,
    );
    successor
}
