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
fn native_button_semantics_preserve_dynamic_barriers() {
    // Only an object bind or a dynamic argument is a barrier (#12826); a
    // static `:type` bind sorts with its neighbors.
    let source = r#"<button class="primary" :[kind]="value" disabled type="button">Save</button>"#;
    let result = format_template_content(source, &FormatOptions::default()).unwrap();

    assert_eq!(
        result.as_str(),
        r#"<button class="primary" :[kind]="value" type="button" disabled>Save</button>"#
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
