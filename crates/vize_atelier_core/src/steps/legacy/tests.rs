// Wrapped in an inline `#[cfg(test)] mod` (the repo convention for split
// test files) so the Davinci assertion lint, which only scans inline
// `#[cfg(test)] mod` bodies under `src/`, keeps covering these tests.
#[cfg(test)]
#[allow(clippy::disallowed_macros)]
mod legacy_desugar_tests {
    use super::super::*;
    use crate::codegen::generate;
    use crate::lane::transform;
    use crate::options::{CodegenOptions, TransformOptions};
    use crate::parser::parse;
    use crate::{SimpleExpressionNode, SourceLocation};
    use vize_armature::legacy::{LegacyDialectCapabilities, LegacyVueVersion};
    use vize_carton::Vec as ArenaVec;
    use vize_carton::config::VueVersion;

    /// Full lane (parse -> transform -> codegen) under a given dialect.
    fn compile(src: &str, dialect: VueVersion) -> std::string::String {
        let allocator = vize_carton::Allocator::new();
        let (mut root, errs) = parse(&allocator, src);
        assert!(errs.is_empty(), "parse errors: {errs:?}");
        let opts = TransformOptions {
            dialect,
            ..Default::default()
        };
        transform(&allocator, &mut root, opts, None);
        generate(&root, CodegenOptions::default())
            .code
            .as_str()
            .to_owned()
    }

    fn v2_caps() -> LegacyDialectCapabilities {
        LegacyVueVersion::V2.capabilities()
    }

    fn directives<'a>(el: &'a ElementNode<'a>) -> std::vec::Vec<&'a DirectiveNode<'a>> {
        el.props
            .iter()
            .filter_map(|p| match p {
                PropNode::Directive(d) => Some(d.as_ref()),
                _ => None,
            })
            .collect()
    }

    fn first_element<'a>(root: &'a RootNode<'a>) -> &'a ElementNode<'a> {
        match &root.children[0] {
            TemplateChildNode::Element(el) => el.as_ref(),
            _ => panic!("expected element"),
        }
    }

    #[test]
    fn sync_modifier_desugars_to_bind_plus_update_listener() {
        let allocator = vize_carton::Allocator::new();
        let source = r#"<Comp :foo.sync="bar" />"#;
        let (mut root, errs) = parse(&allocator, source);
        assert!(errs.is_empty());
        desugar_legacy_template(&allocator, &mut root, v2_caps());

        let el = first_element(&root);
        let dirs = directives(el);
        // Original bind (sync stripped) + new on:update:foo listener.
        assert_eq!(dirs.len(), 2);

        let bind = dirs.iter().find(|d| d.name == "bind").unwrap();
        assert!(bind.modifiers.is_empty(), "sync modifier must be stripped");
        assert_eq!(bind.arg.as_ref().unwrap().loc().span.slice(source), "foo");

        let on = dirs.iter().find(|d| d.name == "on").unwrap();
        assert_eq!(
            match on.arg.as_ref().unwrap() {
                ExpressionNode::Simple(s) => s.content,
                _ => panic!(),
            },
            "update:foo"
        );
        assert_eq!(
            match on.exp.as_ref().unwrap() {
                ExpressionNode::Simple(s) => s.content,
                _ => panic!(),
            },
            "$event => ((bar) = $event)"
        );
    }

    #[test]
    fn sync_modifier_preserves_other_modifiers() {
        let allocator = vize_carton::Allocator::new();
        // `.sync` alongside another modifier: only `sync` is stripped.
        let (mut root, _) = parse(&allocator, r#"<Comp :foo.sync.camel="bar" />"#);
        desugar_legacy_template(&allocator, &mut root, v2_caps());
        let el = first_element(&root);
        let bind = directives(el)
            .into_iter()
            .find(|d| d.name == "bind")
            .unwrap();
        assert_eq!(bind.modifiers.len(), 1);
        assert_eq!(bind.modifiers[0].content, "camel");
    }

    #[test]
    fn template_slot_scope_desugars_to_v_slot() {
        let allocator = vize_carton::Allocator::new();
        let source = r#"<Comp><template slot="header" slot-scope="props">x</template></Comp>"#;
        let (mut root, _) = parse(&allocator, source);
        desugar_legacy_template(&allocator, &mut root, v2_caps());

        let comp = first_element(&root);
        let tmpl = match &comp.children[0] {
            TemplateChildNode::Element(el) => el.as_ref(),
            _ => panic!("expected template element"),
        };
        // slot + slot-scope attributes consumed, replaced by one v-slot directive.
        assert!(
            !tmpl.props.iter().any(|p| matches!(p, PropNode::Attribute(a)
                    if a.name == "slot" || a.name == "slot-scope")),
            "legacy slot attrs must be consumed"
        );
        let dirs = directives(tmpl);
        assert_eq!(dirs.len(), 1);
        let v_slot = dirs[0];
        assert_eq!(v_slot.name, "slot");
        let arg = match v_slot.arg.as_ref().unwrap() {
            ExpressionNode::Simple(s) => s,
            _ => panic!(),
        };
        assert_eq!(arg.content, "header");
        assert_eq!(arg.loc.span.slice(source), "header");

        let exp = match v_slot.exp.as_ref().unwrap() {
            ExpressionNode::Simple(s) => s,
            _ => panic!(),
        };
        assert_eq!(exp.content, "props");
        assert_eq!(exp.loc.span.slice(source), "props");
    }

    #[test]
    fn scope_alias_desugars_to_default_v_slot() {
        let allocator = vize_carton::Allocator::new();
        // `scope` (2.1 alias) with no `slot=` => default slot.
        let source = r#"<Comp><template scope="props">x</template></Comp>"#;
        let (mut root, _) = parse(&allocator, source);
        desugar_legacy_template(&allocator, &mut root, v2_caps());
        let comp = first_element(&root);
        let tmpl = match &comp.children[0] {
            TemplateChildNode::Element(el) => el.as_ref(),
            _ => panic!(),
        };
        let dirs = directives(tmpl);
        assert_eq!(dirs.len(), 1);
        assert_eq!(dirs[0].name, "slot");
        assert!(dirs[0].arg.is_none(), "no slot= means default slot");
        let exp = match dirs[0].exp.as_ref().unwrap() {
            ExpressionNode::Simple(s) => s,
            _ => panic!(),
        };
        assert_eq!(exp.content, "props");
        assert_eq!(exp.loc.span.slice(source), "props");
    }

    #[test]
    fn vue3_dialect_is_a_noop() {
        let allocator = vize_carton::Allocator::new();
        let (mut root, _) = parse(
            &allocator,
            r#"<Comp :foo.sync="bar"><template slot-scope="props">x</template></Comp>"#,
        );
        // Vue 3 capability set: nothing should change.
        desugar_legacy_template(
            &allocator,
            &mut root,
            LegacyDialectCapabilities::for_dialect(VueVersion::V3),
        );
        let comp = first_element(&root);
        let bind = directives(comp)
            .into_iter()
            .find(|d| d.name == "bind")
            .unwrap();
        // sync modifier still present, no update listener added.
        assert_eq!(bind.modifiers.len(), 1);
        assert_eq!(bind.modifiers[0].content, "sync");
        assert!(
            !directives(comp).iter().any(|d| d.name == "on"),
            "no listener added under Vue 3"
        );
        let tmpl = match &comp.children[0] {
            TemplateChildNode::Element(el) => el.as_ref(),
            _ => panic!(),
        };
        assert!(
            tmpl.props
                .iter()
                .any(|p| matches!(p, PropNode::Attribute(a) if a.name == "slot-scope")),
            "slot-scope stays a plain attribute under Vue 3"
        );
    }

    #[test]
    fn e2e_sync_generates_update_listener_under_v2() {
        // `.sync` on a component prop under Vue 2 must emit an update:foo handler.
        let code = compile(r#"<Comp :foo.sync="bar" />"#, VueVersion::V2);
        assert!(
            code.contains("\"onUpdate:foo\""),
            "expected onUpdate:foo handler, got:\n{code}"
        );
        assert!(
            code.contains("foo: bar") || code.contains("foo:bar"),
            "expected :foo binding preserved, got:\n{code}"
        );
    }

    #[test]
    fn e2e_sync_is_unknown_modifier_under_v3() {
        // Under the default Vue 3 dialect `.sync` is just an (ignored) modifier:
        // no update listener is synthesized.
        let code = compile(r#"<Comp :foo.sync="bar" />"#, VueVersion::V3);
        assert!(
            !code.contains("onUpdate:foo"),
            "Vue 3 must not synthesize a .sync update listener, got:\n{code}"
        );
    }

    #[test]
    fn e2e_slot_scope_generates_scoped_slot_under_v2() {
        let code = compile(
            r#"<Comp><template slot="header" slot-scope="props">{{ props.x }}</template></Comp>"#,
            VueVersion::V2,
        );
        assert!(
            code.contains("header:") && code.contains("withCtx"),
            "expected a `header` scoped slot, got:\n{code}"
        );
    }

    #[test]
    fn e2e_v3_default_byte_identical_for_plain_template() {
        // A template with no legacy sugar must compile identically whether the
        // dialect is V3 or V2 (the pre-transform leaves it untouched).
        let src = r#"<div :id="x" @click="go">{{ msg }}</div>"#;
        assert_eq!(compile(src, VueVersion::V3), compile(src, VueVersion::V2));
    }

    // --- v-on event-modifier sugar (`.native`, numeric keycodes) ---

    fn directive_with_modifiers<'a>(
        allocator: &'a Allocator,
        modifiers: &[&'a str],
    ) -> DirectiveNode<'a> {
        let mut dir = DirectiveNode::new(allocator, "on", SourceLocation::STUB);
        let mut mods = ArenaVec::new_in(&allocator);
        for m in modifiers {
            mods.push(SimpleExpressionNode::new(m, false, SourceLocation::STUB));
        }
        dir.modifiers = mods;
        dir
    }

    /// Assert the directive's modifier list (by content, in order) equals
    /// `expected`. Stays on `&str` to avoid the crate's disallowed std
    /// `String`.
    fn assert_modifiers(dir: &DirectiveNode<'_>, expected: &[&str]) {
        assert_eq!(dir.modifiers.len(), expected.len());
        for (got, want) in dir.modifiers.iter().zip(expected) {
            assert_eq!(got.content, *want);
        }
    }

    #[test]
    fn strips_native_modifier() {
        let allocator = Allocator::new();
        let mut dir = directive_with_modifiers(&allocator, &["native"]);
        desugar_v2_v_on_modifiers(&mut dir);
        assert!(dir.modifiers.is_empty());
    }

    #[test]
    fn strips_native_keeps_other_modifiers() {
        let allocator = Allocator::new();
        let mut dir = directive_with_modifiers(&allocator, &["native", "stop"]);
        desugar_v2_v_on_modifiers(&mut dir);
        assert_modifiers(&dir, &["stop"]);
    }

    #[test]
    fn maps_common_numeric_keycodes() {
        let allocator = Allocator::new();
        for (code, name) in [
            ("8", "delete"),
            ("9", "tab"),
            ("13", "enter"),
            ("27", "esc"),
            ("32", "space"),
            ("37", "left"),
            ("38", "up"),
            ("39", "right"),
            ("40", "down"),
            ("46", "delete"),
        ] {
            let mut dir = directive_with_modifiers(&allocator, &[code]);
            desugar_v2_v_on_modifiers(&mut dir);
            assert_modifiers(&dir, &[name]);
        }
    }

    #[test]
    fn leaves_unmapped_numeric_keycode_untouched() {
        let allocator = Allocator::new();
        let mut dir = directive_with_modifiers(&allocator, &["65"]);
        desugar_v2_v_on_modifiers(&mut dir);
        assert_modifiers(&dir, &["65"]);
    }

    #[test]
    fn leaves_named_modifiers_untouched() {
        let allocator = Allocator::new();
        let mut dir = directive_with_modifiers(&allocator, &["stop", "prevent", "enter"]);
        desugar_v2_v_on_modifiers(&mut dir);
        assert_modifiers(&dir, &["stop", "prevent", "enter"]);
    }

    #[test]
    fn combined_native_and_keycode() {
        let allocator = Allocator::new();
        let mut dir = directive_with_modifiers(&allocator, &["native", "13"]);
        desugar_v2_v_on_modifiers(&mut dir);
        assert_modifiers(&dir, &["enter"]);
    }

    #[test]
    fn no_modifiers_is_noop() {
        let allocator = Allocator::new();
        let mut dir = directive_with_modifiers(&allocator, &[]);
        desugar_v2_v_on_modifiers(&mut dir);
        assert!(dir.modifiers.is_empty());
    }
}
