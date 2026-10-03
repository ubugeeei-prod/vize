//! Complete output and source custody for native static binding layout.

use vize_glyph::native_doc::{
    PrintOptions, TemplateRefusal, UnsupportedSyntax, print, template_document,
};
use vize_l0::{Allocator, String};
use vize_l1::{SurfaceChild, check_fidelity, dialect::vue3::surface::parse_component};

fn format(source: &str, width: usize) -> String {
    let allocator = Allocator::default();
    let parsed = parse_component(&allocator, source).unwrap();
    let document = template_document(&parsed, &allocator).unwrap();
    assert!(core::ptr::eq(document.original(), &parsed));
    assert_eq!(check_fidelity(&parsed.tree), Ok(()));
    print(
        document.document(),
        &PrintOptions {
            width,
            ..PrintOptions::default()
        },
    )
}

#[test]
fn typed_static_bind_and_prop_heads_have_complete_flat_and_broken_output() {
    let source = "<Card :name = 'user?.name' .value = \"a&amp;b\" />";
    assert_eq!(
        format(source, 80),
        "<Card :name='user?.name' .value=\"a&amp;b\" />"
    );
    assert_eq!(
        format(source, 20),
        "<Card\n  :name='user?.name'\n  .value=\"a&amp;b\"\n/>"
    );
}

#[test]
fn unicode_arguments_modifiers_and_authored_multiline_values_stay_verbatim() {
    let source = "<!--é-->\n<Panel :日本.camel = 'one\r\n 二 &amp;' id = x />";
    assert_eq!(
        format(source, 80),
        "<!--é-->\n<Panel\n  :日本.camel='one\r\n 二 &amp;'\n  id=x\n/>"
    );
}

#[test]
fn opaque_js_ts_and_entity_values_are_never_decoded_or_reparsed() {
    let source = r#"<Widget :data = '({ key: `x${ n }` }) as const' .innerHTML = "&#x26;&amp;" />"#;
    assert_eq!(
        format(source, 120),
        r#"<Widget :data='({ key: `x${ n }` }) as const' .innerHTML="&#x26;&amp;" />"#
    );
}

#[test]
fn binding_layout_preserves_attribute_order_and_is_a_fixed_point() {
    for source in [
        "<Card id=x :é = 'value' .data-prop=\"original\" disabled />",
        "<div><Panel :value.camel='a' title=b /></div>",
        "<!-- keep -->\n<p :value> text  &amp; </p>",
    ] {
        for width in [0, 7, 20, 80] {
            let output = format(source, width);
            assert_eq!(format(&output, width), output);
        }
    }
}

#[test]
fn incomplete_and_missing_arguments_keep_original_observations_on_refusal() {
    for source in [
        "<p :[]='value'/>",
        "<p .[]='value'/>",
        "<p @[]='act()'/>",
        "<p #[]='item'/>",
        "<p v-bind:[]='value'/>",
        "<p v-on:[]='act()'/>",
        "<p v-slot:[]='item'/>",
        "<p v-/>",
        "<p v-:label='value'/>",
        "<p :='value'/>",
        "<p .='value'/>",
        "<p @='act()'/>",
        "<p #='item'/>",
    ] {
        let allocator = Allocator::default();
        let parsed = parse_component(&allocator, source).unwrap();
        let error_count = parsed.errors.len();
        assert!(matches!(
            template_document(&parsed, &allocator),
            Err(TemplateRefusal::Unsupported {
                syntax: UnsupportedSyntax::Directive,
                ..
            } | TemplateRefusal::Recovered { .. })
        ));
        assert_eq!(parsed.errors.len(), error_count);
        assert_eq!(check_fidelity(&parsed.tree), Ok(()));
        assert_eq!(parsed.tree.source, source);
    }
}

#[test]
fn a_foreign_equal_byte_directive_head_does_not_establish_source_custody() {
    let allocator = Allocator::default();
    let source = String::from("<p :value='original'/>");
    let foreign = String::from(":value");
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

#[test]
fn full_static_and_event_slot_heads_have_complete_flat_and_broken_output() {
    let source = "<Card v-bind:label = 'name' v-on:click.stop = \"act()\" @focus = 'focus()' #header = 'slot' />";
    assert_eq!(
        format(source, 120),
        "<Card v-bind:label='name' v-on:click.stop=\"act()\" @focus='focus()' #header='slot' />"
    );
    assert_eq!(
        format(source, 20),
        "<Card\n  v-bind:label='name'\n  v-on:click.stop=\"act()\"\n  @focus='focus()'\n  #header='slot'\n/>"
    );
}

#[test]
fn full_unicode_name_argument_and_opaque_values_keep_original_spelling() {
    let source = "<!--é-->\n<Panel v-カスタム:日本..camel = 'one\r\n 二 &amp;' @更新.once = \"value as T\" v-slot:見出し = '{ item }' />";
    assert_eq!(
        format(source, 80),
        "<!--é-->\n<Panel\n  v-カスタム:日本..camel='one\r\n 二 &amp;'\n  @更新.once=\"value as T\"\n  v-slot:見出し='{ item }'\n/>"
    );
}

#[test]
fn mixed_static_families_preserve_order_and_are_fixed_points() {
    for source in [
        "<Card id=x v-bind:é = 'value' @click.stop=\"act()\" #header='item' disabled />",
        "<div><Panel v-slot:header='a' :value.camel='b' .data-prop='c' /></div>",
        "<!-- keep -->\n<p v-custom:arg..modifier @click #default> text  &amp; </p>",
    ] {
        for width in [0, 7, 20, 80, 120] {
            let output = format(source, width);
            assert_eq!(format(&output, width), output);
        }
    }
}

#[test]
fn incomplete_dynamic_and_noncontiguous_full_heads_keep_original_observations() {
    for source in [
        "<p v-bind:='value'/>",
        "<p v-bind:[key='value'/>",
        "<p v-bind:[key]suffix='value'/>",
        "<p @click[tail]='act()'/>",
        "<p v-on:click='act()' broken='value></p>",
    ] {
        let allocator = Allocator::default();
        let parsed = parse_component(&allocator, source).unwrap();
        let error_count = parsed.errors.len();
        assert!(template_document(&parsed, &allocator).is_err());
        assert_eq!(parsed.errors.len(), error_count);
        assert_eq!(parsed.tree.source, source);
        assert_eq!(check_fidelity(&parsed.tree), Ok(()));
    }
}
