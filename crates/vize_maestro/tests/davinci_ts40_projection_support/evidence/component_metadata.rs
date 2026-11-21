//! Component export and constructor projections for #13430.

use super::receipt::authenticate_successor_receipt;

pub(super) fn slots(predecessor: &'static str) -> &'static str {
    let predecessor = super::with_defaults_undefined::slots(predecessor);
    let successor = include_str!(
        "../fixtures/public-instance-options-api-slots/successor-component-metadata-13430/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-options-api-slots/successor-component-metadata-13430/receipt.json"
        ),
        "9dbf9ad211f7f6d7c69207cebc5e099110ad7b8233e29427a3f23f401c2f7294",
        13436,
        predecessor,
        successor,
    );
    successor
}

pub(super) fn mixed_vue2(predecessor: &'static str) -> &'static str {
    let predecessor = super::with_defaults_undefined::mixed_vue2(predecessor);
    let successor = include_str!(
        "../fixtures/public-instance-mixed-vue2/successor-component-metadata-13430/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-component-metadata-13430/receipt.json"
        ),
        "2d886bfbc5ce0ee883f17ed22fd403ef7a47d3b4acb397274cc75a2c2c38cf8b",
        13436,
        predecessor,
        successor,
    );
    successor
}
