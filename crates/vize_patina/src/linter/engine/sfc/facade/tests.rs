//! The migrated rules preserve full diagnostics on the real SFC entry point.

use crate::diagnostic::Severity;
use crate::linter::Linter;
use crate::linter::engine::offset_result;
use vize_l0::ToCompactString;

fn linter() -> Linter {
    let mut linter = Linter {
        enabled_rules: Some(
            super::RULES
                .iter()
                .map(|rule| rule.to_compact_string())
                .collect(),
        ),
        ..Linter::default()
    };
    // Opt-in, so the happy-path registry would otherwise skip the comparison.
    linter
        .registry
        .register(Box::new(crate::rules::vue::NoStaticInlineStyles));
    linter.registry.register(Box::new(
        crate::rules::opinionated::vapor::PreferStaticClass,
    ));
    linter
        .registry
        .register(Box::new(crate::rules::vue::NoBareStringsInTemplate));
    linter.registry.mark_has_exit_element_rules();
    linter
}

fn compare_template(source: &str) {
    let prefix = "<script setup>const note = '日本語😀'</script>\r\n<template>";
    let sfc = format!("{prefix}{source}</template>");
    let linter = linter();
    let mut expected = linter.lint_template_relief(source, "test.vue");
    // A raw template is the baseline: only its file offset is adjusted.
    offset_result(&mut expected, prefix.len() as u32);
    let actual = linter.lint_sfc(&sfc, "test.vue");
    assert_eq!(
        actual.error_count, expected.error_count,
        "{source}\nactual: {:?}\nexpected: {:?}",
        actual.diagnostics, expected.diagnostics
    );
    assert_eq!(
        actual.warning_count, expected.warning_count,
        "{source}\nactual: {:?}\nexpected: {:?}",
        actual.diagnostics, expected.diagnostics
    );
    assert_eq!(
        format!("{:?}", actual.diagnostics),
        format!("{:?}", expected.diagnostics),
        "{source}"
    );
}

#[test]
fn migrated_sfc_rules_preserve_diagnostics_and_fixes() {
    for source in [
        r#"<div  style="color:red"><textarea>{{ message }}</textarea><button>go</button></div>"#,
        r#"<a href="https://example.com" target="_blank">go</a><iframe src="/frame"/><input disabled="true"/><div contenteditable="invalid"/>"#,
        r#"<a href="https://example.com" target="_blank" rel="noopener noreferrer">go</a><iframe sandbox=""/><button type="button">go</button><input disabled/>"#,
        r#"<Comp><template #default><textarea>{{ value }}</textarea><button>go</button></template></Comp>"#,
        "<!-- eslint-disable-next-line vue/no-inline-style -->\n<div style=\"color:red\"/>",
        r#"<MyComponent :foo="foo"/><div :title="'static'" :class="[{ active }, { hidden }]"/>"#,
        r#"<MyComponent :foo/><div :title :class/>"#,
        r#"<slot name="my-slot"/><input id="name"/><label for="name">Name</label>"#,
        r#"<MyComponent :title.sync="title" @click.native="handler"/><input @keyup.13="submit"/>"#,
        r#"<p>日本語😀</p><MyComponent :title.sync.camel="title" @click.native.stop="handler"/>"#,
        r#"<p>日本語😀</p><input @keyup.13.27="submit"/><input @keyup.0.13="submit"/><input @keyup.13suffix="submit"/>"#,
        r#"<Foo><template slot="header" slot-scope="props" scope="legacy">x</template><div :slot="name"/></Foo>"#,
        r#"<MyComponent inline-template/><router-link tag="button" :tag="el"/><div is="MyComponent" :is="something"/>"#,
        r#"<component/><div v-is="'MyComponent'"/>"#,
        r#"<button v-on:click.stop="submit"/><Card><template v-slot:header="props">x</template></Card>"#,
        r#"<MyComponent v-model.custom="value" v-html="content"/><component is="div" v-text="content"/>"#,
        r#"<MyComponent myProp="value" :otherProp="value"/><slot :el$="el$"/><template :bad$="bad$"/>"#,
        r#"<div v-pre:arg/><div v-pre.mod/><div v-pre="raw"/>"#,
        "<!-- eslint-disable-next-line vue/prefer-props-shorthand -->\n<MyComponent :foo=\"foo\"/>",
        "<!-- eslint-disable-next-line vue/no-useless-v-bind -->\n<div :title=\"'static'\"/>",
        "<!-- eslint-disable-next-line vue/no-multiple-objects-in-class -->\n<div :class=\"[{ active }, { hidden }]\"/>",
        "<!-- eslint-disable-next-line vue/slot-name-casing -->\n<slot name=\"my-slot\"/>",
        "<!-- eslint-disable-next-line vue/use-unique-element-ids -->\n<input id=\"name\"/>",
        "<!-- eslint-disable-next-line vue/no-deprecated-v-bind-sync -->\n<MyComponent :title.sync=\"title\"/>",
        "<!-- eslint-disable-next-line vue/no-deprecated-v-on-native-modifier -->\n<MyComponent @click.native=\"handler\"/>",
        "<!-- eslint-disable-next-line vue/no-deprecated-v-on-number-modifiers -->\n<input @keyup.13=\"submit\"/>",
        "<!-- eslint-disable-next-line vue/no-deprecated-slot-attribute -->\n<template slot=\"header\"/>",
        "<!-- eslint-disable-next-line vue/no-deprecated-slot-scope-attribute -->\n<template slot-scope=\"props\"/>",
        "<!-- eslint-disable-next-line vue/no-deprecated-scope-attribute -->\n<template scope=\"props\"/>",
        "<!-- eslint-disable-next-line vue/no-deprecated-inline-template -->\n<MyComponent inline-template/>",
        "<!-- eslint-disable-next-line vue/no-deprecated-router-link-tag-prop -->\n<router-link tag=\"button\"/>",
        "<!-- eslint-disable-next-line vue/no-deprecated-html-element-is -->\n<div is=\"MyComponent\"/>",
        "<!-- eslint-disable-next-line vue/require-component-is -->\n<component/>",
        "<!-- eslint-disable-next-line vue/no-deprecated-v-is -->\n<div v-is=\"'MyComponent'\"/>",
        "<!-- eslint-disable-next-line vue/v-on-style -->\n<button v-on:click=\"submit\"/>",
        "<!-- eslint-disable-next-line vue/v-slot-style -->\n<Card><template v-slot:header>header</template></Card>",
        "<!-- eslint-disable-next-line vue/no-custom-modifiers-on-v-model -->\n<MyComponent v-model.custom=\"value\"/>",
        "<!-- eslint-disable-next-line vue/no-v-text-v-html-on-component -->\n<MyComponent v-html=\"content\"/>",
        "<!-- eslint-disable-next-line vue/attribute-hyphenation -->\n<MyComponent myProp=\"value\"/>",
        "<!-- eslint-disable-next-line vue/valid-attribute-name -->\n<slot :el$=\"el$\"/>",
        "<!-- eslint-disable-next-line vue/valid-v-pre -->\n<div v-pre:arg/>",
        r#"<div v-pre><textarea>{{ raw }}</textarea></div>"#,
    ] {
        compare_template(source);
    }
}

#[test]
fn disabled_rules_and_severity_overrides_apply_to_the_batch() {
    let linter = linter()
        .with_disabled_rules(vec!["vue/no-inline-style".into()])
        .with_rule_severity_overrides(vec![("vue/no-textarea-mustache".into(), Severity::Warning)]);
    let source = "<template><textarea aria-label=\"Message\" style=\"color:red\">{{ value }}</textarea></template>";
    let result = linter.lint_sfc(source, "test.vue");
    assert!(
        result
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.rule_name != "vue/no-inline-style"),
        "{:?}",
        result.diagnostics
    );
    let mustache = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.rule_name == "vue/no-textarea-mustache")
        .expect("textarea mustache");
    assert_eq!(mustache.severity, Severity::Warning);
}

#[test]
fn sfc_batch_does_not_inspect_script_jsx() {
    let result = linter().lint_sfc(
        r#"<script setup lang="tsx">
const view = <button style="color:red">go</button>;
</script>"#,
        "test.vue",
    );
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
}

mod battery;
mod binding_order;
mod callback_order;
mod content_model;
mod stylesheet;
mod text_order;

struct FacadeOnly(&'static crate::rule::RuleMeta);

impl crate::markup::MarkupRule for FacadeOnly {
    fn name(&self) -> &'static str {
        self.0.name
    }
    fn enter_element<'a>(
        &self,
        ctx: &mut crate::markup::MarkupContext<'_, 'a>,
        _: &crate::markup::MarkupElement<'a>,
    ) {
        ctx.lint()
            .error_at("facade", crate::ir::ByteRange::new(0, 1));
    }
}

impl crate::rule::Rule for FacadeOnly {
    fn meta(&self) -> &'static crate::rule::RuleMeta {
        self.0
    }
    fn as_markup_rule(&self) -> Option<&dyn crate::markup::MarkupRule> {
        Some(self)
    }
    fn enter_element<'a>(
        &self,
        _: &mut crate::context::LintContext<'a>,
        _: &vize_relief::ElementNode<'a>,
    ) {
        panic!("an admitted SFC rule reached the legacy element visitor");
    }
}

#[test]
fn every_admitted_rule_reaches_only_the_l2_facade() {
    let mut all = crate::rule::RuleRegistry::with_all();
    all.register_opt_in_rules();
    for name in super::RULES {
        let meta = all
            .rules()
            .iter()
            .find(|rule| rule.meta().name == *name)
            .unwrap()
            .meta();
        let mut registry = crate::rule::RuleRegistry::new();
        registry.register(Box::new(FacadeOnly(meta)));
        let result =
            Linter::with_registry(registry).lint_sfc("<template><div/></template>", "test.vue");
        assert_eq!(result.diagnostics.len(), 1, "{name}");
        assert_eq!(result.diagnostics[0].rule_name, *name);
        assert_eq!(result.diagnostics[0].message, "facade");
    }
}
