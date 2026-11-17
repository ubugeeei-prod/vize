use crate::support::fix_history_runner::check_pack;

#[test]
fn tsx_story_accepts_sfc_class_and_style_attrs() {
    check_pack(
        "tsx-sfc-fallthrough-attrs-2086",
        "fix_history_cases::tsx_sfc_fallthrough_attrs_2086::tsx_story_accepts_sfc_class_and_style_attrs",
        "0027caad7a79b0d1a6f00b1ba68198b740f875c6",
        Some(2086),
        &["fallthrough-class-and-style", "declared-prop-wrong-literal"],
    );
}
