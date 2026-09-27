use super::options_api_declarations;

/// Registered SFC inputs exercise wrappers, inherited/camelCase props,
/// unrelated local bindings and a callable module binding named `undefined`.
/// Compare every emitted Options API declaration, including its mutability.
#[test]
fn computed_setter_corpus_preserves_complete_binding_vectors() {
    let cases = [
        (
            "HoistedSetter.vue",
            include_str!(
                "../../../../../../../tests/fixtures/typechecker/options-api-computed-setters/HoistedSetter.vue"
            ),
            include_str!(
                "../../../../../../../tests/fixtures/typechecker/options-api-computed-setters/HoistedSetter.bindings.json"
            ),
        ),
        (
            "WrappedOptions.vue",
            include_str!(
                "../../../../../../../tests/fixtures/typechecker/options-api-computed-setters/WrappedOptions.vue"
            ),
            include_str!(
                "../../../../../../../tests/fixtures/typechecker/options-api-computed-setters/WrappedOptions.bindings.json"
            ),
        ),
        (
            "WrappedSetters.vue",
            include_str!(
                "../../../../../../../tests/fixtures/typechecker/options-api-computed-setters/WrappedSetters.vue"
            ),
            include_str!(
                "../../../../../../../tests/fixtures/typechecker/options-api-computed-setters/WrappedSetters.bindings.json"
            ),
        ),
        (
            "ShadowedSetter.vue",
            include_str!(
                "../../../../../../../tests/fixtures/typechecker/options-api-computed-setters/ShadowedSetter.vue"
            ),
            include_str!(
                "../../../../../../../tests/fixtures/typechecker/options-api-computed-setters/ShadowedSetter.bindings.json"
            ),
        ),
    ];
    for (name, source, expected) in cases {
        let script = source
            .split_once("<script lang=\"ts\">\n")
            .expect("fixture must have a TypeScript script")
            .1
            .split_once("</script>")
            .expect("fixture script must close")
            .0;
        let template = source
            .split_once("<template>\n")
            .expect("fixture must have a template")
            .1
            .split_once("</template>")
            .expect("fixture template must close")
            .0;
        let expected: Vec<std::string::String> = serde_json::from_str(expected).unwrap();
        assert_eq!(
            options_api_declarations(script, template),
            expected,
            "{name}"
        );
    }
}
