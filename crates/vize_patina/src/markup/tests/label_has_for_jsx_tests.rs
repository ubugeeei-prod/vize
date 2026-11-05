#[cfg(test)]
mod tests {
    use crate::context::LintContext;
    use crate::diagnostic::LintDiagnostic;
    use crate::ir::TemplateSyntax;
    use crate::markup::{MarkupContext, MarkupDocument};
    use crate::rules::a11y::LabelHasFor;
    use vize_atelier_jsx::JsxLang;
    use vize_l0::Allocator;

    #[test]
    fn directly_embedded_jsx_controls_associate_their_label() {
        for (source, has_control) in [
            ("const A = () => <label>{<input />}</label>;", true),
            ("const A = () => <label>{<select />}</label>;", true),
            ("const A = () => <label>{<textarea />}</label>;", true),
            ("const A = () => <label>{<><input /></>}</label>;", true),
            (
                "const A = () => <label>{<><span /><textarea /></>}</label>;",
                true,
            ),
            ("const A = () => <label>{<span />}</label>;", false),
            ("const A = () => <label>{<><span /></>}</label>;", false),
            ("const A = () => <label>{<Input />}</label>;", false),
            ("const A = () => <label>{render(<input />)}</label>;", false),
            ("const A = () => <label>{cond && <input />}</label>;", true),
            ("const A = () => <label>{cond && <span />}</label>;", false),
            (
                "const A = () => <label>{items.map(() => <input />)}</label>;",
                true,
            ),
            (
                "const A = () => <label>{items.map(() => <span />)}</label>;",
                false,
            ),
        ] {
            for lang in [JsxLang::Jsx, JsxLang::Tsx] {
                let oxc_allocator = oxc_allocator::Allocator::default();
                let parsed = vize_atelier_jsx::parse_module(&oxc_allocator, source, lang);
                let document = MarkupDocument::from_jsx(&parsed.program, TemplateSyntax::Vue, 0);
                let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
                let mut lint = LintContext::new(&allocator, source, "test.jsx");
                let mut ctx = MarkupContext::new(&mut lint, &document);
                document.visit_with(&LabelHasFor, &mut ctx);

                let expected = if has_control {
                    Vec::new()
                } else {
                    vec![
                        LintDiagnostic::warn(
                            "a11y/label-has-for",
                            lint.t("a11y/label-has-for.message"),
                            source.find("<label>").unwrap() as u32,
                            (source.rfind("</label>").unwrap() + "</label>".len()) as u32,
                        )
                        .with_help(lint.t("a11y/label-has-for.help")),
                    ]
                };
                assert_eq!(
                    format!("{:?}", lint.diagnostics()),
                    format!("{expected:?}"),
                    "{lang:?}: {source}"
                );
            }
        }
    }
}
