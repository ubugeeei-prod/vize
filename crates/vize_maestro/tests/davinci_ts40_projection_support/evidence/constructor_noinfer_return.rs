//! #12523: the component constructor keeps its bound instance behind `NoInfer`.

use super::receipt::authenticate_successor_receipt;

pub(super) fn slots(predecessor: &'static str) -> &'static str {
    let successor = include_str!(
        "../fixtures/public-instance-options-api-slots/successor-constructor-noinfer-12431/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-options-api-slots/successor-constructor-noinfer-12431/receipt.json"
        ),
        "8f1bdb3b4290aa85c4627ff0a7744eab25ced58e10d60a68b6be8c2dd4577508",
        12523,
        predecessor,
        successor,
    );
    successor
}

pub(super) fn mixed_vue2(predecessor: &'static str) -> &'static str {
    let successor = include_str!(
        "../fixtures/public-instance-mixed-vue2/successor-constructor-noinfer-12431/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-constructor-noinfer-12431/receipt.json"
        ),
        "305184ec74e1b2163daeffe8426f3c2b335ab5f7bad94df53648c0078137a30e",
        12523,
        predecessor,
        successor,
    );
    successor
}
