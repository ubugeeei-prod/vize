use super::format_template_content;
use crate::options::FormatOptions;

fn format(source: &str) -> vize_l0::String {
    format_template_content(source, &FormatOptions::default())
        .unwrap_or_else(|error| panic!("template formats: {error}"))
}

#[test]
fn directives_preserve_authored_order() {
    let formatted = format(r#"<button @click="onClick" v-if="props.show" id="main">open</button>"#);
    assert_eq!(
        formatted.as_str(),
        r#"<button @click="onClick" v-if="props.show" id="main">open</button>"#
    );
    let formatted = format(r#"<p class="note" v-if="props.show" :title="props.title">text</p>"#);
    assert_eq!(
        formatted.as_str(),
        r#"<p class="note" v-if="props.show" :title="props.title">text</p>"#
    );
}

#[test]
fn static_event_directives_preserve_authored_order() {
    let formatted =
        format(r#"<button @show="show" tabindex="0" @hide="hide" class="control">open</button>"#);
    assert_eq!(
        formatted.as_str(),
        r#"<button @show="show" tabindex="0" @hide="hide" class="control">open</button>"#
    );
}

#[test]
fn v_bind_object_and_dynamic_argument_stay_fixed() {
    let formatted = format(r#"<div class="b" id="a" v-bind="obj" title="t"></div>"#);
    assert_eq!(
        formatted.as_str(),
        r#"<div id="a" class="b" v-bind="obj" title="t"></div>"#
    );
    let formatted = format(r#"<div class="b" :[name]="value" id="a"></div>"#);
    assert_eq!(
        formatted.as_str(),
        r#"<div class="b" :[name]="value" id="a"></div>"#
    );
}
