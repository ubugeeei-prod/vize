//! SVG attribute names that are reflected as TypeScript DOM properties.

pub(super) fn canonical_svg_dom_attribute_name(
    tag_name: &str,
    attr_name: &str,
) -> Option<&'static str> {
    mapped_svg_dom_attribute_property(tag_name, attr_name)
}

pub(super) fn mapped_svg_dom_attribute_property(
    tag_name: &str,
    attr_name: &str,
) -> Option<&'static str> {
    let attr_name = attr_name.to_ascii_lowercase();
    match attr_name.as_str() {
        "viewbox" if svg_has_view_box(tag_name) => Some("viewBox"),
        "preserveaspectratio" if svg_has_preserve_aspect_ratio(tag_name) => {
            Some("preserveAspectRatio")
        }
        "pathlength" if svg_has_path_length(tag_name) => Some("pathLength"),
        "textlength" if svg_has_text_length(tag_name) => Some("textLength"),
        "lengthadjust" if svg_has_text_length(tag_name) => Some("lengthAdjust"),
        "x" if svg_has_x_y(tag_name) => Some("x"),
        "y" if svg_has_x_y(tag_name) => Some("y"),
        "width" if svg_has_width_height(tag_name) => Some("width"),
        "height" if svg_has_width_height(tag_name) => Some("height"),
        "cx" if svg_has_center(tag_name) => Some("cx"),
        "cy" if svg_has_center(tag_name) => Some("cy"),
        "r" if matches!(tag_name, "circle" | "radialGradient") => Some("r"),
        "rx" if matches!(tag_name, "ellipse" | "rect") => Some("rx"),
        "ry" if matches!(tag_name, "ellipse" | "rect") => Some("ry"),
        "x1" if tag_name == "line" => Some("x1"),
        "y1" if tag_name == "line" => Some("y1"),
        "x2" if tag_name == "line" => Some("x2"),
        "y2" if tag_name == "line" => Some("y2"),
        "href" if matches!(tag_name, "a" | "image" | "mpath" | "textPath" | "use") => Some("href"),
        _ => None,
    }
}

fn svg_has_view_box(tag_name: &str) -> bool {
    matches!(tag_name, "svg" | "marker" | "pattern" | "symbol" | "view")
}

fn svg_has_preserve_aspect_ratio(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "svg" | "image" | "marker" | "pattern" | "symbol" | "view"
    )
}

fn svg_has_path_length(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "circle" | "ellipse" | "line" | "path" | "polygon" | "polyline" | "rect"
    )
}

fn svg_has_text_length(tag_name: &str) -> bool {
    matches!(tag_name, "text" | "textPath" | "tspan")
}

fn svg_has_x_y(tag_name: &str) -> bool {
    matches!(
        tag_name,
        "svg" | "rect" | "image" | "use" | "text" | "tspan"
    )
}

fn svg_has_width_height(tag_name: &str) -> bool {
    matches!(tag_name, "svg" | "rect" | "image" | "use")
}

fn svg_has_center(tag_name: &str) -> bool {
    matches!(tag_name, "circle" | "ellipse" | "radialGradient")
}
