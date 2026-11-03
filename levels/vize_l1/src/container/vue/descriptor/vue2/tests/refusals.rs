use super::super::super::DescriptorIssueCode as Code;
use super::*;

#[test]
fn every_wrong_profile_refuses_without_changing_the_modern_entry() {
    let arena = Allocator::default();
    let source = "<template>{{ value | upper }}</template>";
    for version in VueVersion::ALL {
        let mut selected = options();
        selected.version = version;
        let historical = Vue.observe_vue2_descriptor(&arena, source, selected);
        let modern = Vue.observe_descriptor(&arena, source, selected);
        assert_eq!(historical.selected().is_ok(), version == VueVersion::V2);
        assert_eq!(modern.admitted().is_ok(), version == VueVersion::V3);
        if version != VueVersion::V2 {
            assert!(historical.component().is_none());
            assert!(
                historical
                    .issues()
                    .iter()
                    .any(|i| i.code == Code::UnsupportedVersion)
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
        let owner = Vue.observe_vue2_descriptor(&arena, source, selected);
        assert!(owner.selected().is_err() && owner.component().is_none());
        assert!(owner.issues().iter().any(|i| i.code == expected));
    }
    let modern_options = DescriptorOptions {
        version: VueVersion::V3,
        ..options()
    };
    let source = "<template>{{ value }}</template><script setup lang=ts>let value=1</script><style scoped>.x{}</style>";
    let modern = Vue.observe_descriptor(&arena, source, modern_options);
    let admitted = modern.admitted().unwrap();
    assert!(admitted.setup().is_some() && admitted.styles().len() == 1);
    assert_eq!(admitted.template_lang(), crate::embed::Lang::Ts);
}

#[test]
fn later_scripts_styles_custom_and_duplicates_preserve_the_whole_capture() {
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
        let source = vize_l0::cstr!("<template>{{{{ value | upper }}}}</template>{suffix}");
        let capture = Vue.split(&arena, &source);
        let owner = Vue.observe_vue2_descriptor(&arena, &source, options());
        assert!(owner.selected().is_err() && owner.component().is_none());
        assert!(
            owner
                .issues()
                .iter()
                .any(|i| i.code == expected && i.container_index == Some(1)),
            "{source}: {:?}",
            owner.issues()
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
    }
}

#[test]
fn original_template_attribute_and_splitter_refusals_never_parse_a_component() {
    let arena = Allocator::default();
    for (source, expected) in [
        ("plain outside source", Code::MissingComponentBlock),
        ("<template/>", Code::UnsupportedBoundary),
        ("<Template>x</Template>", Code::UnsupportedBlockSpelling),
        ("<template>x", Code::UnsupportedBoundary),
        ("<template>{{ value</template>", Code::UnsupportedBoundary),
        ("<template src='x.html'></template>", Code::ExternalSource),
        ("<template lang=pug>x</template>", Code::UnsupportedLanguage),
        (
            "<template lang='h&#116;ml'>x</template>",
            Code::EncodedLanguage,
        ),
        (
            "<template lang=html lang=html>x</template>",
            Code::DuplicateAttribute,
        ),
        (
            "<template functional>x</template>",
            Code::UnsupportedAttribute,
        ),
        (
            "<template lang='html'id=x>x</template>",
            Code::AmbiguousAttribute,
        ),
    ] {
        let measured = hooks::Armed::new(hooks::Fault::None);
        let owner = Vue.observe_vue2_descriptor(&arena, source, options());
        assert!(owner.selected().is_err() && owner.component().is_none());
        assert!(
            owner.issues().iter().any(|i| i.code == expected),
            "{source}: {:?}",
            owner.issues()
        );
        assert_eq!(
            (measured.counts().splitters, measured.counts().components),
            (1, 0)
        );
        drop(owner);
        assert_eq!(measured.counts().dropped, 1);
    }
}

#[test]
fn scanner_complete_holes_comments_and_recovery_keep_real_component_refusals() {
    let arena = Allocator::default();
    for (body, expected) in [
        ("{{ x + }}", TextRefusal::NativeHole(EmbedHole::Syntax)),
        (
            "{{ x | wrap(x +) }}",
            TextRefusal::NativeHole(EmbedHole::Syntax),
        ),
        (
            "{{ x // pipe|\n | upper }}",
            TextRefusal::Boundary(TextBoundaryKind::CommentSyntax),
        ),
        (
            "{{ x /* pipe| */ | upper }}",
            TextRefusal::Boundary(TextBoundaryKind::CommentSyntax),
        ),
        (
            "&#123;&#123; x &#125;&#125;{{ next }}",
            TextRefusal::Boundary(TextBoundaryKind::EncodedDelimiter),
        ),
    ] {
        let source = vize_l0::cstr!("<template>{body}</template>");
        let owner = Vue.observe_vue2_descriptor(&arena, &source, options());
        let component = owner.selected().unwrap().component();
        assert_eq!(check_fidelity(component.tree()), Ok(()));
        let child = component
            .children()
            .find(|child| matches!(child.surface(), SurfaceChild::Interpolation(_)))
            .unwrap();
        assert_eq!(component.text_for(child).unwrap_err(), expected);
        let binding = component.bindings().first().unwrap();
        assert!(core::ptr::eq(
            binding.source().authored_root(),
            source.as_str()
        ));
        if expected == TextRefusal::NativeHole(EmbedHole::Syntax) {
            let chain = binding.chain().unwrap();
            assert!(
                chain.base().diagnostics().count() > 0
                    || chain
                        .filters()
                        .iter()
                        .flat_map(|filter| filter.arguments())
                        .any(|arg| arg.diagnostics().count() > 0)
            );
        } else if expected == TextRefusal::Boundary(TextBoundaryKind::CommentSyntax) {
            assert!(binding.chain().is_none());
        }
    }
    let owner = Vue.observe_vue2_descriptor(
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
    let owner = Vue.observe_vue2_descriptor(
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
