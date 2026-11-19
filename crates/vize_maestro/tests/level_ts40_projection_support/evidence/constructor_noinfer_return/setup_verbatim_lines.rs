//! #12481: authored setup lines are copied without added indentation.

use super::super::receipt::authenticate_successor_receipt;

pub(super) fn slots(predecessor: &'static str) -> &'static str {
    let successor = include_str!(
        "../../fixtures/public-instance-options-api-slots/successor-setup-verbatim-12481/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../../fixtures/public-instance-options-api-slots/successor-setup-verbatim-12481/receipt.json"
        ),
        "cdf81f35dd3e5a598fd7a1cf33344bab76f2b7c4d9a32f6c921ad776d74ac56d",
        12550,
        predecessor,
        successor,
    );
    successor
}

pub(super) fn mixed_vue2(predecessor: &'static str) -> &'static str {
    let successor = include_str!(
        "../../fixtures/public-instance-mixed-vue2/successor-setup-verbatim-12481/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../../fixtures/public-instance-mixed-vue2/successor-setup-verbatim-12481/receipt.json"
        ),
        "bd21f1c43958d6d6c55ed40721783e591e61d687ed30fadbf392528ce621a142",
        12550,
        predecessor,
        successor,
    );
    successor
}
