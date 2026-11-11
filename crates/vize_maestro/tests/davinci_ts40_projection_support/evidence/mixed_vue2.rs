//! The mixed Vue 2 public-instance packets behind the reviewed #10933 transition.
use super::super::{capture_fixture, load_matrix, normalize::sha256};
use super::receipt::{
    authenticate_join_receipt, authenticate_packets, authenticate_successor_receipt,
};
use super::{shared_listener_args, unowned_owned_keys};

#[test]
fn public_instance_mixed_vue2_packets_remain_authenticated() {
    authenticate_packets(
        "public-instance-mixed-vue2",
        include_str!("../fixtures/public-instance-mixed-vue2/manifest.json"),
        205,
        &[
            "options-api-slots/maestro/authored-hits.txt",
            "options-api-slots/maestro/mappings.txt",
            "options-api-slots/maestro/text.txt",
            "options-api-slots/maestro-document-0/text.ts",
            "vue2-native-event/maestro/authored-hits.txt",
            "vue2-native-event/maestro/mappings.txt",
            "vue2-native-event/maestro/text.txt",
            "vue2-native-event/maestro-document-0/text.ts",
        ],
        36,
    );
    if cfg!(feature = "legacy") {
        let matrix = load_matrix();
        let fixture = matrix
            .fixtures
            .iter()
            .find(|fixture| fixture.id == "vue2-native-event")
            .expect("reviewed mixed Vue 2 fixture");
        assert_eq!((fixture.options_api, fixture.legacy_vue2), (true, true));
        // Canon opts into Vue 2; Maestro's existing generator uses Vue 3
        // defaults. The feature enables this mixed fixture's capture only.
        let predecessor =
            include_str!("../fixtures/public-instance-mixed-vue2/after/projection-record.txt");
        let successor = include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-11069/projection-record.txt"
        );
        authenticate_successor_receipt(
            include_str!("../fixtures/public-instance-mixed-vue2/successor-11069/receipt.json"),
            "3c486d9b75a1a25cc6e1815fb478f00a89334c48627621ab552e47e0e05d6330",
            11069,
            predecessor,
            successor,
        );
        let latest_successor = include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-11142/projection-record.txt"
        );
        authenticate_successor_receipt(
            include_str!("../fixtures/public-instance-mixed-vue2/successor-11142/receipt.json"),
            "ac91a6f06d1d22359daf89c5ccea8076a60bd9ab637951ccfd76f0fb45cbf253",
            11142,
            successor,
            latest_successor,
        );
        let collision_safe_successor = include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-11153/projection-record.txt"
        );
        authenticate_successor_receipt(
            include_str!("../fixtures/public-instance-mixed-vue2/successor-11153/receipt.json"),
            "84c1c49f0874cb1cb0b3d5227af293e378cfd72be2957232c46dbed7a38aa00e",
            11153,
            latest_successor,
            collision_safe_successor,
        );
        let public_base_successor = include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-11445/projection-record.txt"
        );
        authenticate_successor_receipt(
            include_str!("../fixtures/public-instance-mixed-vue2/successor-11445/receipt.json"),
            "84c692e0e53b990b9880d021c9800590b5ca1eac08d13ff0fa9b0aa20fae12c1",
            11476,
            collision_safe_successor,
            public_base_successor,
        );
        let extra_keys_successor = include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-11591/projection-record.txt"
        );
        authenticate_successor_receipt(
            include_str!("../fixtures/public-instance-mixed-vue2/successor-11591/receipt.json"),
            "93b07465f7826eba813804389c5dbd2aae0677e630f1c31b672b5332e39e4deb",
            11591,
            public_base_successor,
            extra_keys_successor,
        );
        let input_channels_successor = include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-11629/projection-record.txt"
        );
        authenticate_successor_receipt(
            include_str!("../fixtures/public-instance-mixed-vue2/successor-11629/receipt.json"),
            "180caef24d393a002684eb624d2284360555e147f178e197f224e437e3e54987",
            11629,
            extra_keys_successor,
            input_channels_successor,
        );
        let jsx_witness_successor = include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-11629-2/projection-record.txt"
        );
        assert_eq!(
            sha256(include_str!(
                "../fixtures/public-instance-mixed-vue2/successor-11629-2/receipt.json"
            )),
            "85782da294ba5f59f1a173ccc28b4955fa801feb669cd0b7bfe77a95b4296129"
        );
        authenticate_successor_receipt(
            include_str!(
                "../fixtures/public-instance-mixed-vue2/successor-11629-2/receipt-integrated.json"
            ),
            "2b8d918dc0a08559fbe155f5b67d1b54f84a6bf33ea3e1f33f7d3e1d0ad38545",
            11629,
            input_channels_successor,
            jsx_witness_successor,
        );
        let runtime_prop_successor = include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-11734/projection-record.txt"
        );
        authenticate_successor_receipt(
            include_str!("../fixtures/public-instance-mixed-vue2/successor-11734/receipt.json"),
            "c3a66cc34914185adf87def1cb1d0714158ac78b780a479dc65a0d425ed185a3",
            11738,
            extra_keys_successor,
            runtime_prop_successor,
        );

        let delimiter_successor = include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-11675-main-11863/projection-record.txt"
        );
        authenticate_successor_receipt(
            include_str!(
                "../fixtures/public-instance-mixed-vue2/successor-11675-main-11863/receipt.json"
            ),
            "7c9503a77fccad883c99097ee0af7bec02984d02a4a81627b9d0e6ce056254c5",
            11863,
            runtime_prop_successor,
            delimiter_successor,
        );

        let owned_constructor = include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-owned-constructor-20261105/projection-record.txt"
        );
        authenticate_successor_receipt(
            include_str!(
                "../fixtures/public-instance-mixed-vue2/successor-owned-constructor-20261105/receipt.json"
            ),
            "c05d7950767445ae627b602eb5292418530e9b54f3a5613314c168ea63321a5b",
            11919,
            delimiter_successor,
            owned_constructor,
        );
        let integrated_successor = include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-integrated-11964/projection-record.txt"
        );
        authenticate_join_receipt(
            include_str!(
                "../fixtures/public-instance-mixed-vue2/successor-integrated-11964/receipt.json"
            ),
            "fb360f0ff44b0c2086dc2844ed368b231dfc2e499a82c056f2f4f074fd075795",
            owned_constructor,
            jsx_witness_successor,
            integrated_successor,
            "crates/vize_maestro/tests/snapshots/davinci_ts40_projection__davinci_ts40__vue2-native-event__mixed-vue2__legacy-generators-enabled.snap.new",
            "ce519376b1c12c5a51cb8952061ffdd3e581745f883ef50d049609552af56281",
        );
        let discriminated_jsx_union_successor = include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-discriminated-jsx-union-20261106/projection-record.txt"
        );
        authenticate_successor_receipt(
            include_str!(
                "../fixtures/public-instance-mixed-vue2/successor-discriminated-jsx-union-20261106/receipt.json"
            ),
            "f350590d07f4f46c9089001257916e011619b02d9aa75d018ea481d883b22319",
            11964,
            integrated_successor,
            discriminated_jsx_union_successor,
        );
        let strict_extra_keys_successor = include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-strict-extra-keys-11497/projection-record.txt"
        );
        authenticate_successor_receipt(
            include_str!(
                "../fixtures/public-instance-mixed-vue2/successor-strict-extra-keys-11497/receipt.json"
            ),
            "d0abf13f2c818c9ff6f907fb7469446159f9608fb6900911ef75095ac899fffa",
            12025,
            discriminated_jsx_union_successor,
            strict_extra_keys_successor,
        );
        let canon_leftovers_successor = include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-canon-leftovers-11499/projection-record.txt"
        );
        authenticate_successor_receipt(
            include_str!(
                "../fixtures/public-instance-mixed-vue2/successor-canon-leftovers-11499/receipt.json"
            ),
            "f485163bf0dcc8bc395fa97edf179375369f1c9c44c6b5f0285709b5d05c6e94",
            12162,
            strict_extra_keys_successor,
            canon_leftovers_successor,
        );
        let instance_input_keys_successor = include_str!(
            "../fixtures/public-instance-mixed-vue2/successor-instance-input-keys-11654/projection-record.txt"
        );
        authenticate_successor_receipt(
            include_str!(
                "../fixtures/public-instance-mixed-vue2/successor-instance-input-keys-11654/receipt.json"
            ),
            "14246b9544ab2efc798030e183ee666b95395ca695ea48efd31af37acfbd616e",
            12446,
            canon_leftovers_successor,
            instance_input_keys_successor,
        );
        let unowned_successor = unowned_owned_keys::mixed_vue2(instance_input_keys_successor);
        let non_text_fields = |record: &str| {
            record
                .lines()
                .filter(|line| !line.starts_with("text="))
                .map(str::to_owned)
                .collect::<Vec<_>>()
        };
        assert_eq!(
            non_text_fields(delimiter_successor),
            non_text_fields(owned_constructor)
        );
        assert_eq!(
            capture_fixture(fixture).render().as_str(),
            shared_listener_args::mixed_vue2(unowned_successor)
        );
    }
}
