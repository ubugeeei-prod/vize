use vize_atelier_core::TemplateSyntaxMode;
use vize_atelier_dom::{DomCompilerOptions, compile_template_with_template_syntax};
use vize_s0::Allocator;

#[test]
fn quirks_compiles_html_tree_recovery_cases_without_cascading_errors() {
    let cases = [
        "<button><span><button>share</button></span></button>",
        "<a href=\"#outer\"><span><a href=\"#inner\">open</a></span></a>",
        "<form><section><form><input></form></section></form>",
        "<table><div>description</div><tr><td>value</td></tr></table>",
    ];

    for source in cases {
        let allocator = Allocator::new();
        let (_, errors, result) = compile_template_with_template_syntax(
            &allocator,
            source,
            DomCompilerOptions::default(),
            TemplateSyntaxMode::Quirks,
        );

        assert!(errors.is_empty(), "{source}: {errors:?}");
        assert!(!result.code.is_empty(), "{source}");
    }
}
