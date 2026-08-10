use vize_glyph::{AttributeSortOrder, FormatOptions, format_sfc, format_template};

fn assert_template_fixed_point(source: &str, expected: &str, options: &FormatOptions) {
    let first = format_template(source, options).unwrap();
    let second = format_template(&first, options).unwrap();

    assert_eq!(first.as_str(), expected);
    assert_eq!(first, second, "fmt; fmt must preserve the spread boundary");
}

#[test]
fn shorthand_object_bind_is_an_attribute_sorting_barrier() {
    let options = FormatOptions {
        print_width: 240,
        ..FormatOptions::default()
    };

    assert_template_fixed_point(
        r#"<a title="link" class="router-link" :="attrs" target="_blank" rel="noopener" :href="href"></a>"#,
        r#"<a class="router-link" title="link" :="attrs" rel="noopener" target="_blank" :href="href"></a>"#,
        &options,
    );
}

#[test]
fn shorthand_object_on_is_an_event_sorting_barrier() {
    let options = FormatOptions {
        print_width: 240,
        ..FormatOptions::default()
    };

    assert_template_fixed_point(
        r#"<button @keyup="up" @click="click" @="listeners" @mouseup="up" @mousedown="down"></button>"#,
        r#"<button @click="click" @keyup="up" @="listeners" @mousedown="down" @mouseup="up"></button>"#,
        &options,
    );
}

#[test]
fn every_longhand_and_shorthand_object_spread_pins_its_segment() {
    let options = FormatOptions {
        print_width: 240,
        ..FormatOptions::default()
    };
    let source = r#"<Comp z="z" :="first" b="b" a="a" v-bind="second" d="d" c="c" @="listeners" @keyup="up" @click="click" />"#;
    let expected = r#"<Comp z="z" :="first" a="a" b="b" v-bind="second" c="c" d="d" @="listeners" @click="click" @keyup="up" />"#;

    assert_template_fixed_point(source, expected, &options);
}

#[test]
fn sfc_shorthand_spreads_preserve_order_with_as_written_sorting() {
    let options = FormatOptions {
        attribute_sort_order: AttributeSortOrder::AsWritten,
        ..FormatOptions::default()
    };
    let source = r#"<template>
  <slot :name="column.key" :="{ column, record }" data-after="after" @="listeners" @click="click" />
</template>
"#;
    let first = format_sfc(source, &options).unwrap();
    let second = format_sfc(&first.code, &options).unwrap();

    assert!(
        first.code.find(":name").unwrap() < first.code.find(":=").unwrap(),
        "the named binding must not cross the object-bind shorthand"
    );
    assert!(
        first.code.find(":=").unwrap() < first.code.find("data-after").unwrap(),
        "the post-spread attribute must stay after the object bind"
    );
    assert!(
        first.code.find("@=").unwrap() < first.code.find("@click").unwrap(),
        "the event binding must not cross the object-on shorthand"
    );
    assert_eq!(first.code, second.code, "fmt; fmt must be a no-op");
}
