use super::support::*;
use std::sync::atomic::Ordering;
use vize_l0::{String, cstr};
use vize_l1::markup::{ArgSyntax, DirectivePrefix};
use vize_patina::{HelpLevel, Locale, native::template::NativeTemplateAttributeProfile as Profile};

#[test]
fn actual_static_attribute_tokens_ordinals_opaque_values_and_unicode_ranges_are_preserved() {
    let refused =
        "前\r\n<div disabled title = \"&amp; 界\" data-id='{{ opaque }}'><span lang=en/></div>";
    let source =
        "前\r\n<div disabled title = \"&amp; 界\" data-id='{{ opaque }}'><span lang=en /></div>";
    for locale in LOCALES {
        let log = events();
        let rule = Audit::new(&FIRST, "actual", Profile::StaticOnly, log.clone());
        let observations = rule.observations.clone();
        let configured = linter(vec![rule], locale, HelpLevel::Full);
        unquoted::refused_original(&configured, refused, &log);
        log.lock().unwrap().clear();
        observations.lock().unwrap().clear();
        let result = pair(&configured, source, "/元/exact.vue");
        assert_eq!(result.diagnostics.len(), 5);
        let observed = observations.lock().unwrap();
        let parent = &observed[0];
        let child = &observed[1];
        assert_eq!(
            (&*parent.tag, parent.ordinal, &parent.parent),
            ("div", 1, &None)
        );
        assert_eq!(
            (&*child.tag, child.ordinal, child.parent.as_deref()),
            ("span", 0, Some("div"))
        );
        for (actual, (raw, full, value)) in parent.attributes.iter().zip([
            ("disabled", "disabled", None),
            ("title", "title = \"&amp; 界\"", Some("&amp; 界")),
            ("data-id", "data-id='{{ opaque }}'", Some("{{ opaque }}")),
        ]) {
            assert_eq!(actual.name, raw);
            assert_eq!(actual.range, span(source, full));
            assert_eq!(actual.value.as_deref(), value);
            assert_eq!(actual.head, None);
            assert_eq!(actual.binding, "static");
            assert_eq!(actual.argument, None);
        }
        assert_eq!(
            parent
                .attributes
                .iter()
                .map(|a| a.ordinal)
                .collect::<Vec<_>>(),
            [0, 1, 2]
        );
        assert_eq!(child.attributes[0].name, "lang");
        assert_eq!(child.attributes[0].range, span(source, "lang=en"));
        assert_eq!(child.attributes[0].value.as_deref(), Some("en"));
    }
}

mod unquoted;

#[test]
fn fixed_bind_prop_full_names_modifiers_and_non_ascii_arguments_retain_original_head_geometry() {
    let source = "界\r\n<div :title.camel='opaque' .value='raw' v-bind:属性.prop='&amp;'/>";
    let rule = Audit::new(&FIRST, "actual", Profile::Bindings, events());
    let observations = rule.observations.clone();
    pair(
        &linter(vec![rule], Locale::En, HelpLevel::Full),
        source,
        "exact.vue",
    );
    let observed = observations.lock().unwrap();
    for (actual, (head, argument, prefix, name, modifiers, value)) in
        observed[0].attributes.iter().zip([
            (
                ":title.camel",
                "title",
                DirectivePrefix::Bind,
                None,
                ".camel",
                "opaque",
            ),
            (".value", "value", DirectivePrefix::Prop, None, "", "raw"),
            (
                "v-bind:属性.prop",
                "属性",
                DirectivePrefix::Full,
                Some("bind"),
                ".prop",
                "&amp;",
            ),
        ])
    {
        let geometry = actual.head.unwrap();
        assert_eq!(geometry.prefix, prefix);
        assert_eq!(
            geometry.arg,
            Some(ArgSyntax::Static(span(source, argument)))
        );
        assert_eq!(actual.argument, Some(span(source, argument)));
        assert_eq!(actual.name, head);
        assert_eq!(actual.binding, "bind");
        assert_eq!(actual.value.as_deref(), Some(value));
        if let Some(name) = name {
            assert_eq!(geometry.name, span(source, name));
        } else {
            assert_eq!(
                geometry.name,
                vize_l0::Span::new(span(source, head).start, span(source, head).start)
            );
        }
        let end = span(source, head).end;
        assert_eq!(
            geometry.modifiers,
            if modifiers.is_empty() {
                vize_l0::Span::new(end, end)
            } else {
                span(source, modifiers)
            }
        );
    }
}

#[test]
fn nested_quoted_and_template_literal_dynamic_heads_are_lexical_observations_not_resolved_names() {
    for (head, argument, prefix) in [
        (":[keys['a.b']].camel", "keys['a.b']", DirectivePrefix::Bind),
        (
            ".[keys[indices[index]]].prop",
            "keys[indices[index]]",
            DirectivePrefix::Prop,
        ),
        (
            "v-bind:[`prefix${keys['[']}`].camel",
            "`prefix${keys['[']}`",
            DirectivePrefix::Full,
        ),
    ] {
        let source = cstr!("界\r\n<div {head}=\"opaque\"></div>");
        for locale in LOCALES {
            let rule = Audit::new(&FIRST, "actual", Profile::Bindings, events());
            let observations = rule.observations.clone();
            pair(
                &linter(vec![rule], locale, HelpLevel::Full),
                &source,
                "exact.vue",
            );
            let observed = observations.lock().unwrap();
            let attribute = &observed[0].attributes[0];
            assert_eq!(attribute.name, head);
            assert_eq!(attribute.binding, "dynamic-bind");
            assert_eq!(attribute.range, span(&source, &cstr!("{head}=\"opaque\"")));
            assert_eq!(attribute.argument, Some(span(&source, argument)));
            assert_eq!(
                attribute.head.unwrap().arg,
                Some(ArgSyntax::Dynamic(span(&source, argument)))
            );
            assert_eq!(attribute.head.unwrap().prefix, prefix);
        }
    }
}

#[test]
fn opaque_values_and_lexically_complete_invalid_javascript_do_not_gain_semantic_validity_claims() {
    for source in [
        "<div :[key+].camel='(()'></div>",
        "<div :title='{{ malformed + }}'></div>",
        "<div v-bind:[keys[index]]='&amp;'></div>",
    ] {
        for locale in LOCALES {
            let rule = Audit::new(&FIRST, "actual", Profile::Bindings, events());
            let observations = rule.observations.clone();
            let result = pair(
                &linter(vec![rule], locale, HelpLevel::Full),
                source,
                "exact.vue",
            );
            assert_eq!(result.diagnostics.len(), 2);
            assert_eq!(observations.lock().unwrap()[0].attributes.len(), 1);
        }
    }
}

#[test]
fn wide_nested_headers_preserve_full_original_output_and_preorder_without_allocation_free_credit() {
    let attributes = |prefix: char| {
        let mut header = String::new("");
        for n in 0..12 {
            header.push_str(cstr!(" {prefix}{n}='{prefix}{n}'").as_str());
        }
        header
    };
    let source = cstr!(
        "<div{}><section{}><span{}/></section></div>",
        attributes('a'),
        attributes('b'),
        attributes('c')
    );
    let rule = Audit::new(&FIRST, "actual", Profile::StaticOnly, events());
    let observations = rule.observations.clone();
    let result = pair(
        &linter(vec![rule], Locale::En, HelpLevel::Full),
        &source,
        "exact.vue",
    );
    assert_eq!(result.diagnostics.len(), 37);
    let observed = observations.lock().unwrap();
    assert_eq!(
        observed[..3]
            .iter()
            .map(|e| e.tag.as_str())
            .collect::<Vec<_>>(),
        ["div", "section", "span"]
    );
    for (element, prefix) in observed[..3].iter().zip(['a', 'b', 'c']) {
        assert_eq!(element.attributes.len(), 12);
        for (ordinal, attribute) in element.attributes.iter().enumerate() {
            assert_eq!(attribute.ordinal, ordinal);
            assert_eq!(
                attribute.range,
                span(&source, &cstr!("{prefix}{ordinal}='{prefix}{ordinal}'"))
            );
        }
    }
}

#[test]
fn duplicate_registered_names_preserve_actual_instance_order_and_each_profile_contribution() {
    let log = events();
    let one = Audit::new(&FIRST, "one", Profile::Bindings, log.clone());
    let two = Audit::new(&FIRST, "two", Profile::Bindings, log.clone());
    let calls = [one.profile_calls.clone(), two.profile_calls.clone()];
    let result = pair(
        &linter(vec![one, two], Locale::En, HelpLevel::Full),
        "<div :x='v' y='w'/>",
        "exact.vue",
    );
    assert_eq!(result.diagnostics.len(), 6);
    assert_eq!(
        result
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.message.as_str())
            .collect::<Vec<_>>(),
        [
            "Component file name 'one/root' should be PascalCase or kebab-case",
            "Component file name 'two/root' should be PascalCase or kebab-case",
            "Component file name 'one' should be PascalCase or kebab-case",
            "Component file name 'one' should be PascalCase or kebab-case",
            "Component file name 'two' should be PascalCase or kebab-case",
            "Component file name 'two' should be PascalCase or kebab-case",
        ]
    );
    for counter in calls {
        assert_eq!(counter.load(Ordering::SeqCst), 2);
    }
    let configured = linter(
        vec![
            Audit::new(&FIRST, "one", Profile::Bindings, events()),
            Audit::new(&FIRST, "two", Profile::StaticOnly, events()),
        ],
        Locale::En,
        HelpLevel::Full,
    );
    let source = "<div :x='v'/>";
    assert_eq!(
        configured
            .lint_native_template(source, "exact.vue")
            .unwrap_err(),
        vize_patina::native::template::NativeTemplateLintRefusal::UnsupportedAttribute {
            span: span(source, ":x='v'")
        }
    );
}
