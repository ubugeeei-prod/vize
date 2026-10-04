use super::super::super::DescriptorIssueCode as Code;
use super::*;

#[test]
fn historical_profile_is_explicit_and_existing_vue2_and_modern_entries_are_unchanged() {
    let arena = Allocator::default();
    let source = "<template>{{ value }}</template>";
    for version in VueVersion::ALL {
        let selected = DescriptorOptions {
            version,
            ..options()
        };
        let historical = Vue.observe_vue1_descriptor(&arena, source, selected);
        let vue2 = Vue.observe_vue2_descriptor(&arena, source, selected);
        let modern = Vue.observe_descriptor(&arena, source, selected);
        assert_eq!(historical.selected().is_ok(), version == VueVersion::V1);
        assert_eq!(vue2.selected().is_ok(), version == VueVersion::V2);
        assert_eq!(modern.admitted().is_ok(), version == VueVersion::V3);
        if version != VueVersion::V1 {
            assert!(historical.component().is_none());
            assert!(
                historical
                    .issues()
                    .iter()
                    .any(|issue| issue.code == Code::UnsupportedVersion)
            );
        }
    }
    for (selected, expected) in [
        (
            DescriptorOptions {
                dialect: VueDialect::PetiteVue,
                ..options()
            },
            Code::UnsupportedDialect,
        ),
        (
            DescriptorOptions {
                template: SurfaceParseOptions {
                    experimental_in_tag_comments: true,
                },
                ..options()
            },
            Code::UnsupportedOptions,
        ),
    ] {
        let owner = Vue.observe_vue1_descriptor(&arena, source, selected);
        assert_eq!(owner.options(), selected);
        assert!(owner.selected().is_err() && owner.component().is_none());
        assert!(owner.issues().iter().any(|issue| issue.code == expected));
    }
    let modern_source = "<template>{{ value }}</template><script setup lang=ts>let value=1</script><style scoped>.x{}</style>";
    let modern = Vue.observe_descriptor(
        &arena,
        modern_source,
        DescriptorOptions {
            version: VueVersion::V3,
            ..options()
        },
    );
    let admitted = modern.admitted().unwrap();
    assert!(admitted.setup().is_some() && admitted.styles().len() == 1);
    assert_eq!(admitted.template_lang(), crate::embed::Lang::Ts);
}

#[test]
fn later_unsupported_slots_retain_whole_original_capture_without_component_entry() {
    let arena = Allocator::default();
    for (suffix, expected) in [
        ("<script></script>", Code::UnsupportedScript),
        ("<script>//keep</script>", Code::UnsupportedScript),
        (
            "<script setup lang=ts>const x=1</script>",
            Code::UnsupportedScript,
        ),
        ("<script src='external.js'/>", Code::UnsupportedScript),
        ("<style></style>", Code::UnsupportedStyle),
        (
            "<style lang=scss scoped>.x{}</style>",
            Code::UnsupportedStyle,
        ),
        ("<custom>retained</custom>", Code::UnsupportedBlock),
        ("<template>second</template>", Code::DuplicateRole),
    ] {
        let source = vize_l0::cstr!("<template>{{{{ value }}}}</template>{suffix}");
        let capture = Vue.split(&arena, &source);
        let measured = hooks::Armed::new(hooks::Fault::None);
        let owner = Vue.observe_vue1_descriptor(&arena, &source, options());
        let refusal = owner.selected().unwrap_err();
        assert!(owner.component().is_none());
        assert!(
            refusal
                .issues()
                .iter()
                .any(|issue| issue.code == expected && issue.container_index == Some(1)),
            "{source}: {:?}",
            owner.issues()
        );
        assert!(core::ptr::eq(refusal.issues(), owner.issues()));
        assert!(core::ptr::eq(refusal.errors(), &*owner.container().errors));
        assert_eq!(
            (measured.counts().splitters, measured.counts().components),
            (1, 0)
        );
        assert_eq!(owner.container().blocks.len(), capture.blocks.len());
        assert_eq!(&*owner.container().errors, &*capture.errors);
        for (actual, original) in owner.container().blocks.iter().zip(capture.blocks.iter()) {
            assert!(core::ptr::eq(actual.name, original.name));
            assert_eq!(
                (actual.open_tag, actual.content, actual.close_tag),
                (original.open_tag, original.content, original.close_tag)
            );
            assert_eq!(&*actual.attrs, &*original.attrs);
        }
        drop(owner);
        assert_eq!(measured.counts().dropped, 1);
    }
}

#[test]
fn emitted_outer_attributes_refuse_as_one_original_header_without_reclassification() {
    let arena = Allocator::default();
    for source in [
        "<template src='x.html'></template>",
        "<template lang=html>x</template>",
        "<template lang='h&#116;ml'>x</template>",
        "<template lang=html lang=html>x</template>",
        "<template functional>x</template>",
        "<template lang='html'id=x>x</template>",
    ] {
        let measured = hooks::Armed::new(hooks::Fault::None);
        let owner = Vue.observe_vue1_descriptor(&arena, source, options());
        assert!(owner.selected().is_err() && owner.component().is_none());
        let block = owner.container().blocks.first().unwrap();
        assert!(!block.attrs.is_empty());
        let mut headers = owner
            .issues()
            .iter()
            .filter(|issue| issue.container_index == Some(0));
        assert_eq!(
            headers.next(),
            Some(&DescriptorIssue {
                code: Code::UnsupportedAttribute,
                container_index: Some(0),
                span: block.open_tag
            })
        );
        assert!(headers.next().is_none());
        assert!(
            owner
                .issues()
                .iter()
                .any(|issue| issue.code == Code::MissingComponentBlock)
        );
        assert_eq!(
            (measured.counts().splitters, measured.counts().components),
            (1, 0)
        );
        drop(owner);
        assert_eq!(measured.counts().dropped, 1);
    }
    for (source, expected) in [
        ("plain outside source", Code::MissingComponentBlock),
        ("<template/>", Code::UnsupportedBoundary),
        ("<Template>x</Template>", Code::UnsupportedBlockSpelling),
        ("<template>x", Code::UnsupportedBoundary),
        ("<template>{{ value</template>", Code::UnsupportedBoundary),
    ] {
        let measured = hooks::Armed::new(hooks::Fault::None);
        let owner = Vue.observe_vue1_descriptor(&arena, source, options());
        assert!(owner.selected().is_err() && owner.component().is_none());
        assert!(
            owner.issues().iter().any(|issue| issue.code == expected),
            "{source}: {:?}",
            owner.issues()
        );
        assert_eq!(
            (measured.counts().splitters, measured.counts().components),
            (1, 0)
        );
    }
}

#[test]
fn selected_envelope_preserves_original_body_holes_boundaries_recovery_and_comments() {
    let arena = Allocator::default();
    for (body, expected) in [
        (
            "{{{ value }}}",
            TextRefusal::Boundary(TextBoundaryKind::RawInterpolation),
        ),
        (
            "{{ }}",
            TextRefusal::Boundary(TextBoundaryKind::EmptyInterpolation),
        ),
        (
            "{{ /*kept*/ value + }}",
            TextRefusal::NativeHole(EmbedHole::Syntax),
        ),
        (
            "{{ value | upper }}",
            TextRefusal::Boundary(TextBoundaryKind::PipeSyntax),
        ),
        (
            "{{ left || right }}",
            TextRefusal::Boundary(TextBoundaryKind::PipeSyntax),
        ),
        (
            "{{ 'literal|pipe' }}",
            TextRefusal::Boundary(TextBoundaryKind::PipeSyntax),
        ),
        (
            "{{*value}}",
            TextRefusal::Boundary(TextBoundaryKind::OneTimeInterpolation),
        ),
        (
            "{{ value\r\n }}",
            TextRefusal::Boundary(TextBoundaryKind::HistoricalLineSeparator),
        ),
        (
            "{{ value\u{2028} }}",
            TextRefusal::Boundary(TextBoundaryKind::HistoricalLineSeparator),
        ),
        (
            "{{ value\u{2029} }}",
            TextRefusal::Boundary(TextBoundaryKind::HistoricalLineSeparator),
        ),
        (
            "&#123;&#123; value &#125;&#125;{{ next }}",
            TextRefusal::Boundary(TextBoundaryKind::EncodedDelimiter),
        ),
    ] {
        let source = vize_l0::cstr!("<template>{body}</template>");
        let owner = Vue.observe_vue1_descriptor(&arena, &source, options());
        let component = owner.selected().unwrap().component();
        assert_eq!(check_fidelity(component.tree()), Ok(()));
        let child = component
            .children()
            .find(|child| matches!(child.surface(), SurfaceChild::Interpolation(_)))
            .unwrap();
        assert_eq!(component.text_for(child).unwrap_err(), expected);
        let binding = component.bindings().first().unwrap();
        assert_eq!(binding.content_span().slice(&source), binding.raw_content());
        assert!(core::ptr::eq(
            binding.source().unwrap().authored_root(),
            source.as_str()
        ));
        if expected == TextRefusal::NativeHole(EmbedHole::Syntax) {
            let syntax = binding.syntax().unwrap();
            assert!(syntax.expression().is_none() && syntax.diagnostics().count() > 0);
            assert_eq!(
                syntax.comments().next().unwrap().text().unwrap(),
                "/*kept*/"
            );
            for diagnostic in syntax.diagnostics() {
                for label in diagnostic.labels() {
                    if let Ok(span) = label.authored_span() {
                        assert!(owner.selected().unwrap().block().contains_block_span(span));
                    }
                }
            }
        } else if !component.text_boundaries().is_empty() {
            assert_eq!(component.text_boundaries().len(), 4);
            assert!(
                component
                    .text_boundaries()
                    .iter()
                    .all(|boundary| boundary.kind == TextBoundaryKind::EncodedDelimiter)
            );
            assert!(binding.boundary().is_none());
            assert!(binding.syntax().unwrap().hole().is_none());
        } else {
            assert!(binding.syntax().is_none());
        }
    }
    let owner = Vue.observe_vue1_descriptor(
        &arena,
        "<template><a><b>{{ value }}</b></template>",
        options(),
    );
    let component = owner.selected().unwrap().component();
    let child = component
        .children()
        .next()
        .unwrap()
        .children()
        .unwrap()
        .next()
        .unwrap()
        .children()
        .unwrap()
        .next()
        .unwrap();
    assert_eq!(
        component.text_for(child).unwrap_err(),
        TextRefusal::RecoveredComponent
    );
    let owner = Vue.observe_vue1_descriptor(
        &arena,
        "<template><i v-pre>{{ value }}</i></template>",
        options(),
    );
    let component = owner.selected().unwrap().component();
    let child = component
        .children()
        .next()
        .unwrap()
        .children()
        .unwrap()
        .next()
        .unwrap();
    assert!(matches!(child.surface(), SurfaceChild::Text(_)));
    assert_eq!(
        component.text_for(child).unwrap_err(),
        TextRefusal::NotInterpolation
    );
}
