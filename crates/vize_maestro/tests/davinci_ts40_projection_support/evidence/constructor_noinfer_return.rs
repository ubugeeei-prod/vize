//! #12523: the component constructor keeps its bound instance behind `NoInfer`,
//! on top of the #12481 verbatim setup successor.

mod setup_verbatim_lines;

use super::receipt::authenticate_successor_receipt;

pub(super) fn slots(predecessor: &'static str) -> &'static str {
    let predecessor = setup_verbatim_lines::slots(predecessor);
    let successor = include_str!(
        "../fixtures/public-instance-options-api-slots/successor-constructor-noinfer-12431/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-options-api-slots/successor-constructor-noinfer-12431/receipt.json"
        ),
        "105d55a7901e17193c5f1c6073e77ea38076588d243b903b895ef1dbeb39cfa8",
        12523,
        predecessor,
        successor,
    );
    successor
}

pub(super) fn mixed_vue2(predecessor: &'static str) -> &'static str {
    let predecessor = setup_verbatim_lines::mixed_vue2(predecessor);
    let successor = include_str!(
        "../fixtures/public-instance-mixed-vue2/successor-constructor-noinfer-12431/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-constructor-noinfer-12431/receipt.json"
        ),
        "f562cc5e4db5e573068720120e68ab34525f8d5bb0e5d69f01d06123e7abfe9a",
        12523,
        predecessor,
        successor,
    );
    successor
}
