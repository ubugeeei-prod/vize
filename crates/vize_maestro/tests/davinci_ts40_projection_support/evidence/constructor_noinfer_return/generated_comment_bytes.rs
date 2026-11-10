//! #12621: generated TS drops its debug-only `@vize-map` lines and the
//! reasons after generated `@ts-ignore`, on top of the #12523 NoInfer successor.

use super::super::receipt::authenticate_successor_receipt;

pub(super) fn slots(predecessor: &'static str) -> &'static str {
    let successor = include_str!(
        "../../fixtures/public-instance-options-api-slots/successor-generated-comment-bytes-12621/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../../fixtures/public-instance-options-api-slots/successor-generated-comment-bytes-12621/receipt.json"
        ),
        "420b053931ec19e8f09e8db8495580370ce4057d4188d41ed2471d488bd53814",
        12621,
        predecessor,
        successor,
    );
    successor
}

pub(super) fn mixed_vue2(predecessor: &'static str) -> &'static str {
    let successor = include_str!(
        "../../fixtures/public-instance-mixed-vue2/successor-generated-comment-bytes-12621/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../../fixtures/public-instance-mixed-vue2/successor-generated-comment-bytes-12621/receipt.json"
        ),
        "aad50a19ee5ec3daf75823eeccf43263e8abf9d9f1bbd80be98428fa00153892",
        12621,
        predecessor,
        successor,
    );
    successor
}
