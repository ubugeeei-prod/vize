//! #12793: the shared listener-argument declaration joins every generated
//! projection that emits component event gates.

use super::receipt::authenticate_successor_receipt;

pub(super) fn mixed_vue2(predecessor: &'static str) -> &'static str {
    let predecessor = super::verbatim_runs::mixed_vue2(predecessor);
    let successor = include_str!(
        "../fixtures/public-instance-mixed-vue2/successor-shared-listener-args-12793/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-shared-listener-args-12793/receipt.json"
        ),
        "af29437a50a33ee42808f3020681c825e90212a98b7e3d4ca992db879d013121",
        12793,
        predecessor,
        successor,
    );
    successor
}
