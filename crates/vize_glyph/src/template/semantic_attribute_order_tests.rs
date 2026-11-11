//! Semantic attribute-order coverage kept separate from the broad template
//! formatter suite so that suite remains within the source-length budget.

use super::format_template_content;
use crate::{AttributeSortOrder, FormatOptions};

#[test]
fn native_button_type_precedes_ordinary_attributes() {
    let source =
        r#"<button title="Save" name="save" disabled class="primary" type="button">Save</button>"#;
    let options = FormatOptions::default();
    let result = format_template_content(source, &options).unwrap();

    assert!(result.find("type=").unwrap() < result.find("class=").unwrap());
    assert!(result.find("type=").unwrap() < result.find("disabled").unwrap());
    assert!(result.find("type=").unwrap() < result.find("name=").unwrap());

    let converged = format_template_content(&result, &options).unwrap();
    assert_eq!(converged, result);
}

#[test]
fn native_button_semantics_do_not_reorder_component_props() {
    let source = r#"<Button type="button" class="primary">Save</Button>"#;
    let result = format_template_content(source, &FormatOptions::default()).unwrap();

    assert!(result.find("class=").unwrap() < result.find("type=").unwrap());
}

#[test]
fn native_button_semantics_sort_static_bind_arguments_with_their_neighbors() {
    let source = r#"<button class="primary" :type="kind" disabled>Save</button>"#;
    let result = format_template_content(source, &FormatOptions::default()).unwrap();

    assert_eq!(
        result,
        r#"<button class="primary" disabled :type="kind">Save</button>"#
    );
    assert_eq!(
        format_template_content(&result, &FormatOptions::default()).unwrap(),
        result
    );
}

#[test]
fn as_written_keeps_native_button_type_position() {
    let source = r#"<button class="primary" disabled type="button">Save</button>"#;
    let options = FormatOptions {
        attribute_sort_order: AttributeSortOrder::AsWritten,
        ..FormatOptions::default()
    };
    let result = format_template_content(source, &options).unwrap();

    assert!(result.find("class=").unwrap() < result.find("disabled").unwrap());
    assert!(result.find("disabled").unwrap() < result.find("type=").unwrap());
}
