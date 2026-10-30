use vize_croquis::{Analyzer, AnalyzerOptions};

use super::super::TemplatePropsModel;

#[test]
fn runtime_constructor_booleans_resolve_without_changing_authored_defaults() {
    let resolved = "__WithDefaultsResult<Props, Pick<Props, Extract<\"enabled\", keyof Props>>>";
    for (declaration, expected, default) in [
        ("Boolean", resolved, None),
        ("{ type: Boolean, required: true }", resolved, None),
        ("[String, Boolean]", resolved, None),
        ("[Boolean, String]", resolved, None),
        ("Boolean as PropType<boolean | null>", resolved, None),
        ("{ type: Boolean, default: false }", resolved, Some("false")),
        ("{ type: Boolean, default: null }", resolved, Some("null")),
        (
            "{ type: Boolean, default: undefined }",
            "Props",
            Some("undefined"),
        ),
        ("String as PropType<boolean>", "Props", None),
        ("String", "Props", None),
    ] {
        let script = vize_carton::cstr!("defineProps({{ enabled: {declaration} }})");
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(&script);
        let summary = analyzer.finish();
        let model = TemplatePropsModel::new(&summary, None);
        assert_eq!(model.authored_type_ref.as_str(), "Props", "{declaration}");
        assert_eq!(model.resolved_type_ref.as_str(), expected, "{declaration}");
        assert_eq!(summary.macros.props().len(), 1, "{declaration}");
        assert_eq!(
            summary.macros.props()[0].default_value.as_deref(),
            default,
            "raw metadata must retain the authored default: {declaration}"
        );
    }
}

#[test]
fn unresolved_runtime_overrides_and_shadowed_constructors_do_not_prove_casting() {
    for script in [
        "defineProps({ enabled: Boolean, ...unknownProps })",
        "defineProps({ enabled: { type: Boolean, ...unknownOptions } })",
        "defineProps({ enabled: { ...unknownOptions, type: Boolean } })",
        "defineProps({ enabled: Boolean, enabled: String })",
        "const Boolean = String; defineProps({ enabled: Boolean })",
    ] {
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        let summary = analyzer.finish();
        let model = TemplatePropsModel::new(&summary, None);
        assert_eq!(model.resolved_type_ref.as_str(), "Props", "{script}");
    }
}
