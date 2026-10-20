use super::{
    is_shared_helpers_target, save_virtual_ts_for_path, user_facing_type_message,
    virtual_ts_save_path,
};

#[test]
fn missing_required_prop_hides_setup_props_type() {
    let message = "Argument of type '{ items: number[]; }' is not assignable to parameter of type '__VizeSetupProps<{ readonly headers: string[]; readonly items: number[]; }, __VizePublicComponentAttrs & { zoomAndPan?: string; }>'.\nProperty 'headers' is missing in type '{ items: number[]; }' but required in type '{ readonly headers: string[]; readonly items: number[]; }'.";

    assert_eq!(
        user_facing_type_message(true, message),
        "Missing required prop `headers`."
    );
}

#[test]
fn missing_required_prop_hides_component_check_props_type() {
    let message = "Argument of type '{}' is not assignable to parameter of type '__VizeComponentCheckProps<{ readonly text: string; }, __VizePublicComponentAttrs>'.\nProperty 'text' is missing in type '{}' but required in type '{ readonly text: string; }'.";

    assert_eq!(
        user_facing_type_message(true, message),
        "Missing required prop `text`."
    );
}

#[test]
fn type_message_simplification_is_narrow() {
    let ordinary_message =
        "Property 'headers' is missing in type '{}' but required in type '{ headers: string[]; }'.";
    let plural_message = "Argument of type '{}' is not assignable to parameter of type '__VizeComponentCheckProps<{}>'. Type '{}' is missing the following properties: headers, items";

    assert_eq!(
        user_facing_type_message(true, ordinary_message),
        ordinary_message
    );
    assert_eq!(
        user_facing_type_message(
            false,
            "__VizeSetupProps Property 'headers' is missing in type {}"
        ),
        "__VizeSetupProps Property 'headers' is missing in type {}"
    );
    assert_eq!(
        user_facing_type_message(true, plural_message),
        plural_message
    );
}

#[test]
fn virtual_ts_save_path_appends_virtual_ts_after_full_file_name() {
    let path = std::path::Path::new("/workspace/src/Hoge.vue");

    assert_eq!(
        virtual_ts_save_path(path).unwrap(),
        std::path::PathBuf::from("/workspace/src/Hoge.vue.virtual.ts")
    );
}

#[test]
fn save_virtual_ts_for_path_writes_matching_canonical_file() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("src").join("Hoge.vue");
    std::fs::create_dir_all(source.parent().unwrap()).unwrap();
    std::fs::write(&source, "<template />").unwrap();
    let canonical_source = source.canonicalize().unwrap();

    let saved = save_virtual_ts_for_path(
        std::path::Path::new("src/Hoge.vue"),
        temp.path(),
        [(canonical_source.as_path(), "const value = 1;\n")],
    )
    .unwrap();

    assert_eq!(
        saved,
        canonical_source.with_file_name("Hoge.vue.virtual.ts")
    );
    assert_eq!(
        std::fs::read_to_string(saved).unwrap(),
        "const value = 1;\n"
    );
}

#[test]
fn is_shared_helpers_target_matches_helpers_file_name_in_any_form() {
    let bare = std::path::Path::new(vize_canon::virtual_ts::SHARED_PREAMBLE_FILE_NAME);
    assert!(is_shared_helpers_target(bare));
    assert!(is_shared_helpers_target(std::path::Path::new(
        "some/nested/dir/__vize_helpers.d.ts"
    )));
    assert!(is_shared_helpers_target(std::path::Path::new(
        "/abs/__vize_helpers.d.ts"
    )));
    assert!(!is_shared_helpers_target(std::path::Path::new(
        "src/App.vue"
    )));
    assert!(!is_shared_helpers_target(std::path::Path::new(
        "__vize_helpers.ts"
    )));
}

#[test]
fn save_virtual_ts_for_path_writes_shared_helpers_preamble() {
    let temp = tempfile::tempdir().unwrap();

    // No `.vue` candidate matches the helpers target; it is served from the
    // shared preamble constant instead of the per-file virtual modules.
    let saved = save_virtual_ts_for_path(
        std::path::Path::new(vize_canon::virtual_ts::SHARED_PREAMBLE_FILE_NAME),
        temp.path(),
        std::iter::empty(),
    )
    .unwrap();

    assert_eq!(
        saved,
        temp.path()
            .join(vize_canon::virtual_ts::SHARED_PREAMBLE_FILE_NAME)
    );
    assert_eq!(
        std::fs::read_to_string(saved).unwrap(),
        vize_canon::virtual_ts::SHARED_PREAMBLE_DTS
    );
}

#[test]
fn save_virtual_ts_for_path_writes_shared_helpers_to_absolute_target() {
    let temp = tempfile::tempdir().unwrap();
    let target = temp.path().join("out").join("__vize_helpers.d.ts");
    std::fs::create_dir_all(target.parent().unwrap()).unwrap();

    // An unrelated cwd must not redirect an absolute helpers target.
    let saved = save_virtual_ts_for_path(
        &target,
        std::path::Path::new("/nonexistent"),
        std::iter::empty(),
    )
    .unwrap();

    assert_eq!(saved, target);
    assert_eq!(
        std::fs::read_to_string(saved).unwrap(),
        vize_canon::virtual_ts::SHARED_PREAMBLE_DTS
    );
}
