extern crate std;

use super::{NativeSelectedScopedSfcObservation, lower_selected_scoped_sfc_native};
use crate::native_file::{
    NativeSelectedSfcIssueKind as LowerKind, lower_selected_setup_sfc_native,
    lower_selected_sfc_native,
};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{
    SurfaceParseOptions,
    container::{
        Vue,
        vue::{DescriptorObservation, DescriptorOptions},
    },
    css::StyleSyntax,
    markup::NativeTemplateComponent,
};
use vize_l2::lang::js::{NativeScopedTemplateIssueKind as Kind, NativeTemplateOwner};

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}
fn syntax<'a>(original: &DescriptorObservation<'a>) -> StyleSyntax<'a> {
    StyleSyntax::observe(original.admitted().unwrap().styles().next().unwrap())
}

#[test]
fn whole_original_style_parser_and_completed_file_survive_movement() {
    for source in [
        "<!--whole--><template><p></p></template><style scoped>.a:empty{background:blue}</style>",
        "<style lang=css scoped>/* 雪🌸 */\r\n.雪:empty \t{ background:blue; }</style><template><p></p><p>visible</p></template>",
        "<template><div><p></p><br/></div></template><style scoped>.a:empty{background:blue}</style>",
    ] {
        let arena = Allocator::default();
        let original = lower_selected_scoped_sfc_native(&arena, source, options());
        assert!(original.original().issues().is_empty());
        let parser = original.style_syntax().unwrap();
        let token_ptr = parser.rule().unwrap().prelude().as_ptr();
        let body_ptr = original
            .original()
            .template()
            .unwrap()
            .selected()
            .component()
            .carrier()
            .tree
            .children
            .as_ptr();
        let moved = core::hint::black_box(original);
        let admitted = moved.admitted().unwrap();
        assert!(core::ptr::eq(admitted.observation(), &moved));
        assert!(moved.original().admitted().is_none());
        let view = admitted.into_scoped_template_view();
        assert!(core::ptr::eq(
            view.owner(),
            moved.original().template().unwrap()
        ));
        assert!(core::ptr::eq(view.file(), view.owner().file().unwrap()));
        assert!(view.file().is_complete());
        assert!(view.file().units().is_empty());
        assert!(core::ptr::eq(view.file().artifact().source(), source));
        assert!(core::ptr::eq(
            view.style_syntax(),
            moved.style_syntax().unwrap()
        ));
        assert_eq!(
            view.style_syntax().rule().unwrap().prelude().as_ptr(),
            token_ptr
        );
        assert_eq!(
            view.owner()
                .selected()
                .component()
                .carrier()
                .tree
                .children
                .as_ptr(),
            body_ptr
        );
        let style = view.descriptor().styles().next().unwrap();
        assert_eq!(
            view.style_syntax().container_index(),
            style.container_index()
        );
        assert_eq!(view.style_syntax().source().span(), style.block().span());
        assert!(core::ptr::eq(
            view.style_syntax().source().source(),
            style.block().source()
        ));
        assert!(core::ptr::eq(
            view.style_syntax().source().root_source(),
            source
        ));
        let empty = view.empty_style();
        assert!(core::ptr::eq(
            empty.class_token(),
            &view.style_syntax().rule().unwrap().prelude()[1]
        ));
        assert_eq!(empty.pseudo_span().slice(source), ":empty");
    }
}

#[test]
fn public_join_checks_physical_buffer_and_block_not_descriptor_allocation_identity() {
    let arena = Allocator::default();
    let source = vize_l0::String::from(
        "<template><p/></template><style scoped>.a:empty{color:red}</style><!--whole-->",
    );
    let other = source.clone();
    let owner = lower_selected_scoped_sfc_native(&arena, &source, options());
    let template = owner.original().template().unwrap();
    let independently_observed = Vue.observe_descriptor(&arena, &source, options());
    let same_buffer_syntax = syntax(&independently_observed);
    let joined = template
        .scoped_view(&independently_observed, &same_buffer_syntax)
        .unwrap();
    assert!(core::ptr::eq(joined.file(), template.file().unwrap()));
    let foreign = Vue.observe_descriptor(&arena, &other, options());
    let foreign_syntax = syntax(&foreign);
    assert_eq!(
        template
            .scoped_view(&foreign, &foreign_syntax)
            .err()
            .unwrap()
            .kind,
        Kind::Source
    );
    assert_eq!(
        template
            .scoped_view(owner.original().descriptor(), &foreign_syntax)
            .err()
            .unwrap()
            .kind,
        Kind::Source
    );
    // A genuine style observation over the same allocation but a different
    // physical root extent cannot borrow this original completed File.
    let end = source.find("<!--whole-->").unwrap();
    let subroot = &source[..end];
    let wrong = Vue.observe_descriptor(&arena, subroot, options());
    let wrong_syntax = syntax(&wrong);
    assert_eq!(
        template
            .scoped_view(owner.original().descriptor(), &wrong_syntax)
            .err()
            .unwrap()
            .kind,
        Kind::Source
    );
    assert_eq!(subroot.as_ptr(), source.as_ptr());
    assert_ne!(subroot.len(), source.len());
    let multiple = Vue.observe_descriptor(
        &arena,
        "<template><p/></template><style scoped>.a:empty{}</style><style scoped>.b:empty{}</style>",
        options(),
    );
    let wrong_block = StyleSyntax::observe(multiple.admitted().unwrap().styles().nth(1).unwrap());
    assert_eq!(
        template
            .scoped_view(&multiple, &wrong_block)
            .err()
            .unwrap()
            .kind,
        Kind::StyleCount
    );
    assert_eq!(
        template
            .scoped_view(owner.original().descriptor(), &wrong_block)
            .err()
            .unwrap()
            .kind,
        Kind::Source
    );
}

#[test]
fn unsupported_css_retains_real_tokens_or_parser_error_without_partial_template() {
    for css in [
        ".a{color:red}",
        ".a:empty,.b:empty{color:red}",
        ".a:hover{color:red}",
        ".a:empty{color:v-bind(tone)}",
        ".a:empty{color:'v-bind(tone)'}",
        "@media screen{.a:empty{color:red}}",
        ".a:empty{color:red",
    ] {
        let arena = Allocator::default();
        let source = std::format!("<template><p/></template><style scoped>{css}</style>");
        let owner = lower_selected_scoped_sfc_native(&arena, &source, options());
        assert!(owner.admitted().is_none(), "{css}");
        assert!(owner.original().template().is_none(), "{css}");
        let syntax = owner.style_syntax().unwrap();
        let issue = syntax.empty_class().err().unwrap();
        assert_eq!(
            owner.original().issues()[0].kind,
            LowerKind::ScopedStyle(Kind::StyleSyntax(issue))
        );
        assert_eq!(syntax.source().source(), css);
        assert!(core::ptr::eq(
            syntax.source().root_source(),
            source.as_str()
        ));
        assert_eq!(owner.original().descriptor().container().blocks.len(), 2);
        assert!(
            syntax.rule().is_some() || syntax.parser_error().is_some() || syntax.issue().is_some()
        );
    }
}

#[test]
fn unsupported_envelope_profiles_keep_descriptor_without_fabricating_css() {
    for suffix in [
        "",
        "<style>.a:empty{}</style>",
        "<style scoped='yes'>.a:empty{}</style>",
        "<style scoped module>.a:empty{}</style>",
        "<style scoped lang=scss>.a:empty{}</style>",
        "<style scoped>.a:empty{}</style><style scoped>.b:empty{}</style>",
        "<style scoped src='outside.css'></style>",
        "<style scoped custom>.a:empty{}</style>",
        "<style scoped>.a:empty{}</style><script setup>const v=1;</script>",
        "<style scoped>.a:empty{}</style><script>export default {};</script>",
    ] {
        let arena = Allocator::default();
        let source = std::format!("<template><p/></template>{suffix}");
        let owner = lower_selected_scoped_sfc_native(&arena, &source, options());
        assert!(owner.admitted().is_none(), "{suffix}");
        assert!(owner.style_syntax().is_none(), "{suffix}");
        assert!(owner.original().template().is_none());
        assert!(!owner.original().issues().is_empty());
        assert!(core::ptr::eq(
            owner.original().descriptor().source(),
            source.as_str()
        ));
    }
}

#[test]
fn nondefault_public_profile_and_uncompleted_file_cannot_grant_scoped_authority() {
    let arena = Allocator::default();
    let source = "<template><p/></template><style scoped>.a:empty{color:red}</style>";
    let owner = lower_selected_scoped_sfc_native(&arena, source, options());
    let template = owner.original().template().unwrap();
    for changed in [
        DescriptorOptions {
            version: VueVersion::V2,
            ..options()
        },
        DescriptorOptions {
            dialect: VueDialect::PetiteVue,
            ..options()
        },
        DescriptorOptions {
            template: SurfaceParseOptions {
                experimental_in_tag_comments: true,
            },
            ..options()
        },
    ] {
        let original = Vue.observe_descriptor(&arena, source, changed);
        assert_eq!(
            template
                .scoped_view(&original, owner.style_syntax().unwrap())
                .err()
                .unwrap()
                .kind,
            Kind::Profile
        );
        let refused = lower_selected_scoped_sfc_native(&arena, source, changed);
        assert!(refused.admitted().is_none());
        assert!(refused.style_syntax().is_none());
        assert_eq!(refused.original().descriptor().options(), changed);
    }
    let original = owner.original().descriptor();
    let selected = NativeTemplateComponent::parse_in(&arena, original.admitted().unwrap())
        .unwrap()
        .unwrap();
    let unfinished = NativeTemplateOwner::new(selected).ok().unwrap().finish();
    assert!(matches!(
        unfinished
            .scoped_view(original, owner.style_syntax().unwrap())
            .err()
            .unwrap()
            .kind,
        Kind::Completion(_)
    ));
}

#[test]
fn old_scriptless_setup_and_authored_class_gates_remain_intact() {
    let arena = Allocator::default();
    let source = "<template><p/></template><style scoped>.a:empty{color:red}</style>";
    let old = lower_selected_sfc_native(&arena, source, options());
    assert_eq!(old.issues()[0].kind, LowerKind::Style);
    assert!(old.template().is_none());
    let setup_source = "<script setup>const value=1;</script><template><p/></template><style scoped>.a:empty{color:red}</style>";
    let setup = lower_selected_setup_sfc_native(&arena, setup_source, options());
    assert!(setup.admitted().is_none());
    assert_eq!(setup.original().issues()[0].kind, LowerKind::Style);
    for attr in ["class='a'", "style='color:red'"] {
        let source = std::format!(
            "<template><p {attr}/></template><style scoped>.a:empty{{color:red}}</style>"
        );
        let owner: NativeSelectedScopedSfcObservation<'_> =
            lower_selected_scoped_sfc_native(&arena, &source, options());
        assert!(owner.admitted().is_none());
        assert!(owner.style_syntax().is_some());
        assert!(owner.original().template().unwrap().view().is_err());
    }
}
