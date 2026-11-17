//! #13113: the content mapper binds an authored default export by inference,
//! on top of the #12789 and #12793 successors.

use super::receipt::authenticate_successor_receipt;

pub(super) fn slots(predecessor: &'static str) -> &'static str {
    let predecessor = super::verbatim_runs::slots(predecessor);
    let successor = include_str!(
        "../fixtures/public-instance-options-api-slots/successor-authored-component-infer-13113/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-options-api-slots/successor-authored-component-infer-13113/receipt.json"
        ),
        "e0f4b77cc2bc6a723c180888d30423e6ce4ff3d48b5ee015339465c4084badbc",
        13113,
        predecessor,
        successor,
    );
    successor
}

pub(super) fn mixed_vue2(predecessor: &'static str) -> &'static str {
    let predecessor = super::shared_listener_args::mixed_vue2(predecessor);
    let successor = include_str!(
        "../fixtures/public-instance-mixed-vue2/successor-authored-component-infer-13113/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-authored-component-infer-13113/receipt.json"
        ),
        "be63836c7a263cf7ecb1c0157fa47512240918d2f019d09a2e4704286005516d",
        13113,
        predecessor,
        successor,
    );
    successor
}
