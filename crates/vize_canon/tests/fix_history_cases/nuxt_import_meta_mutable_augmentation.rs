use crate::support::fix_history_runner::check_pack;

#[test]
fn nuxt_import_meta_augmentation_has_no_modifier_conflict() {
    check_pack(
        "nuxt-import-meta-mutable-augmentation",
        "fix_history_cases::nuxt_import_meta_mutable_augmentation::nuxt_import_meta_augmentation_has_no_modifier_conflict",
        "aa309ff7dbb11688ccaf932b99e43120fb4037e8",
        None,
        &[
            "nuxt-import-meta-client",
            "nuxt-import-meta-client-wrong-annotation",
        ],
    );
}
