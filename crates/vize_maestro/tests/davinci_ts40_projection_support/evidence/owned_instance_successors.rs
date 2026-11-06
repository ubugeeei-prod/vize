//! Authenticate the #11984 owned-field and dead-alias successor packets.

use super::receipt::authenticate_successor_receipt;

pub(super) fn options_api(previous: &str) -> &'static str {
    let owned = include_str!(
        "../fixtures/public-instance-options-api-slots/successor-owned-instance-11984/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-options-api-slots/successor-owned-instance-11984/receipt.json"
        ),
        "67aab9c65deb2bac25a7698ee25091d1fc1abb2d32180a65c47b4f789a900743",
        11984,
        previous,
        owned,
    );
    let elided = include_str!(
        "../fixtures/public-instance-options-api-slots/successor-elided-instance-11984/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-options-api-slots/successor-elided-instance-11984/receipt.json"
        ),
        "589f73df3491aba730bd15a6f206c87076bdf4707473097efcec5ded4160332b",
        11984,
        owned,
        elided,
    );
    elided
}

pub(super) fn mixed_vue2(previous: &str) -> &'static str {
    let owned = include_str!(
        "../fixtures/public-instance-mixed-vue2/successor-owned-instance-11984/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-owned-instance-11984/receipt.json"
        ),
        "5aba53760a798a8ae3d55fc78272781401fe8d380f8c2a65ccb5e5fe19b269db",
        11984,
        previous,
        owned,
    );
    let elided = include_str!(
        "../fixtures/public-instance-mixed-vue2/successor-elided-instance-11984/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-elided-instance-11984/receipt.json"
        ),
        "8ec79ed5636f31a3d0d787f4b9577de01b15fab7efaa50e32b33a5fbe145a29c",
        11984,
        owned,
        elided,
    );
    elided
}
