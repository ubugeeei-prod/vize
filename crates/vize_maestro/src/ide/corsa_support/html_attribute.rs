use tower_lsp::lsp_types::Url;
use vize_s0::{String, cstr};

use super::html_tag::native_dom_tag_info;
use super::svg_attribute::{canonical_svg_dom_attribute_name, mapped_svg_dom_attribute_property};

pub(crate) struct NativeDomAttributeInfo {
    pub(crate) category: &'static str,
    pub(crate) property_name: String,
    pub(crate) type_expression: String,
    pub(crate) documentation_url: String,
    pub(crate) is_boolean: bool,
}

pub(crate) struct HtmlAttributeVirtualDocument {
    pub(crate) content: String,
    pub(crate) hover_offset: usize,
    pub(crate) definition_offset: usize,
}

pub(crate) fn html_attribute_request_path(uri: &Url) -> String {
    cstr!("{}.html_attr.ts", uri.path())
}

pub(crate) fn html_attribute_virtual_document(
    tag_name: &str,
    attr_name: &str,
) -> Option<HtmlAttributeVirtualDocument> {
    let info = native_dom_attribute_info(tag_name, attr_name)?;
    let tag_info = native_dom_tag_info(tag_name)?;
    let element_type = tag_info.type_expression.as_str();
    let property_name = info.property_name.as_str();
    let content = cstr!(
        "/// <reference lib=\"es2022\" />\n\
         /// <reference lib=\"dom\" />\n\
         /// <reference lib=\"dom.iterable\" />\n\
         type __VizeDomElement = {element_type};\n\
         declare const __vizeDomElement: __VizeDomElement;\n\
         __vizeDomElement.{property_name};\n"
    );
    let property_offset = content.rfind(property_name)? + (property_name.len() / 2);

    Some(HtmlAttributeVirtualDocument {
        content,
        hover_offset: property_offset,
        definition_offset: property_offset,
    })
}

pub(crate) fn native_dom_attribute_info(
    tag_name: &str,
    attr_name: &str,
) -> Option<NativeDomAttributeInfo> {
    if crate::ide::is_component_tag(tag_name) {
        return None;
    }

    let tag_info = native_dom_tag_info(tag_name)?;
    let normalized_attr = normalize_attribute_name(tag_name, attr_name)?;
    let property_name = dom_attribute_property_name(tag_name, &normalized_attr)?;
    let is_boolean = vize_s0::is_boolean_attr(&normalized_attr);
    let category = if vize_s0::is_html_tag(tag_name) {
        "HTML attribute"
    } else if vize_s0::is_svg_tag(tag_name) {
        "SVG attribute"
    } else {
        "MathML attribute"
    };
    let documentation_url = if let Some(aria_name) = normalized_attr.strip_prefix("aria-") {
        cstr!(
            "https://developer.mozilla.org/en-US/docs/Web/Accessibility/ARIA/Attributes/aria-{aria_name}"
        )
    } else if normalized_attr.starts_with("data-") {
        String::from(
            "https://developer.mozilla.org/en-US/docs/Learn/HTML/Howto/Use_data_attributes",
        )
    } else if vize_s0::is_svg_tag(tag_name) {
        cstr!(
            "https://developer.mozilla.org/en-US/docs/Web/SVG/Reference/Attribute/{normalized_attr}"
        )
    } else if is_html_global_native_attribute(&normalized_attr)
        || is_dom_element_global_attribute(&normalized_attr)
    {
        cstr!(
            "https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Global_attributes/{normalized_attr}"
        )
    } else {
        cstr!(
            "https://developer.mozilla.org/en-US/docs/Web/HTML/Reference/Attributes/{normalized_attr}"
        )
    };
    let type_expression = cstr!(
        "{}[\"{}\"]",
        tag_info.type_expression.as_str(),
        property_name.as_str()
    );

    Some(NativeDomAttributeInfo {
        category,
        property_name,
        type_expression,
        documentation_url,
        is_boolean,
    })
}

fn normalize_attribute_name(tag_name: &str, attr_name: &str) -> Option<String> {
    let attr_name = attr_name.trim();
    if attr_name.is_empty()
        || attr_name.starts_with('@')
        || attr_name.starts_with('#')
        || attr_name.starts_with('[')
    {
        return None;
    }

    let attr_name = attr_name
        .strip_prefix(':')
        .or_else(|| attr_name.strip_prefix("v-bind:"))
        .unwrap_or(attr_name);
    if attr_name.starts_with("v-") {
        return None;
    }

    let attr_name = attr_name
        .split_once('.')
        .map_or(attr_name, |(name, _)| name)
        .trim();
    if attr_name.is_empty() || attr_name.contains(':') {
        return None;
    }

    if vize_s0::is_svg_tag(tag_name)
        && let Some(canonical) = canonical_svg_dom_attribute_name(tag_name, attr_name)
    {
        return Some(String::from(canonical));
    }

    Some(attr_name.to_ascii_lowercase().into())
}

fn dom_attribute_property_name(tag_name: &str, attr_name: &str) -> Option<String> {
    if let Some(data_name) = attr_name.strip_prefix("data-")
        && data_name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
    {
        return Some(String::from("dataset"));
    }

    if let Some(aria_name) = attr_name.strip_prefix("aria-") {
        return Some(cstr!("aria{}", kebab_to_pascal(aria_name)));
    }

    if vize_s0::is_svg_tag(tag_name)
        && let Some(property_name) = mapped_svg_dom_attribute_property(tag_name, attr_name)
    {
        return Some(String::from(property_name));
    }

    if let Some(mapped) = mapped_dom_attribute_property(attr_name)
        && is_known_native_attribute_for_tag(tag_name, attr_name)
    {
        return Some(String::from(mapped));
    }

    if is_known_native_attribute_for_tag(tag_name, attr_name) {
        return Some(kebab_to_camel(attr_name));
    }

    None
}

fn mapped_dom_attribute_property(attr_name: &str) -> Option<&'static str> {
    match attr_name {
        "accept-charset" => Some("acceptCharset"),
        "accesskey" => Some("accessKey"),
        "allowfullscreen" => Some("allowFullscreen"),
        "bgcolor" => Some("bgColor"),
        "char" => Some("ch"),
        "charoff" => Some("chOff"),
        "class" => Some("className"),
        "codebase" => Some("codeBase"),
        "codetype" => Some("codeType"),
        "colspan" => Some("colSpan"),
        "contenteditable" => Some("contentEditable"),
        "crossorigin" => Some("crossOrigin"),
        "datetime" => Some("dateTime"),
        "enterkeyhint" => Some("enterKeyHint"),
        "dirname" => Some("dirName"),
        "fetchpriority" => Some("fetchPriority"),
        "for" => Some("htmlFor"),
        "frameborder" => Some("frameBorder"),
        "formaction" => Some("formAction"),
        "formenctype" => Some("formEnctype"),
        "formmethod" => Some("formMethod"),
        "formnovalidate" => Some("formNoValidate"),
        "formtarget" => Some("formTarget"),
        "http-equiv" => Some("httpEquiv"),
        "imagesizes" => Some("imageSizes"),
        "imagesrcset" => Some("imageSrcset"),
        "inputmode" => Some("inputMode"),
        "ismap" => Some("isMap"),
        "longdesc" => Some("longDesc"),
        "marginheight" => Some("marginHeight"),
        "marginwidth" => Some("marginWidth"),
        "maxlength" => Some("maxLength"),
        "minlength" => Some("minLength"),
        "nomodule" => Some("noModule"),
        "nowrap" => Some("noWrap"),
        "novalidate" => Some("noValidate"),
        "playsinline" => Some("playsInline"),
        "readonly" => Some("readOnly"),
        "referrerpolicy" => Some("referrerPolicy"),
        "rowspan" => Some("rowSpan"),
        "tabindex" => Some("tabIndex"),
        "usemap" => Some("useMap"),
        "valign" => Some("vAlign"),
        _ => None,
    }
}

fn is_known_native_attribute_for_tag(tag_name: &str, attr_name: &str) -> bool {
    if vize_s0::is_html_tag(tag_name) {
        return is_html_global_native_attribute(attr_name)
            || HTML_TAG_ATTRIBUTES
                .iter()
                .any(|(tag, attrs)| *tag == tag_name && attrs.contains(&attr_name));
    }

    if vize_s0::is_svg_tag(tag_name) || vize_s0::is_math_ml_tag(tag_name) {
        return is_dom_element_global_attribute(attr_name);
    }

    false
}

#[rustfmt::skip]
const GLOBAL_ATTRIBUTES: &[&str] = &[
    "accesskey",
    "autocapitalize",
    "autofocus",
    "class",
    "contenteditable",
    "dir",
    "draggable",
    "enterkeyhint",
    "hidden",
    "id",
    "inert",
    "inputmode",
    "lang",
    "nonce",
    "popover",
    "role",
    "slot",
    "spellcheck",
    "style",
    "tabindex",
    "title",
    "translate",
];

#[rustfmt::skip]
const HTML_TAG_ATTRIBUTES: &[(&str, &[&str])] = &[
    ("a", &["charset", "coords", "download", "href", "hreflang", "name", "ping", "referrerpolicy", "rel", "rev", "shape", "target", "type"]),
    ("area", &["alt", "coords", "download", "href", "ping", "referrerpolicy", "rel", "shape", "target"]),
    ("audio", &["autoplay", "controls", "crossorigin", "loop", "muted", "preload", "src"]),
    ("base", &["href", "target"]),
    ("blockquote", &["cite"]),
    ("button", &["disabled", "form", "formaction", "formenctype", "formmethod", "formnovalidate", "formtarget", "name", "type", "value"]),
    ("canvas", &["height", "width"]),
    ("col", &["align", "char", "charoff", "span", "valign", "width"]),
    ("colgroup", &["align", "char", "charoff", "span", "valign", "width"]),
    ("data", &["value"]),
    ("del", &["cite", "datetime"]),
    ("details", &["open"]),
    ("dialog", &["open"]),
    ("embed", &["height", "src", "type", "width"]),
    ("fieldset", &["disabled", "form", "name"]),
    ("form", &["accept-charset", "action", "autocomplete", "enctype", "method", "name", "novalidate", "rel", "target"]),
    ("iframe", &["align", "allow", "allowfullscreen", "frameborder", "height", "loading", "longdesc", "marginheight", "marginwidth", "name", "referrerpolicy", "sandbox", "scrolling", "src", "srcdoc", "width"]),
    ("img", &["align", "alt", "border", "crossorigin", "decoding", "fetchpriority", "height", "hspace", "ismap", "loading", "longdesc", "lowsrc", "name", "referrerpolicy", "sizes", "src", "srcset", "usemap", "vspace", "width"]),
    ("input", &["accept", "align", "alt", "autocomplete", "capture", "checked", "dirname", "disabled", "form", "formaction", "formenctype", "formmethod", "formnovalidate", "formtarget", "height", "list", "max", "maxlength", "min", "minlength", "multiple", "name", "pattern", "placeholder", "readonly", "required", "size", "src", "step", "type", "usemap", "value", "webkitdirectory", "width"]),
    ("ins", &["cite", "datetime"]),
    ("label", &["for", "form"]),
    ("li", &["value"]),
    ("link", &["as", "charset", "crossorigin", "disabled", "fetchpriority", "href", "hreflang", "imagesizes", "imagesrcset", "integrity", "media", "referrerpolicy", "rel", "rev", "sizes", "target", "type"]),
    ("map", &["name"]),
    ("meta", &["content", "http-equiv", "name"]),
    ("meter", &["high", "low", "max", "min", "optimum", "value"]),
    ("object", &["align", "archive", "border", "code", "codebase", "codetype", "data", "declare", "form", "height", "hspace", "name", "standby", "type", "usemap", "vspace", "width"]),
    ("ol", &["reversed", "start", "type"]),
    ("optgroup", &["disabled", "label"]),
    ("option", &["disabled", "label", "selected", "value"]),
    ("output", &["for", "form", "name"]),
    ("param", &["name", "value"]),
    ("progress", &["max", "value"]),
    ("q", &["cite"]),
    ("script", &["async", "charset", "crossorigin", "defer", "event", "fetchpriority", "for", "integrity", "nomodule", "referrerpolicy", "src", "type"]),
    ("select", &["autocomplete", "disabled", "form", "multiple", "name", "required", "size"]),
    ("source", &["height", "media", "sizes", "src", "srcset", "type", "width"]),
    ("style", &["disabled", "media", "type"]),
    ("td", &["align", "axis", "bgcolor", "char", "charoff", "colspan", "headers", "height", "nowrap", "rowspan", "valign", "width"]),
    ("th", &["abbr", "align", "axis", "bgcolor", "char", "charoff", "colspan", "headers", "height", "nowrap", "rowspan", "scope", "valign", "width"]),
    ("textarea", &["autocomplete", "cols", "dirname", "disabled", "form", "maxlength", "minlength", "name", "placeholder", "readonly", "required", "rows", "wrap"]),
    ("time", &["datetime"]),
    ("track", &["default", "kind", "label", "src", "srclang"]),
    ("video", &["autoplay", "controls", "crossorigin", "height", "loop", "muted", "playsinline", "poster", "preload", "src", "width"]),
];

fn is_html_global_native_attribute(attr_name: &str) -> bool {
    GLOBAL_ATTRIBUTES.contains(&attr_name)
}

fn is_dom_element_global_attribute(attr_name: &str) -> bool {
    matches!(
        attr_name,
        "class" | "id" | "nonce" | "role" | "slot" | "style" | "tabindex"
    )
}

fn kebab_to_camel(value: &str) -> String {
    let mut result = String::with_capacity(value.len());
    let mut uppercase_next = false;
    for ch in value.chars() {
        if ch == '-' {
            uppercase_next = true;
        } else if uppercase_next {
            result.push(ch.to_ascii_uppercase());
            uppercase_next = false;
        } else {
            result.push(ch);
        }
    }
    result
}

fn kebab_to_pascal(value: &str) -> String {
    let camel = kebab_to_camel(value);
    let mut chars = camel.chars();
    let Some(first) = chars.next() else {
        return String::from("");
    };
    let mut result = String::with_capacity(camel.len());
    result.push(first.to_ascii_uppercase());
    result.extend(chars);
    result
}
