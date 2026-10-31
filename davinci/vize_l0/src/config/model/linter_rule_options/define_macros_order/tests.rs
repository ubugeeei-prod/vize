use super::DefineMacrosOrderOptions;
use serde_json::json;

#[test]
fn expose_last_defaults_and_old_serialization_are_preserved() {
    for value in [json!({}), json!({"defineExposeLast":false})] {
        let parsed: DefineMacrosOrderOptions = serde_json::from_value(value).unwrap();
        assert_eq!(parsed, DefineMacrosOrderOptions::default());
        assert_eq!(
            serde_json::to_value(parsed).unwrap(),
            json!({"order":["defineProps","defineEmits"]})
        );
    }
    let configured: DefineMacrosOrderOptions =
        serde_json::from_value(json!({"defineExposeLast":true})).unwrap();
    assert!(configured.define_expose_last);
    assert_eq!(
        serde_json::to_value(configured).unwrap(),
        json!({"order":["defineProps","defineEmits"],"defineExposeLast":true})
    );
}

#[test]
fn supported_boolean_does_not_admit_unknown_keys_bad_types_or_conflicting_order() {
    for value in [
        json!({"defineExposeLast":"true"}),
        json!({"defineExposeLast":1}),
        json!({"defineExposeLast":null}),
        json!({"defineExposeLastTypo":true}),
        json!({"order":["defineExpose"],"defineExposeLast":true}),
        json!({"order":["defineProps","defineProps"],"defineExposeLast":true}),
        json!({"order":[""],"defineExposeLast":true}),
    ] {
        assert!(
            serde_json::from_value::<DefineMacrosOrderOptions>(value.clone()).is_err(),
            "{value}"
        );
    }
    for value in [
        json!({"order":[],"defineExposeLast":true}),
        json!({"order":["definePage"],"defineExposeLast":true}),
        json!({"order":["defineExpose"],"defineExposeLast":false}),
    ] {
        serde_json::from_value::<DefineMacrosOrderOptions>(value).unwrap();
    }
    let invalid = DefineMacrosOrderOptions {
        order: vec!["defineExpose".into()],
        define_expose_last: true,
    };
    assert_eq!(
        invalid.validate(),
        Err(DefineMacrosOrderOptions::EXPOSE_LAST_CONFLICT)
    );
}
