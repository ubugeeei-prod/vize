use super::support::{LOCALES, complete, error, expected, parser, registered, selected, span};
use vize_l0::{Allocator, Span, cstr};
use vize_l1::markup::NativeLintTagRefusal;
use vize_patina::native::{
    NativeLintRefusal, NativeSyntaxLint, header_facts::NativeHeaderFactError,
};

#[test]
fn late_dynamic_object_and_custom_headers_refuse_before_marker_lookup() {
    for (head, unresolved) in [
        ("v-bind", true),
        (":[key]", true),
        ("v-bind:[key]", true),
        ("@[key]", true),
        ("v-unknown", false),
    ] {
        let source =
            cstr!("<template><textarea title='safe' {head}='x'>{{{{x}}}}</textarea></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let range = span(&source, head);
        let refusal = if unresolved {
            NativeLintRefusal::UnresolvedBinding { span: range }
        } else {
            NativeLintRefusal::UnsupportedDirective { span: range }
        };
        assert_eq!(
            lint.header_facts(&element).err(),
            Some(NativeHeaderFactError::Header(refusal))
        );
        for locale in LOCALES {
            assert_eq!(
                complete(&registered(&source, locale)),
                expected(vec![error(locale, span(&source, "{{x}}"))])
            );
        }
    }
}

#[test]
fn malformed_modifier_refusal_preserves_parser_and_product_errors_in_order() {
    for head in [
        ":title.",
        ":title..prop",
        ".title.",
        "@click.",
        "v-bind:title..camel",
    ] {
        let source = cstr!("<template><textarea {head}='x'>{{{{x}}}}</textarea></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        let range = span(&source, head);
        assert_eq!(
            lint.header_facts(&element).err(),
            Some(NativeHeaderFactError::Header(
                NativeLintRefusal::UnsupportedDirective { span: range }
            ))
        );
        let start = range.start
            + u32::try_from(head.find("..").map_or(head.len(), |index| index + 1)).unwrap();
        for locale in LOCALES {
            assert_eq!(
                complete(&registered(&source, locale)),
                expected(vec![
                    parser(
                        "error",
                        "Directive modifier is expected.",
                        Span::new(start, start + 1)
                    ),
                    error(locale, span(&source, "{{x}}")),
                ])
            );
        }
    }
}

#[test]
fn repeated_attribute_refusal_retains_complete_original_warning_then_marker_error() {
    let source = "<template><textarea title='one' TITLE='two'>{{x}}</textarea></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let element = owner.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    let repeated = span(source, "TITLE");
    assert_eq!(
        lint.header_facts(&element).err(),
        Some(NativeHeaderFactError::Header(
            NativeLintRefusal::DuplicateAttribute { span: repeated }
        ))
    );
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![
                parser(
                    "warning",
                    "Duplicate attribute `TITLE`. Keeping the repeated attribute so parsing can continue.",
                    repeated
                ),
                error(locale, span(source, "{{x}}")),
            ])
        );
    }
}

#[test]
fn modified_pre_requires_ambiguous_refusal_even_when_native_child_is_literal_text() {
    for head in ["v-pre.foo", "v-pre:argument", "v-pre:[argument]"] {
        let source = cstr!("<template><textarea {head}>{{{{x}}}}</textarea></template>");
        let arena = Allocator::default();
        let owner = selected(&arena, &source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        assert_eq!(
            lint.header_facts(&element).err(),
            Some(NativeHeaderFactError::Header(NativeLintRefusal::LintTag {
                reason: NativeLintTagRefusal::AmbiguousVerbatim
            }))
        );
        for locale in LOCALES {
            assert_eq!(
                complete(&registered(&source, locale)),
                expected(vec![error(locale, span(&source, "{{x}}"))])
            );
        }
    }
}

#[test]
fn inherited_modified_pre_cannot_grant_false_marker_absence() {
    let source = "<template><div v-pre:argument><textarea>{{x}}</textarea></div></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    let parent = owner.children().next().unwrap().into_element().unwrap();
    let element = parent.children().next().unwrap().into_element().unwrap();
    let lint = NativeSyntaxLint::new(&owner).unwrap();
    assert_eq!(
        lint.header_facts(&element).err(),
        Some(NativeHeaderFactError::Header(NativeLintRefusal::LintTag {
            reason: NativeLintTagRefusal::AmbiguousVerbatim
        }))
    );
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![error(locale, span(source, "{{x}}"))])
        );
    }
}

#[test]
fn missing_original_parent_close_retains_parser_error_without_claiming_body_clean() {
    for (source, tag, product) in [
        ("<template><div>{{x}}</template>", "<div>", false),
        ("<template><textarea>{{x}}</template>", "<textarea>", true),
    ] {
        let arena = Allocator::default();
        let owner = selected(&arena, source);
        let element = owner.children().next().unwrap().into_element().unwrap();
        let lint = NativeSyntaxLint::new(&owner).unwrap();
        assert_eq!(
            lint.header_facts(&element).err(),
            Some(NativeHeaderFactError::Header(NativeLintRefusal::Hole))
        );
        for locale in LOCALES {
            let mut diagnostics = vec![parser(
                "error",
                "Element is missing end tag.",
                span(source, tag),
            )];
            if product {
                diagnostics.push(error(locale, span(source, "{{x}}")));
            }
            assert_eq!(complete(&registered(source, locale)), expected(diagnostics));
        }
    }
}

#[test]
fn unsupported_descriptor_version_dialect_and_options_never_mint_marker_custody() {
    use vize_l0::config::{VueDialect, VueVersion};
    use vize_l1::{
        SurfaceParseOptions,
        container::{
            Vue,
            vue::{DescriptorIssue, DescriptorIssueCode, DescriptorOptions},
        },
    };
    let source = "<template><textarea>{{x}}</textarea></template>";
    let supported = DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    };
    for (options, code) in [
        (
            DescriptorOptions {
                version: VueVersion::V2_7,
                ..supported
            },
            DescriptorIssueCode::UnsupportedVersion,
        ),
        (
            DescriptorOptions {
                dialect: VueDialect::PetiteVue,
                ..supported
            },
            DescriptorIssueCode::UnsupportedDialect,
        ),
        (
            DescriptorOptions {
                template: SurfaceParseOptions {
                    experimental_in_tag_comments: true,
                },
                ..supported
            },
            DescriptorIssueCode::UnsupportedOptions,
        ),
    ] {
        let arena = Allocator::default();
        let observation = Vue.observe_descriptor(&arena, source, options);
        assert_eq!(observation.options(), options);
        assert_eq!(observation.source(), source);
        assert_eq!(
            observation.issues(),
            [DescriptorIssue {
                code,
                container_index: None,
                span: Span::new(0, 0)
            }]
        );
        assert_eq!(
            observation.admitted().err().unwrap().issues(),
            observation.issues()
        );
        for locale in LOCALES {
            assert_eq!(
                complete(&registered(source, locale)),
                expected(vec![error(locale, span(source, "{{x}}"))])
            );
        }
    }
}

#[test]
fn unterminated_marker_refuses_recovery_and_retains_original_parser_only_order() {
    let source = "<template><textarea>{{x</textarea></template>";
    let arena = Allocator::default();
    let owner = selected(&arena, source);
    assert_eq!(owner.component().block().span(), Span::new(10, 34));
    assert_eq!(
        NativeSyntaxLint::new(&owner).err(),
        Some(NativeLintRefusal::Recovered { offset: 34 })
    );
    // A parser-only result retains original lexical-then-finalization order.
    // No source sorting or synthetic marker can replace that complete vector.
    for locale in LOCALES {
        assert_eq!(
            complete(&registered(source, locale)),
            expected(vec![
                parser(
                    "error",
                    "Interpolation is missing its closing delimiter `}}`; treating the unfinished interpolation as text.",
                    Span::new(33, 34)
                ),
                parser("error", "Element is missing end tag.", Span::new(10, 20)),
            ])
        );
    }
}

#[test]
fn missing_outer_template_boundary_preserves_descriptor_refusal_and_original_empty_route() {
    use vize_l0::config::{VueDialect, VueVersion};
    use vize_l1::{
        SurfaceParseOptions,
        container::{
            ContainerError, ContainerErrorCode, Vue,
            vue::{DescriptorIssue, DescriptorIssueCode, DescriptorOptions},
        },
    };
    let source = "<template><textarea>{{x}}</textarea>";
    let arena = Allocator::default();
    let observation = Vue.observe_descriptor(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    assert_eq!(
        observation.issues(),
        [DescriptorIssue {
            code: DescriptorIssueCode::UnsupportedBoundary,
            container_index: Some(0),
            span: Span::new(0, 10),
        }]
    );
    assert_eq!(
        observation.container().errors.as_slice(),
        [ContainerError {
            code: ContainerErrorCode::MissingCloseTag,
            offset: 0
        }]
    );
    let refusal = observation.admitted().err().unwrap();
    assert_eq!(refusal.issues(), observation.issues());
    assert_eq!(refusal.errors(), observation.container().errors.as_slice());
    let block = &observation.container().blocks[0];
    assert_eq!(block.open_tag, Span::new(0, 10));
    assert_eq!(block.content, Span::new(10, 36));
    assert_eq!(block.close_tag, None);
    for locale in LOCALES {
        assert_eq!(complete(&registered(source, locale)), expected(vec![]));
    }
}
