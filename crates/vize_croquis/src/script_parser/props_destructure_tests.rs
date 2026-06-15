use super::parse_script_setup;
use vize_relief::BindingType;

#[test]
fn test_parse_define_props_destructure_tracks_aliased_defaults() {
    let result = parse_script_setup(
        r#"
            const {
                count: localCount = 1,
                label = "Untitled",
                ...rest
            } = defineProps<{
                count?: number
                label?: string
                id?: string
            }>()
        "#,
    );

    let destructure = result
        .macros
        .props_destructure()
        .expect("defineProps destructure metadata should be recorded");
    let count = destructure
        .get("count")
        .expect("renamed prop should use the original prop key");
    assert_eq!(count.local.as_str(), "localCount");
    assert_eq!(count.default.as_deref(), Some("1"));

    let label = destructure
        .get("label")
        .expect("shorthand prop should be recorded");
    assert_eq!(label.local.as_str(), "label");
    assert_eq!(label.default.as_deref(), Some("\"Untitled\""));
    assert_eq!(destructure.rest_id.as_deref(), Some("rest"));

    assert_eq!(result.bindings.get("localCount"), Some(BindingType::Props));
    assert_eq!(result.bindings.get("label"), Some(BindingType::Props));
    assert_eq!(result.bindings.get("rest"), Some(BindingType::Props));
}
