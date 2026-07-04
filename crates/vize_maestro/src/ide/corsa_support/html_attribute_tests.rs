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
    assert!(
        disabled
            .documentation_url
            .ends_with("/Web/HTML/Reference/Attributes/disabled")
    );

    let class_attr = native_dom_attribute_info("div", "class").expect("class attr");
    assert_eq!(class_attr.property_name.as_str(), "className");
    assert!(
        class_attr
            .documentation_url
            .ends_with("/Web/HTML/Reference/Global_attributes/class")
    );
    let access_key = native_dom_attribute_info("div", "accesskey").expect("accesskey attr");
    assert_eq!(access_key.property_name.as_str(), "accessKey");
    let bound = native_dom_attribute_info("button", "v-bind:disabled").expect("v-bind attr");
    assert_eq!(bound.property_name.as_str(), "disabled");
    let aria = native_dom_attribute_info("button", "aria-label").expect("aria-label attr");
    assert_eq!(aria.property_name.as_str(), "ariaLabel");
    let data = native_dom_attribute_info("div", "data-test-id").expect("data attribute");
    assert_eq!(data.property_name.as_str(), "dataset");
    let fetch_priority =
        native_dom_attribute_info("img", "fetchpriority").expect("fetchpriority attr");
    assert_eq!(fetch_priority.property_name.as_str(), "fetchPriority");
    assert_eq!(
        fetch_priority.type_expression.as_str(),
        "HTMLElementTagNameMap[\"img\"][\"fetchPriority\"]"
    );
    let dir_name = native_dom_attribute_info("textarea", "dirname").expect("dirname attr");
    assert_eq!(dir_name.property_name.as_str(), "dirName");
    assert_eq!(
        dir_name.type_expression.as_str(),
        "HTMLElementTagNameMap[\"textarea\"][\"dirName\"]"
    );

    let anchor_type = native_dom_attribute_info("a", "type").expect("anchor type attr");
    assert_eq!(anchor_type.property_name.as_str(), "type");
    assert_eq!(
        anchor_type.type_expression.as_str(),
        "HTMLElementTagNameMap[\"a\"][\"type\"]"
    );
    let area_alt = native_dom_attribute_info("area", "alt").expect("area alt attr");
    assert_eq!(area_alt.property_name.as_str(), "alt");
    assert_eq!(
        area_alt.type_expression.as_str(),
        "HTMLElementTagNameMap[\"area\"][\"alt\"]"
    );
    let area_coords = native_dom_attribute_info("area", "coords").expect("area coords attr");
    assert_eq!(area_coords.property_name.as_str(), "coords");
    let area_shape = native_dom_attribute_info("area", "shape").expect("area shape attr");
    assert_eq!(area_shape.property_name.as_str(), "shape");

    let link_image_sizes =
        native_dom_attribute_info("link", "imagesizes").expect("link imagesizes attr");
    assert_eq!(link_image_sizes.property_name.as_str(), "imageSizes");
    assert_eq!(
        link_image_sizes.type_expression.as_str(),
        "HTMLElementTagNameMap[\"link\"][\"imageSizes\"]"
    );
    let link_image_srcset =
        native_dom_attribute_info("link", "imagesrcset").expect("link imagesrcset attr");
    assert_eq!(link_image_srcset.property_name.as_str(), "imageSrcset");
    assert_eq!(
        link_image_srcset.type_expression.as_str(),
        "HTMLElementTagNameMap[\"link\"][\"imageSrcset\"]"
    );
    let script_for = native_dom_attribute_info("script", "for").expect("script for attr");
    assert_eq!(script_for.property_name.as_str(), "htmlFor");
    assert_eq!(
        script_for.type_expression.as_str(),
        "HTMLElementTagNameMap[\"script\"][\"htmlFor\"]"
    );
    let style_disabled =
        native_dom_attribute_info("style", "disabled").expect("style disabled attr");
    assert_eq!(style_disabled.property_name.as_str(), "disabled");
    assert!(style_disabled.is_boolean);
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

    let svg_class = native_dom_attribute_info("svg", "class").expect("svg class attr");
    assert_eq!(svg_class.property_name.as_str(), "className");
    assert_eq!(
        svg_class.type_expression.as_str(),
        "SVGElementTagNameMap[\"svg\"][\"className\"]"
    );
    let math_tabindex = native_dom_attribute_info("math", "tabindex").expect("math tabindex attr");
    assert_eq!(math_tabindex.property_name.as_str(), "tabIndex");
    assert_eq!(
        math_tabindex.type_expression.as_str(),
        "MathMLElementTagNameMap[\"math\"][\"tabIndex\"]"
    );
    assert!(
        math_tabindex
            .documentation_url
            .ends_with("/Web/HTML/Reference/Global_attributes/tabindex")
    );

    let menclose_tabindex =
        native_dom_attribute_info("menclose", "tabindex").expect("menclose tabindex attr");
    assert_eq!(menclose_tabindex.property_name.as_str(), "tabIndex");
    assert_eq!(
        menclose_tabindex.type_expression.as_str(),
        "MathMLElement[\"tabIndex\"]"
    );
}

#[test]
fn native_dom_attribute_info_rejects_unknown_and_component_attributes() {
    assert!(native_dom_attribute_info("button", "not-real").is_none());
    assert!(native_dom_attribute_info("div", "href").is_none());
    assert!(native_dom_attribute_info("div", "is").is_none());
    assert!(native_dom_attribute_info("div", "itemprop").is_none());
    assert!(native_dom_attribute_info("area", "hreflang").is_none());
    assert!(native_dom_attribute_info("meta", "charset").is_none());
    assert!(native_dom_attribute_info("param", "name").is_none());
    assert!(native_dom_attribute_info("param", "value").is_none());
    assert!(native_dom_attribute_info("svg", "not-real").is_none());
    assert!(native_dom_attribute_info("svg", "accesskey").is_none());
    assert!(native_dom_attribute_info("math", "contenteditable").is_none());
    assert!(native_dom_attribute_info("textPath", "x").is_none());
    assert!(native_dom_attribute_info("animate", "href").is_none());
    assert!(native_dom_attribute_info("mesh", "id").is_none());
    assert!(native_dom_attribute_info("unknown", "id").is_none());
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
