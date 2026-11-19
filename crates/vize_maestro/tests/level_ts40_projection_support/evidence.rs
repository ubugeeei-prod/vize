//! Authenticate the complete packets behind the reviewed #10933 transition.
mod authored_component_infer;
mod constructor_noinfer_return;
mod mixed_vue2;
mod receipt;
mod shared_listener_args;
mod unowned_owned_keys;
mod verbatim_runs;
mod with_defaults_undefined;

use super::{capture_fixture, load_matrix};
use receipt::{authenticate_join_receipt, authenticate_packets, authenticate_successor_receipt};

#[test]
fn public_instance_slot_packets_remain_authenticated() {
    authenticate_packets(
        "public-instance-options-api-slots",
        include_str!("fixtures/public-instance-options-api-slots/manifest.json"),
        194,
        &[
            "options-api-slots/maestro/authored-hits.txt",
            "options-api-slots/maestro/mappings.txt",
            "options-api-slots/maestro/text.txt",
            "options-api-slots/maestro-document-0/text.ts",
        ],
        14,
    );
    let historical_receipt =
        include_str!("fixtures/public-instance-options-api-slots/successor-11064/receipt.json");
    let predecessor =
        include_str!("fixtures/public-instance-options-api-slots/after/projection-record.txt");
    let historical_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11064/projection-record.txt"
    );
    authenticate_successor_receipt(
        historical_receipt,
        "466b14fde3af6e68fa2131c9c0e474b548bbe1e1cf4004c3b71da1ca2da6629a",
        11064,
        predecessor,
        historical_successor,
    );
    let previous_receipt =
        include_str!("fixtures/public-instance-options-api-slots/successor-11069/receipt.json");
    let previous_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11069/projection-record.txt"
    );
    authenticate_successor_receipt(
        previous_receipt,
        "e70a0df8738650436f8492f8a9b94cd1f498ddd6f1e21aacea0b63d35643fed2",
        11069,
        historical_successor,
        previous_successor,
    );
    let current_receipt =
        include_str!("fixtures/public-instance-options-api-slots/successor-11142/receipt.json");
    let current_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11142/projection-record.txt"
    );
    authenticate_successor_receipt(
        current_receipt,
        "a59917f2d91969c7085eaaa550a055fdb2084ec0106a8db87f6671725764eeb1",
        11142,
        previous_successor,
        current_successor,
    );
    let latest_receipt =
        include_str!("fixtures/public-instance-options-api-slots/successor-11142-2/receipt.json");
    let latest_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11142-2/projection-record.txt"
    );
    authenticate_successor_receipt(
        latest_receipt,
        "ddc538fbfa8bdb52dc5269f145487edce7db8030429e6f312ff2b4532bf5cae6",
        11142,
        current_successor,
        latest_successor,
    );
    let exact_receipt =
        include_str!("fixtures/public-instance-options-api-slots/successor-11142-3/receipt.json");
    let exact_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11142-3/projection-record.txt"
    );
    authenticate_successor_receipt(
        exact_receipt,
        "f17f60f00ceff278d19e0e29c01b774903c376588765975ebc58759f6c7e6b76",
        11142,
        latest_successor,
        exact_successor,
    );
    let external_receipt =
        include_str!("fixtures/public-instance-options-api-slots/successor-11148/receipt.json");
    let external_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11148/projection-record.txt"
    );
    authenticate_successor_receipt(
        external_receipt,
        "0ef87961d1b384269fdd2670c41092a58bb6d460831d321fc8980735cab8e546",
        11148,
        exact_successor,
        external_successor,
    );
    let collision_safe_receipt =
        include_str!("fixtures/public-instance-options-api-slots/successor-11153/receipt.json");
    let collision_safe_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11153/projection-record.txt"
    );
    authenticate_successor_receipt(
        collision_safe_receipt,
        "cd58fde4a43fbb181c31f21f2b2c56d80dcd504b943e2370350b1db5894474f0",
        11153,
        external_successor,
        collision_safe_successor,
    );
    let public_base_receipt =
        include_str!("fixtures/public-instance-options-api-slots/successor-11445/receipt.json");
    let public_base_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11445/projection-record.txt"
    );
    authenticate_successor_receipt(
        public_base_receipt,
        "0d499b84efc38a60a2f7563c710403d2c002fa67cbfa8ebeb896a2344582c5ed",
        11476,
        collision_safe_successor,
        public_base_successor,
    );
    let extra_keys_receipt =
        include_str!("fixtures/public-instance-options-api-slots/successor-11591/receipt.json");
    let extra_keys_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11591/projection-record.txt"
    );
    authenticate_successor_receipt(
        extra_keys_receipt,
        "852676bbcbe713b9f32c1526eff2a6e150e578a45b0409dd96bcd6bd3c34bf85",
        11591,
        public_base_successor,
        extra_keys_successor,
    );
    let input_channels_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11629/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!("fixtures/public-instance-options-api-slots/successor-11629/receipt.json"),
        "7b8186b0029f05fd3d344f6e9f7f195baeec37e27c5e31a2a9bc2d95b05af55f",
        11629,
        extra_keys_successor,
        input_channels_successor,
    );
    let jsx_witness_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11629-2/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!("fixtures/public-instance-options-api-slots/successor-11629-2/receipt.json"),
        "1d56f8a2a9742bcb802ca7998524b3bfb621d376745fc5fa7a448b66e301823b",
        11629,
        input_channels_successor,
        jsx_witness_successor,
    );
    let runtime_prop_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11734/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!("fixtures/public-instance-options-api-slots/successor-11734/receipt.json"),
        "e6fa24941c0f5fdb1dacac5047c36c0ed42153d1a2bba49ff3358df5ee67111e",
        11738,
        extra_keys_successor,
        runtime_prop_successor,
    );
    let delimiter_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-11675-main-11863/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "fixtures/public-instance-options-api-slots/successor-11675-main-11863/receipt.json"
        ),
        "554080a49a1536daa454d8f63f5fff978bae5c04afd39db111afd89ee48dc1bd",
        11863,
        runtime_prop_successor,
        delimiter_successor,
    );
    let owned_constructor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-owned-constructor-20261105/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "fixtures/public-instance-options-api-slots/successor-owned-constructor-20261105/receipt.json"
        ),
        "735a3ed8171447dadc518475361c82c49640cabb87ba027ac826dadbf823f5d9",
        11919,
        delimiter_successor,
        owned_constructor,
    );
    let integrated_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-integrated-11964/projection-record.txt"
    );
    authenticate_join_receipt(
        include_str!(
            "fixtures/public-instance-options-api-slots/successor-integrated-11964/receipt.json"
        ),
        "6f0161e35773a36403ad3e78835828adb0c03183d08041977d234d12738461c3",
        owned_constructor,
        jsx_witness_successor,
        integrated_successor,
        "crates/vize_maestro/tests/snapshots/level_ts40_projection__level_ts40__options-api-slots__default.snap.new",
        "fb29a6ea745462c954eb0e7b0b4c008b4c1d5ec8379ec99400ddacaf075e779f",
    );
    let discriminated_jsx_union_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-discriminated-jsx-union-20261106/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "fixtures/public-instance-options-api-slots/successor-discriminated-jsx-union-20261106/receipt.json"
        ),
        "cd87038c80f50f21ef62dd90bdc543fcc9236c2ebd128a9c7906c893cf8b0f28",
        11964,
        integrated_successor,
        discriminated_jsx_union_successor,
    );
    let strict_extra_keys_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-strict-extra-keys-11497/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "fixtures/public-instance-options-api-slots/successor-strict-extra-keys-11497/receipt.json"
        ),
        "438edec886bb89765617eda8e91ff10b9c8da08bbb2568396fd0da2b12e2553f",
        12025,
        discriminated_jsx_union_successor,
        strict_extra_keys_successor,
    );
    let canon_leftovers_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-canon-leftovers-11499/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "fixtures/public-instance-options-api-slots/successor-canon-leftovers-11499/receipt.json"
        ),
        "7c511bcec1bc789a76d62c6ce3dad496f623e4efd1f54cd578098f8ecef19275",
        12162,
        strict_extra_keys_successor,
        canon_leftovers_successor,
    );
    let instance_input_keys_successor = include_str!(
        "fixtures/public-instance-options-api-slots/successor-instance-input-keys-11654/projection-record.txt"
    );
    authenticate_successor_receipt(
        include_str!(
            "fixtures/public-instance-options-api-slots/successor-instance-input-keys-11654/receipt.json"
        ),
        "706ec3664a8abd2b64c85eb0ef9bdbae53f20c677ab0533068ea5e6633e22f83",
        12446,
        canon_leftovers_successor,
        instance_input_keys_successor,
    );
    let unowned_successor = unowned_owned_keys::slots(instance_input_keys_successor);
    let non_text_fields = |record: &str| {
        record
            .lines()
            .filter(|line| {
                !line.starts_with("text=")
                    && !line.starts_with("pre-rewrite-text=")
                    && !line.starts_with("import-source-map-probes=")
            })
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    assert_eq!(
        non_text_fields(delimiter_successor),
        non_text_fields(owned_constructor)
    );
    let matrix = load_matrix();
    let fixture = matrix
        .fixtures
        .iter()
        .find(|fixture| fixture.id == "options-api-slots")
        .expect("reviewed Options API slot fixture");
    assert_eq!(
        capture_fixture(fixture).render().as_str(),
        with_defaults_undefined::slots(unowned_successor)
    );
}
