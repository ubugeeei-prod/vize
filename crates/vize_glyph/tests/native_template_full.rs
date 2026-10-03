//! Complete native layout and preservation for Full heads without arguments.

use vize_glyph::native_doc::{
    LineEnding, PrintOptions, TemplateRefusal, UnsupportedSyntax, print, template_document,
};
use vize_l0::{Allocator, String};
use vize_l1::{SurfaceChild, check_fidelity, dialect::vue3::surface::parse_component};

fn format(source: &str, options: PrintOptions) -> String {
    let allocator = Allocator::default();
    let parsed = parse_component(&allocator, source).unwrap();
    let document = template_document(&parsed, &allocator).unwrap();
    assert!(core::ptr::eq(document.original(), &parsed));
    assert_eq!(check_fidelity(&parsed.tree), Ok(()));
    print(document.document(), &options)
}

#[test]
fn full_heads_with_opaque_values_have_complete_flat_and_broken_output() {
    let source = "<Card v-if = 'ready && ok' v-for = \"({ title }, index) in rows\" v-show = 'visible' v-bind = 'props' v-on = 'handlers' v-custom..keep = 'opaque?.value as T' />";
    assert_eq!(
        format(
            source,
            PrintOptions {
                width: 240,
                ..PrintOptions::default()
            }
        ),
        "<Card v-if='ready && ok' v-for=\"({ title }, index) in rows\" v-show='visible' v-bind='props' v-on='handlers' v-custom..keep='opaque?.value as T' />"
    );
    assert_eq!(
        format(
            source,
            PrintOptions {
                width: 20,
                ..PrintOptions::default()
            }
        ),
        "<Card\n  v-if='ready && ok'\n  v-for=\"({ title }, index) in rows\"\n  v-show='visible'\n  v-bind='props'\n  v-on='handlers'\n  v-custom..keep='opaque?.value as T'\n/>"
    );
}

#[test]
fn complete_full_names_admit_optional_values_without_semantic_classification() {
    let source = "<p v-else v-on v-bind v-custom..keep />";
    assert_eq!(format(source, PrintOptions::default()), source);
}

#[test]
fn unicode_names_entities_comments_and_multiline_values_stay_authored() {
    let source = "<!--é-->\n<Panel v-カスタム..keep = 'one\r\n 二 &amp;' v-if = \"&#x26;&amp;\" />";
    assert_eq!(
        format(source, PrintOptions::default()),
        "<!--é-->\n<Panel\n  v-カスタム..keep='one\r\n 二 &amp;'\n  v-if=\"&#x26;&amp;\"\n/>"
    );
}

#[test]
fn original_native_pre_suppression_keeps_nested_interpolation_text() {
    for pre in ["v-pre", "v-pre.foo"] {
        let source = std::format!(
            "<div {pre} = '保留' id = raw>{{{{ user?.name }}}}<span title = '{{{{ raw }}}}'>{{{{ nested }}}}</span></div>"
        );
        assert_eq!(
            format(
                &source,
                PrintOptions {
                    width: 240,
                    ..PrintOptions::default()
                }
            ),
            std::format!(
                "<div {pre}='保留' id=raw>{{{{ user?.name }}}}<span title='{{{{ raw }}}}'>{{{{ nested }}}}</span></div>"
            )
        );
    }
    let source = "<div id=raw>{{ user?.name }}<span>{{ nested }}</span></div>";
    let allocator = Allocator::default();
    let parsed = parse_component(&allocator, source).unwrap();
    assert!(matches!(
        template_document(&parsed, &allocator),
        Err(TemplateRefusal::Unsupported {
            syntax: UnsupportedSyntax::Interpolation,
            ..
        })
    ));
    assert_eq!(parsed.tree.source, source);
    assert_eq!(check_fidelity(&parsed.tree), Ok(()));
}

#[test]
fn mixed_full_static_dynamic_and_plain_heads_keep_order_and_are_fixed_points() {
    for source in [
        "<Card id=x v-if = 'ready' :[key].camel = 'value' v-bind:static='two' v-on='handlers' disabled />",
        "<div><Panel v-for = 'item in rows' v-slot:[slots[key]]='{ item }' #header='other' .[prop]='value' /></div>",
        "<!-- keep -->\n<p v-custom v-show='ready' :[other]> text  &amp; </p>",
        "<p v-pre>{{ raw }}<span v-custom='opaque'>{{ nested }}</span></p>",
    ] {
        for width in [0, 7, 20, 80, 160] {
            let options = PrintOptions {
                width,
                ..PrintOptions::default()
            };
            let output = format(source, options);
            assert_eq!(format(&output, options), output);
        }
    }
}

#[test]
fn generated_crlf_keeps_opaque_full_values_and_source_content_newlines() {
    let source = "<p v-if='one\n two' v-bind='props'>\ncontent\n</p>";
    assert_eq!(
        format(
            source,
            PrintOptions {
                width: 10,
                line_ending: LineEnding::CrLf,
                ..PrintOptions::default()
            }
        ),
        "<p\r\n  v-if='one\n two'\r\n  v-bind='props'\r\n>\ncontent\n</p>"
    );
}

#[test]
fn malformed_or_recovered_heads_keep_original_parse_observations_on_refusal() {
    for source in [
        "<p v-/>",
        "<p v-.prop/>",
        "<p :='value'/>",
        "<p .='value'/>",
        "<p @='act()'/>",
        "<p #='item'/>",
        "<p v-bind:='props'/>",
        "<p v-on:.once='handlers'/>",
        "<p v-custom:[='value'/>",
        "<p v-bind:[]='value'/>",
        "<p v-bind:[one][two]='value'/>",
        "<p v-if='unterminated></p>",
        "<p v-show='ready'>",
        "<p>{{ raw }}</p>",
    ] {
        let allocator = Allocator::default();
        let parsed = parse_component(&allocator, source).unwrap();
        let errors = parsed
            .errors
            .iter()
            .map(|error| error.offset)
            .collect::<std::vec::Vec<_>>();
        assert!(template_document(&parsed, &allocator).is_err(), "{source}");
        assert_eq!(
            parsed
                .errors
                .iter()
                .map(|error| error.offset)
                .collect::<std::vec::Vec<_>>(),
            errors
        );
        assert_eq!(parsed.tree.source, source);
        assert_eq!(check_fidelity(&parsed.tree), Ok(()));
    }
}

#[test]
fn foreign_equal_byte_full_head_cannot_establish_source_custody() {
    let allocator = Allocator::default();
    let source = String::from("<p v-if..keep='original'/>");
    let foreign = String::from("v-if..keep");
    let mut parsed = parse_component(&allocator, &source).unwrap();
    let SurfaceChild::Element(element) = &mut parsed.tree.children[0] else {
        panic!("element")
    };
    element.open.attrs[0].name.text = &foreign;
    assert!(matches!(
        template_document(&parsed, &allocator),
        Err(TemplateRefusal::SourceMismatch { .. })
    ));
    assert_eq!(parsed.tree.source, source.as_str());
}
