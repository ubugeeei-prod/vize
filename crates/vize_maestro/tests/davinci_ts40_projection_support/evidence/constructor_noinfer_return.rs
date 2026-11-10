//! #12523: the component constructor keeps its bound instance behind `NoInfer`,
//! on top of the #12481 verbatim setup successor.

mod setup_verbatim_lines;

use super::receipt::authenticate_successor_receipt;

pub(super) fn slots(predecessor: &'static str) -> &'static str {
    let predecessor = setup_verbatim_lines::slots(predecessor);
    let constructor_successor = include_str!(
        "../fixtures/public-instance-options-api-slots/successor-constructor-noinfer-12431/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-options-api-slots/successor-constructor-noinfer-12431/receipt.json"
        ),
        "105d55a7901e17193c5f1c6073e77ea38076588d243b903b895ef1dbeb39cfa8",
        12523,
        predecessor,
        constructor_successor,
    );

    let successor = include_str!(
        "../fixtures/public-instance-options-api-slots/successor-with-defaults-docs-12461/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-options-api-slots/successor-with-defaults-docs-12461/receipt.json"
        ),
        "20631db8fe59ecc81c9dfe1f91bc6fffadeee792b52de6ae625c83f28ca85a8f",
        12614,
        constructor_successor,
        successor,
    );
    successor
}

pub(super) fn mixed_vue2(predecessor: &'static str) -> &'static str {
    let predecessor = setup_verbatim_lines::mixed_vue2(predecessor);
    let constructor_successor = include_str!(
        "../fixtures/public-instance-mixed-vue2/successor-constructor-noinfer-12431/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-constructor-noinfer-12431/receipt.json"
        ),
        "f562cc5e4db5e573068720120e68ab34525f8d5bb0e5d69f01d06123e7abfe9a",
        12523,
        predecessor,
        constructor_successor,
    );

    let successor = include_str!(
        "../fixtures/public-instance-mixed-vue2/successor-with-defaults-docs-12461/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-with-defaults-docs-12461/receipt.json"
        ),
        "867958376021429cd36a17c0f8bf39b34ceb058e8b00a29ce2925ad9752e8aba",
        12614,
        constructor_successor,
        successor,
    );
    successor
}
