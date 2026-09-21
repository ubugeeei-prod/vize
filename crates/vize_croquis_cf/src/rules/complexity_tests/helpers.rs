use crate::rules::TemplateComplexity;

/// The S2 facts of `template` wrapped in an SFC.
pub(crate) fn template_facts(template: &str) -> TemplateComplexity {
    let source = vize_carton::cstr!("<template>{template}</template>\n");
    TemplateComplexity::from_sfc(source.as_str()).expect("an HTML template has facts")
}
