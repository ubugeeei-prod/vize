//! Cross-backend verification for the rule IR.
//!
//! Each test drives a [`MarkupRule`] over a Vue template fixture **and** a
//! JSX fixture and asserts the diagnostic count, proving one rule body runs
//! over both backends through the zero-copy facade.

// Wrapped in an inline `#[cfg(test)] mod` (the repo convention for split
// test files) so the Davinci assertion lint, which only scans inline
// `#[cfg(test)] mod` bodies under `src/`, keeps covering these tests.
mod deprecated_attr_tests;
mod heading_levels_tests;
mod iframe_has_title_jsx_tests;
mod iframe_has_title_tests;
mod interactive_supports_focus_jsx_tests;
mod interactive_supports_focus_tests;
mod media_has_caption_tests;
mod mouse_events_have_key_events_tests;
mod no_aria_hidden_on_focusable_jsx_tests;
mod no_aria_hidden_on_focusable_tests;
mod no_bare_strings_in_template_tests;
mod no_boolean_attr_value_tests;
mod no_consecutive_br_tests;
mod no_dupe_style_properties_tests;
mod no_duplicate_class_tests;
mod no_duplicate_dt_tests;
mod no_i_for_icon_tests;
mod no_redundant_roles_tests;
mod no_role_presentation_on_focusable_jsx_tests;
mod no_role_presentation_on_focusable_tests;
mod placeholder_label_option_tests;
mod require_datetime_tests;
mod use_list_tests;

#[cfg(test)]
mod markup_ir_tests {
    use crate::context::LintContext;
    use crate::markup::*;
    use crate::rules::a11y::ImgAlt;
    use crate::rules::vapor::{NoVueLifecycleEvents, PreferStaticClass};
    use crate::rules::vue::RequireVForKey;
    use vize_atelier_jsx::JsxLang;
    use vize_s0::Allocator;

    /// Run a markup rule over a Vue template and return the diagnostic count.
    fn run_over_template<R: MarkupRule>(rule: &R, source: &str) -> usize {
        let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
        let parser = vize_armature::Parser::new(&allocator, source);
        let (root, _errors) = parser.parse();
        let document = MarkupDocument::new(&root, TemplateSyntax::Vue);

        let mut lint = LintContext::new(&allocator, source, "test.vue");
        let mut ctx = MarkupContext::new(&mut lint, &document);
        document.visit_with(rule, &mut ctx);
        lint.diagnostics().len()
    }

    /// Run a markup rule over JSX/TSX **lowered to the shared relief AST**, the
    /// path directive-shaped rules use (so `.map()`/`key={…}` surface as
    /// `v-for`/`:key`). Returns the diagnostic count.
    fn run_over_jsx_lowered<R: MarkupRule>(rule: &R, source: &str) -> usize {
        let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
        let lowered =
            vize_atelier_jsx::lower_source(&allocator, allocator.as_oxc(), source, JsxLang::Jsx);

        let mut total = 0;
        for lowered_root in &lowered.roots {
            let document = MarkupDocument::new(&lowered_root.root, TemplateSyntax::Vue);
            let mut lint = LintContext::new(&allocator, source, "test.jsx");
            let mut ctx = MarkupContext::new(&mut lint, &document);
            document.visit_with(rule, &mut ctx);
            total += lint.diagnostics().len();
        }
        total
    }

    /// Run a markup rule over JSX projected **directly from the OXC AST** (no
    /// relief lowering), the path HTML-shaped rules use. Returns the diagnostic
    /// count.
    fn run_over_jsx_oxc<R: MarkupRule>(rule: &R, source: &str) -> usize {
        let oxc_allocator = oxc_allocator::Allocator::default();
        let parsed = vize_atelier_jsx::parse_module(&oxc_allocator, source, JsxLang::Jsx);
        let document = MarkupDocument::from_jsx(&parsed.program, TemplateSyntax::Vue, 0);

        // The lint context still needs an arena; reuse a fresh carton allocator.
        let lint_allocator = Allocator::with_capacity(source.len() * 4 + 1024);
        let mut lint = LintContext::new(&lint_allocator, source, "test.jsx");
        let mut ctx = MarkupContext::new(&mut lint, &document);
        document.visit_with(rule, &mut ctx);
        lint.diagnostics().len()
    }

    // ---- vue/require-v-for-key (Vue correctness) ----------------------------

    #[test]
    fn require_v_for_key_template() {
        let rule = RequireVForKey;
        assert_eq!(
            run_over_template(
                &rule,
                r#"<ul><li v-for="item in items">{{ item }}</li></ul>"#
            ),
            1,
            "template v-for without :key must report through the IR"
        );
        assert_eq!(
            run_over_template(
                &rule,
                r#"<ul><li v-for="item in items" :key="item.id">{{ item }}</li></ul>"#
            ),
            0,
            "template v-for with :key must be clean"
        );
    }

    #[test]
    fn require_v_for_key_jsx() {
        let rule = RequireVForKey;
        // `.map()` lowers to v-for; missing key must report.
        assert_eq!(
            run_over_jsx_lowered(
                &rule,
                "const L = () => <ul>{items.map((item) => <li>{item}</li>)}</ul>;",
            ),
            1,
            "JSX .map() without key must report through the IR"
        );
        // With a key it is clean.
        assert_eq!(
            run_over_jsx_lowered(
                &rule,
                "const L = () => <ul>{items.map((item) => <li key={item.id}>{item}</li>)}</ul>;",
            ),
            0,
            "JSX .map() with key={{…}} must be clean"
        );
    }

    // ---- a11y/img-alt (accessibility / HTML) --------------------------------

    #[test]
    fn img_alt_template() {
        let rule = ImgAlt;
        assert_eq!(
            run_over_template(&rule, r#"<img src="/photo.jpg" />"#),
            1,
            "template <img> without alt must warn through the IR"
        );
        assert_eq!(
            run_over_template(&rule, r#"<img src="/photo.jpg" alt="Team photo" />"#),
            0,
            "template <img> with alt must be clean"
        );
        assert_eq!(
            run_over_template(&rule, r#"<img :src="photo" :alt="caption" />"#),
            0,
            "template <img> with dynamic :alt must be clean"
        );
    }

    #[test]
    fn img_alt_jsx_oxc() {
        let rule = ImgAlt;
        // Projected straight from the OXC AST — no synthetic template AST.
        assert_eq!(
            run_over_jsx_oxc(&rule, "const I = () => <img src=\"/photo.jpg\" />;"),
            1,
            "JSX <img> without alt must warn through the OXC IR path"
        );
        assert_eq!(
            run_over_jsx_oxc(
                &rule,
                "const I = () => <img src=\"/photo.jpg\" alt=\"Team\" />;"
            ),
            0,
            "JSX <img> with static alt must be clean"
        );
        assert_eq!(
            run_over_jsx_oxc(&rule, "const I = () => <img src={photo} alt={caption} />;"),
            0,
            "JSX <img> with dynamic alt={{…}} must be clean"
        );
    }

    // ---- vapor/prefer-static-class (Vapor) ----------------------------------

    #[test]
    fn prefer_static_class_template() {
        let rule = PreferStaticClass;
        assert_eq!(
            run_over_template(&rule, r#"<div :class="'static'"></div>"#),
            1,
            "template :class with a string literal must warn through the IR"
        );
        assert_eq!(
            run_over_template(&rule, r#"<div :class="dynamic"></div>"#),
            0,
            "template :class with a real expression must be clean"
        );
        assert_eq!(
            run_over_template(&rule, r#"<div class="static"></div>"#),
            0,
            "template static class must be clean"
        );
    }

    #[test]
    fn prefer_static_class_jsx() {
        let rule = PreferStaticClass;
        // `class={'static'}` lowers to the same `:class="'static'"` string
        // literal a Vue template produces.
        assert_eq!(
            run_over_jsx_lowered(&rule, "const C = () => <div class={'static'} />;"),
            1,
            "JSX class={{'static'}} must warn through the IR"
        );
        assert_eq!(
            run_over_jsx_lowered(&rule, "const C = () => <div class={dynamic} />;"),
            0,
            "JSX class={{dynamic}} must be clean"
        );
    }

    // ---- vapor/no-vue-lifecycle-events (Vapor, template-native bonus) -------

    #[test]
    fn no_vue_lifecycle_events_template() {
        let rule = NoVueLifecycleEvents;
        assert_eq!(
            run_over_template(&rule, r#"<div @vue:mounted="onMounted"></div>"#),
            1,
            "template @vue:mounted must report through the IR"
        );
        assert_eq!(
            run_over_template(&rule, r#"<div @click="onClick"></div>"#),
            0,
            "template @click must be clean"
        );
    }

    // ---- Facade unit coverage ----------------------------------------------

    #[test]
    fn jsx_binding_classification() {
        // `onClick` is an event, `class={…}` is a bind, `id="x"` is a plain
        // attribute, `key={…}` is a key binding.
        let oxc_allocator = oxc_allocator::Allocator::default();
        let source = "const C = () => <li id=\"a\" class={cls} key={k} onClick={f} />;";
        let parsed = vize_atelier_jsx::parse_module(&oxc_allocator, source, JsxLang::Jsx);
        let document = MarkupDocument::from_jsx(&parsed.program, TemplateSyntax::Vue, 0);

        let mut kinds = Vec::new();
        let mut has_key = false;
        let mut click_is_event = false;
        document.walk_elements(&mut |element| {
            if element.is_tag("li") {
                has_key = element.has_key_binding();
                element.walk_bindings(&mut |binding| {
                    kinds.push((binding.arg_name().map(str::to_owned), binding.kind()));
                    // `onClick` is an event; its argument matches `click`
                    // case-insensitively (JSX event names are PascalCase).
                    if binding.kind() == MarkupBindingKind::On && binding.arg_name_eq("click") {
                        click_is_event = true;
                    }
                });
            }
        });

        assert!(has_key, "key={{k}} must be detected as a key binding");
        assert!(
            click_is_event,
            "onClick must be an event binding with arg `click`"
        );
        assert!(kinds.contains(&(Some("id".to_owned()), MarkupBindingKind::Attribute)));
        assert!(kinds.contains(&(Some("class".to_owned()), MarkupBindingKind::Bind)));
        assert!(kinds.contains(&(Some("key".to_owned()), MarkupBindingKind::Bind)));
    }

    #[test]
    fn template_event_modifiers_are_exposed() {
        // Modifiers come through the normalized binding view for templates.
        let allocator = Allocator::with_capacity(1024);
        let source = r#"<button @click.stop.prevent="f"></button>"#;
        let parser = vize_armature::Parser::new(&allocator, source);
        let (root, _errors) = parser.parse();
        let document = MarkupDocument::new(&root, TemplateSyntax::Vue);

        let mut modifiers = Vec::new();
        document.walk_elements(&mut |element| {
            element.walk_bindings(&mut |binding| {
                if binding.kind() == MarkupBindingKind::On {
                    binding.walk_modifiers(&mut |m| modifiers.push(m.to_owned()));
                }
            });
        });
        assert_eq!(modifiers, vec!["stop".to_owned(), "prevent".to_owned()]);
    }

    #[test]
    fn diagnostics_map_to_original_source_offsets() {
        // The reported range must fall inside the original source for both
        // backends — this is what makes fixes map back to written syntax.
        let rule = ImgAlt;
        let allocator = Allocator::with_capacity(1024);
        let source = r#"<div><img src="/p.jpg" /></div>"#;
        let parser = vize_armature::Parser::new(&allocator, source);
        let (root, _errors) = parser.parse();
        let document = MarkupDocument::new(&root, TemplateSyntax::Vue);
        let mut lint = LintContext::new(&allocator, source, "test.vue");
        let mut ctx = MarkupContext::new(&mut lint, &document);
        document.visit_with(&rule, &mut ctx);

        let diagnostics = lint.diagnostics();
        assert_eq!(diagnostics.len(), 1);
        let diag = &diagnostics[0];
        let img_start = source.find("<img").unwrap() as u32;
        assert_eq!(diag.start, img_start, "range must point at the <img> tag");
        assert!(diag.end <= source.len() as u32);
        assert_eq!(&source[diag.start as usize..diag.end as usize][..4], "<img");
    }
}
