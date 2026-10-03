//! Complete output, source custody and refusals for typed dynamic heads.

use vize_glyph::native_doc::{LineEnding, PrintOptions, TemplateRefusal, print, template_document};
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
fn every_dynamic_prefix_has_complete_flat_and_broken_output() {
    let source = "<Card :[key].camel = 'value' .[prop] = \"value as T\" @[event].once = 'act()' #[slot] = '{ item }' />";
    assert_eq!(
        format(
            source,
            PrintOptions {
                width: 120,
                ..PrintOptions::default()
            }
        ),
        "<Card :[key].camel='value' .[prop]=\"value as T\" @[event].once='act()' #[slot]='{ item }' />"
    );
    assert_eq!(
        format(
            source,
            PrintOptions {
                width: 20,
                ..PrintOptions::default()
            }
        ),
        "<Card\n  :[key].camel='value'\n  .[prop]=\"value as T\"\n  @[event].once='act()'\n  #[slot]='{ item }'\n/>"
    );
    let full = "<Card v-bind:[key].prop = 'value' v-on:[event].once = 'act()' v-slot:[slot] = '{ item }' v-custom:[other] />";
    assert_eq!(
        format(
            full,
            PrintOptions {
                width: 160,
                ..PrintOptions::default()
            }
        ),
        "<Card v-bind:[key].prop='value' v-on:[event].once='act()' v-slot:[slot]='{ item }' v-custom:[other] />"
    );
}

#[test]
fn nested_brackets_strings_and_templates_preserve_opaque_argument_bytes() {
    let source = "<Panel :[keys[index]].camel = 'value' @[events[\"x]y\"]].once = \"act()\" v-bind:[`x${key[0]}`] = 'value' #[slots[(index)]] = '{ item }' />";
    assert_eq!(
        format(
            source,
            PrintOptions {
                width: 180,
                ..PrintOptions::default()
            }
        ),
        "<Panel :[keys[index]].camel='value' @[events[\"x]y\"]].once=\"act()\" v-bind:[`x${key[0]}`]='value' #[slots[(index)]]='{ item }' />"
    );
    let source = r#"<Panel :[keys["a\"b]"]] = 'value' :[`x${`y${key[0]}`}`] = 'other' />"#;
    assert_eq!(
        format(source, PrintOptions::default()),
        r#"<Panel :[keys["a\"b]"]]='value' :[`x${`y${key[0]}`}`]='other' />"#
    );
}

#[test]
fn unicode_heads_entities_comments_and_multiline_values_stay_original() {
    let source = "<!--é-->\n<Panel v-カスタム:[日本[鍵]]..camel = 'one\r\n 二 &amp;' @[更新].once = \"&#x26;&amp;\" />";
    assert_eq!(
        format(source, PrintOptions::default()),
        "<!--é-->\n<Panel\n  v-カスタム:[日本[鍵]]..camel='one\r\n 二 &amp;'\n  @[更新].once=\"&#x26;&amp;\"\n/>"
    );
}

#[test]
fn generated_crlf_does_not_rewrite_authored_dynamic_argument_or_content() {
    let source = "<p :[key]='one\n two' @[event]='act()'>\ncontent\n</p>";
    assert_eq!(
        format(
            source,
            PrintOptions {
                width: 10,
                line_ending: LineEnding::CrLf,
                ..PrintOptions::default()
            }
        ),
        "<p\r\n  :[key]='one\n two'\r\n  @[event]='act()'\r\n>\ncontent\n</p>"
    );
}

#[test]
fn mixed_dynamic_static_and_plain_heads_keep_order_and_are_fixed_points() {
    for source in [
        "<Card id=x :[key].camel = 'value' v-bind:static='two' @[event].once='act()' disabled />",
        "<div><Panel v-slot:[slots[key]]='{ item }' #header='other' .[prop]='value' /></div>",
        "<!-- keep -->\n<p v-custom:[key] :[other]> text  &amp; </p>",
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
fn incomplete_or_noncontiguous_dynamic_heads_keep_original_observations() {
    for source in [
        "<p :[]='value'/>",
        "<p :[key='value'/>",
        "<p :[key]suffix='value'/>",
        "<p :[one][two]='value'/>",
        "<p :before[key]='value'/>",
        "<p :[key[\"unfinished]]='value'/>",
        "<p :[key>`tail`]='value'/>",
        "<p v-:[key]='value'/>",
        "<p v-bind:[key].camel='value></p>",
        "<p v-bind='object'/>",
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
fn existing_delimiter_capacity_is_inherited_and_unsupported_evidence_is_retained() {
    use vize_l1::dialect::vue3::directive::MAX_DIRECTIVE_DELIMITER_RUNS;
    use vize_l1::markup::DirectiveNameError;

    for count in [
        MAX_DIRECTIVE_DELIMITER_RUNS - 1,
        MAX_DIRECTIVE_DELIMITER_RUNS,
    ] {
        let opens: std::string::String = (0..count)
            .map(|index| if index % 2 == 0 { '(' } else { '[' })
            .collect();
        let closes: std::string::String = opens
            .chars()
            .rev()
            .map(|delimiter| if delimiter == '(' { ')' } else { ']' })
            .collect();
        let source = std::format!("<p :[{opens}key{closes}] = 'value' />");
        let allocator = Allocator::default();
        let parsed = parse_component(&allocator, &source).unwrap();
        if count < MAX_DIRECTIVE_DELIMITER_RUNS {
            let document = template_document(&parsed, &allocator).unwrap();
            assert_eq!(
                print(
                    document.document(),
                    &PrintOptions {
                        width: 512,
                        ..PrintOptions::default()
                    }
                ),
                std::format!("<p :[{opens}key{closes}]='value' />")
            );
            assert!(parsed.unsupported.is_empty());
        } else {
            assert_eq!(parsed.unsupported.len(), 1);
            let admission = parsed.unsupported[0];
            assert_eq!(admission.error, DirectiveNameError::NestingLimit);
            assert!(template_document(&parsed, &allocator).is_err());
            assert_eq!(parsed.unsupported[0], admission);
        }
        assert_eq!(parsed.tree.source, source);
        assert_eq!(check_fidelity(&parsed.tree), Ok(()));
    }
}

#[test]
fn foreign_equal_byte_dynamic_head_cannot_establish_source_custody() {
    let allocator = Allocator::default();
    let source = String::from("<p :[key].camel='original'/>");
    let foreign = String::from(":[key].camel");
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
