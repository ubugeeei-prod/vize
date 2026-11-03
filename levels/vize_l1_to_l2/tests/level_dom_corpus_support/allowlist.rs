const OLD_LANE_INVALID_FIXTURES: &[(&str, &[&str])] = &[
    (
        "crater/resources/scripts/admin/views/settings/PreferencesSetting.vue",
        &[
            "ExtendPoint",
            "InvalidEndTag",
            "InvalidEndTag",
            "InvalidEndTag",
            "MissingEndTag",
            "MissingEndTag",
        ],
    ),
    (
        "dashy/src/components/Widgets/MvgConnection.vue",
        &["VIfSameKey"],
    ),
    (
        "gogocode/packages/gogocode-vue-playground/packages/vue3/src/components/slots-unification/Comp-out.vue",
        &["VSlotDuplicateSlotNames"],
    ),
    (
        "habitica/website/client/src/components/modifyInventory.vue",
        &[
            "InvalidEndTag",
            "InvalidEndTag",
            "InvalidEndTag",
            "InvalidEndTag",
            "InvalidEndTag",
            "InvalidEndTag",
            "InvalidEndTag",
            "InvalidEndTag",
            "InvalidEndTag",
            "InvalidEndTag",
            "InvalidEndTag",
            "InvalidEndTag",
            "MissingEndTag",
            "MissingEndTag",
            "MissingEndTag",
            "MissingEndTag",
            "MissingEndTag",
            "MissingEndTag",
            "MissingEndTag",
            "MissingEndTag",
        ],
    ),
    (
        "habitica/website/client/src/components/static/privacy.vue",
        &["InvalidEndTag"],
    ),
    (
        "heyui/src/components/carousel/carousel.vue",
        &["VIfSameKey"],
    ),
    (
        "heyui/src/components/table/table.vue",
        &["VElseNoAdjacentIf"],
    ),
    ("tdesign/site/src/pages/design/fonts.vue", &["VIfSameKey"]),
    (
        "tdesign/site/src/pages/design/fonts_zh-CN.vue",
        &["VIfSameKey"],
    ),
    (
        "vue-manage-system/src/views/table/basetable.vue",
        &["InvalidEndTag"],
    ),
    (
        "vue2-elm/src/page/shop/shop.vue",
        &["MissingWhitespaceBetweenAttributes"],
    ),
    (
        "vuesax/docs/.vuepress/theme/Home.vue",
        &["InvalidEndTag", "InvalidEndTag"],
    ),
    (
        "vuesax/docs/.vuepress/theme/homePatreons.vue",
        &["InvalidEndTag"],
    ),
    (
        "vux/src/components/inline-x-number/index.vue",
        &["MissingWhitespaceBetweenAttributes"],
    ),
    (
        "vux/src/components/x-number/index.vue",
        &["MissingWhitespaceBetweenAttributes"],
    ),
    (
        "vux/src/demos/PopupPicker.vue",
        &["MissingWhitespaceBetweenAttributes"],
    ),
];

pub fn old_lane_skip_is_allowed(name: &str, actual_codes: &[String]) -> bool {
    let Some(path) = fixture_path(name) else {
        return false;
    };
    let Some((_, expected_codes)) = OLD_LANE_INVALID_FIXTURES
        .iter()
        .find(|(allowed_path, _)| *allowed_path == path)
    else {
        return false;
    };
    let mut actual = actual_codes.iter().map(String::as_str).collect::<Vec<_>>();
    actual.sort_unstable();
    let mut expected = expected_codes.to_vec();
    expected.sort_unstable();
    actual == expected
}

fn fixture_path(name: &str) -> Option<&str> {
    name.split_once("tests/_fixtures/_git/")
        .map(|(_, path)| path)
}
