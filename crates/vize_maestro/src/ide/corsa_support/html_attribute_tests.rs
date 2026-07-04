use super::html_attribute::{html_attribute_virtual_document, native_dom_attribute_info};

#[test]
fn native_dom_attribute_info_maps_html_attributes_to_dom_properties() {
    let disabled = native_dom_attribute_info("button", "disabled").expect("disabled attr");
    assert_eq!(disabled.property_name.as_str(), "disabled");
    assert_eq!(disabled.category, "HTML attribute");
    assert!(disabled.is_boolean);
    assert_eq!(
        disabled.type_expression.as_str(),
        "HTMLElementTagNameMap[\"button\"][\"disabled\"]"
    );

    let class_attr = native_dom_attribute_info("div", "class").expect("class attr");
    assert_eq!(class_attr.property_name.as_str(), "className");
    let bound = native_dom_attribute_info("button", "v-bind:disabled").expect("v-bind attr");
    assert_eq!(bound.property_name.as_str(), "disabled");
    let aria = native_dom_attribute_info("button", "aria-label").expect("aria-label attr");
    assert_eq!(aria.property_name.as_str(), "ariaLabel");
    let data = native_dom_attribute_info("div", "data-test-id").expect("data attribute");
    assert_eq!(data.property_name.as_str(), "dataset");
}

#[test]
fn native_dom_attribute_info_preserves_svg_dom_property_names() {
    let view_box = native_dom_attribute_info("svg", "viewBox").expect("viewBox attr");
    assert_eq!(view_box.category, "SVG attribute");
    assert_eq!(view_box.property_name.as_str(), "viewBox");
    assert_eq!(
        view_box.type_expression.as_str(),
        "SVGElementTagNameMap[\"svg\"][\"viewBox\"]"
    );
    assert!(
        view_box
            .documentation_url
            .ends_with("/Web/SVG/Reference/Attribute/viewBox")
    );

    let lowercase_view_box =
        native_dom_attribute_info("svg", "viewbox").expect("lowercase viewbox attr");
    assert_eq!(lowercase_view_box.property_name.as_str(), "viewBox");

    let circle_center = native_dom_attribute_info("circle", "cx").expect("circle cx attr");
    assert_eq!(circle_center.property_name.as_str(), "cx");
    assert_eq!(
        circle_center.type_expression.as_str(),
        "SVGElementTagNameMap[\"circle\"][\"cx\"]"
    );

    let text_path_href = native_dom_attribute_info("textPath", "href").expect("textPath href");
    assert_eq!(text_path_href.property_name.as_str(), "href");
    assert_eq!(
        text_path_href.type_expression.as_str(),
        "SVGElementTagNameMap[\"textPath\"][\"href\"]"
    );
}

#[test]
fn native_dom_attribute_info_rejects_unknown_and_component_attributes() {
    assert!(native_dom_attribute_info("button", "not-real").is_none());
    assert!(native_dom_attribute_info("div", "href").is_none());
    assert!(native_dom_attribute_info("svg", "not-real").is_none());
    assert!(native_dom_attribute_info("textPath", "x").is_none());
    assert!(native_dom_attribute_info("animate", "href").is_none());
    assert!(native_dom_attribute_info("my-element", "disabled").is_none());
    assert!(native_dom_attribute_info("button", "@click").is_none());
}

#[test]
fn html_attribute_virtual_document_queries_dom_property() {
    let doc = html_attribute_virtual_document("button", "disabled").expect("disabled attr doc");
    assert!(doc.content.contains("__vizeDomElement.disabled"));
    let property_start = doc.content.rfind("disabled").expect("disabled property");
    assert!(doc.hover_offset >= property_start);
    assert!(doc.hover_offset < property_start + "disabled".len());
    assert_eq!(doc.hover_offset, doc.definition_offset);
}
