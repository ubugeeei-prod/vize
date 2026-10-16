use super::*;

/// Derive the corrected kinds from the untouched original template bytes,
/// independently of both the registered L2 facade and native header facts.
fn assert_original_parser_kinds(source: &str, actual: &[Observation]) {
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let (root, errors) =
        vize_armature::Parser::new(&arena, owner.component().block().source()).parse();
    assert!(errors.is_empty(), "{errors:?}");
    let document =
        vize_patina::markup::MarkupDocument::new(&root, vize_patina::ir::TemplateSyntax::Vue);
    let mut expected = Vec::new();
    document
        .walk_elements(&mut |element| expected.push((element.tag().to_owned(), element.kind())));
    let actual = actual
        .iter()
        .map(|item| (item.tag.clone(), item.kind))
        .collect::<Vec<_>>();
    assert_eq!(actual, expected, "original parser kinds for {source}");
}

#[test]
fn exact_and_inherited_pre_keep_real_registered_output_and_component_exemptions() {
    let source = "<template><section v-pre autofocus><template v-if='ok' autofocus><slot autofocus></slot><Foo autofocus/></template><input :autofocus='ignored' autofocus /></section><input :autofocus='enabled' /></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let section = owner.children().next().unwrap().into_element().unwrap();
    let template = section.children().next().unwrap().into_element().unwrap();
    assert!(matches!(
        template.lint_tag(),
        Err(NativeLintTagRefusal::InheritedTemplate)
    ));
    for locale in [Locale::En, Locale::Ja, Locale::Zh] {
        let (output, result) = observed(source, locale, &owner);
        assert_original_parser_kinds(source, &output);
        assert_eq!(
            output
                .iter()
                .map(|item| (item.tag.as_str(), item.kind))
                .collect::<Vec<_>>(),
            [
                ("section", MarkupElementKind::Element),
                ("template", MarkupElementKind::Element),
                ("slot", MarkupElementKind::Slot),
                ("Foo", MarkupElementKind::Component),
                ("input", MarkupElementKind::Element),
                ("input", MarkupElementKind::Element),
            ]
        );
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 5);
    }
    for attributes in [
        "v-pre.foo v-pre",
        "v-pre v-pre.foo",
        "v-pre:arg v-pre",
        "v-pre v-pre:arg",
    ] {
        let source = cstr!(
            "<template><template {attributes} v-if='ok' autofocus><Foo autofocus/><input :autofocus='opaque' autofocus /></template></template>"
        );
        let (output, result) = parity(&source, Locale::En);
        assert_original_parser_kinds(&source, &output);
        assert_eq!(output[0].kind, MarkupElementKind::Element);
        assert_eq!(output[1].kind, MarkupElementKind::Component);
        assert_eq!(result.error_count, 0);
        assert_eq!(result.warning_count, 2);
    }
}

#[test]
fn modified_pre_refusals_preserve_the_unfiltered_registered_oracle_control() {
    for head in ["v-pre.foo", "v-pre:arg", "v-pre:[key]"] {
        let source = cstr!(
            "<template><template {head} v-if='ok'><input :autofocus='enabled' /></template><input autofocus /></template>"
        );
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let template = owner.children().next().unwrap().into_element().unwrap();
        assert!(matches!(
            template.lint_tag(),
            Err(NativeLintTagRefusal::AmbiguousVerbatim)
        ));
        assert!(template.surface().open.is_verbatim());
        let child = template.children().next().unwrap().into_element().unwrap();
        assert!(matches!(
            child.lint_tag(),
            Err(NativeLintTagRefusal::AmbiguousVerbatim)
        ));
        assert_eq!(child.surface().open.attrs[0].name.text, ":autofocus");
        let outside = owner.children().nth(1).unwrap().into_element().unwrap();
        assert_eq!(
            outside.lint_tag().unwrap().kind(),
            NativeLintTagKind::Element
        );
        for locale in [Locale::En, Locale::Ja, Locale::Zh] {
            let (output, result) = observed(&source, locale, &owner);
            assert_original_parser_kinds(&source, &output);
            assert_eq!(
                output
                    .iter()
                    .map(|item| (item.tag.as_str(), item.kind))
                    .collect::<Vec<_>>(),
                [
                    ("template", MarkupElementKind::Element),
                    ("input", MarkupElementKind::Element),
                    ("input", MarkupElementKind::Element),
                ]
            );
            assert_eq!(result.error_count, 0);
            assert_eq!(result.warning_count, 1);
            assert_eq!(result.diagnostics.len(), 1);
            assert_eq!(
                result.diagnostics[0].start,
                source.find("autofocus />").unwrap() as u32
            );
        }
        assert_eq!(check_fidelity(&owner.component().carrier().tree), Ok(()));
        assert!(owner.component().carrier().unsupported.is_empty());
    }
}
