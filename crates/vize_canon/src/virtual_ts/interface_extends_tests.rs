use super::generate_virtual_ts;
use vize_croquis::{Analyzer, AnalyzerOptions};

#[test]
fn define_props_local_interface_extends_binds_inherited_template_props() {
    let script = r#"
interface BaseProps {
  required?: boolean
}

interface FooProps extends BaseProps {
  label: string
}

defineProps<FooProps>();
"#;
    let template = r#"<input :required="required" :aria-label="label" />"#;

    let allocator = vize_carton::Bump::new();
    let (root, _) = vize_armature::parse(&allocator, template);

    let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
    analyzer.analyze_script_setup(script);
    analyzer.analyze_template(&root);
    let summary = analyzer.finish();

    let props = summary.types.extract_properties("FooProps");
    let names = props
        .iter()
        .map(|prop| prop.name.as_str())
        .collect::<Vec<_>>();
    assert_eq!(names, ["required", "label"]);

    let output = generate_virtual_ts(&summary, Some(script), Some(&root), 0);

    for field in ["required", "label"] {
        assert!(
            output
                .code
                .contains(&format!(r#"const {field} = props["{field}"];"#)),
            "expected inherited prop `{field}` to be bound in template scope:\n{}",
            output.code
        );
    }
}
