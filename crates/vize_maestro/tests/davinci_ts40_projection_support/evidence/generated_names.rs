//! Generated values, keys and markers are spelled `$name`
//! (`vize_l0::generated_name`), on top of the #13179 successor.

use super::receipt::authenticate_successor_receipt;

pub(super) fn slots(predecessor: &'static str) -> &'static str {
    let predecessor = super::with_defaults_undefined::slots(predecessor);
    let successor = include_str!(
        "../fixtures/public-instance-options-api-slots/successor-generated-names/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "../fixtures/public-instance-options-api-slots/successor-generated-names/receipt.json"
        ),
        "0124c649f8614f4fcf96440871b015269dec8714a879a5d59bad330356304a01",
        13230,
        predecessor,
        successor,
    );
    successor
}
