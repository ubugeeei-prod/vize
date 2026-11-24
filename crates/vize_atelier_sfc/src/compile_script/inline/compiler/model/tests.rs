use super::{model_type_accepts_null, model_value_prop};
use crate::script::ScriptCompileContext;
use crate::{
    SfcCompileOptions, SfcParseOptions, SfcScriptOutputMode, compile_sfc_for_adapter, parse_sfc,
};

#[test]
fn model_runtime_types_preserve_top_level_null() {
    let mut ctx = ScriptCompileContext::new("");
    ctx.type_aliases
        .insert("Maybe".into(), "number | null".into());
    for (ty, expected) in [
        ("number | null", "{ type: [Number, null] }"),
        ("string | null", "{ type: [String, null] }"),
        ("(number | null)", "{ type: [Number, null] }"),
        ("Maybe", "{ type: [Number, null] }"),
        ("Array<number | null>", "{ type: Array }"),
        ("() => number | null", "{ type: Function }"),
        ("{ value: number | null }", "{ type: Object }"),
        ("null", "{ type: null }"),
        ("unknown | null", "{ type: null }"),
        (
            "boolean | unknown | null",
            "{ type: [Boolean, null], skipCheck: true }",
        ),
    ] {
        assert_eq!(
            model_value_prop(&ctx, &None, &Some(ty.into()), false),
            expected,
            "{ty}"
        );
    }
    assert_eq!(
        model_value_prop(
            &ctx,
            &Some("{ required: true }".into()),
            &Some("number | null".into()),
            false
        ),
        "{ type: [Number, null], ...{ required: true } }"
    );
}

#[test]
fn model_nullability_follows_shared_and_recursive_aliases() {
    let mut ctx = ScriptCompileContext::new("");
    for (name, body) in [
        ("Model", "Left | Right"),
        ("Left", "Shared"),
        ("Right", "Shared | Left"),
        ("Shared", "Left"),
    ] {
        ctx.type_aliases.insert(name.into(), body.into());
    }
    let mut visited = vize_carton::FxHashSet::default();
    assert!(!model_type_accepts_null(&ctx, "Model", &mut visited));
    assert_eq!(visited.len(), 4);
    ctx.type_aliases
        .insert("Shared".into(), "Left | null".into());
    assert!(model_type_accepts_null(
        &ctx,
        "Model",
        &mut Default::default()
    ));
}

#[test]
fn model_runtime_types_preserve_top_level_null_in_ssr_module() {
    let source = include_str!(
        "../../../../../../../tests/_fixtures/differential/compiler/nullable-model-runtime-13546/Layout.vue.txt"
    );
    let descriptor = parse_sfc(
        source,
        SfcParseOptions {
            filename: "Layout.vue".into(),
            ..Default::default()
        },
    )
    .unwrap();
    let mut options = SfcCompileOptions::default();
    options.parse.filename = "Layout.vue".into();
    options.template.ssr = true;
    let result = compile_sfc_for_adapter(
        &descriptor,
        options,
        Default::default(),
        Default::default(),
        Default::default(),
        SfcScriptOutputMode::SeparateTemplate,
    )
    .unwrap();
    assert_eq!(result.errors.len(), 0);
    assert_eq!(result.warnings.len(), 0);
    assert_eq!(result.css, None);
    assert_eq!(
        result.code,
        include_str!(
            "../../../../../../../tests/_fixtures/differential/compiler/nullable-model-runtime-13546/module.expected.txt"
        )
    );
}
