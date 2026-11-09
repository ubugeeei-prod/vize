//! #12467: `$UnownedComponentAttrs` computes its owned key set once.

use super::receipt::authenticate_successor_receipt;

pub(super) fn slots(predecessor: &'static str) -> &'static str {
    let successor = include_str!(
        "../fixtures/public-instance-options-api-slots/successor-unowned-owned-keys-11654/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-options-api-slots/successor-unowned-owned-keys-11654/receipt.json"
        ),
        "b727422eef7660b2530778e0fad2311fa62e9ae71ec9b1a46ade805c6f3ecd80",
        12467,
        predecessor,
        successor,
    );
    successor
}

pub(super) fn mixed_vue2(predecessor: &'static str) -> &'static str {
    let successor = include_str!(
        "../fixtures/public-instance-mixed-vue2/successor-unowned-owned-keys-11654/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-unowned-owned-keys-11654/receipt.json"
        ),
        "893ea8990266fcb47e560f21845cf9697243f38eddbed539679a1832eb0d1157",
        12467,
        predecessor,
        successor,
    );
    successor
}
