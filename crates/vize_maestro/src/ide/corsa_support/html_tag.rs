use tower_lsp::lsp_types::Url;
use vize_s0::{String, cstr};

pub(crate) struct NativeDomTagInfo {
    pub(crate) category: &'static str,
    pub(crate) type_expression: String,
    pub(crate) documentation_url: String,
}

pub(crate) struct HtmlTagVirtualDocument {
    pub(crate) content: String,
    pub(crate) hover_offset: usize,
    pub(crate) definition_offset: usize,
}

pub(crate) fn html_tag_request_path(uri: &Url) -> String {
    cstr!("{}.html_tag.ts", uri.path())
}

pub(crate) fn html_tag_virtual_document(tag_name: &str) -> Option<HtmlTagVirtualDocument> {
    let info = native_dom_tag_info(tag_name)?;

    let type_expression = info.type_expression.as_str();
    let content = cstr!(
        "/// <reference lib=\"es2022\" />\n\
         /// <reference lib=\"dom\" />\n\
         /// <reference lib=\"dom.iterable\" />\n\
         type __VizeDomElement = {type_expression};\n\
         declare const __vizeDomElement: __VizeDomElement;\n\
         __vizeDomElement;\n"
    );
    let definition_symbol = dom_definition_symbol(tag_name)?;
    let definition_offset = content.find(definition_symbol)?;
    let hover_offset = content.rfind("__vizeDomElement")?;

    Some(HtmlTagVirtualDocument {
        content,
        hover_offset,
        definition_offset,
    })
}

pub(crate) fn native_dom_tag_info(tag_name: &str) -> Option<NativeDomTagInfo> {
    if matches!(
        tag_name,
        "component" | "template" | "slot" | "teleport" | "suspense"
    ) {
        return None;
    }

    if has_html_element_tag_name_map_entry(tag_name) {
        return Some(NativeDomTagInfo {
            category: "HTML element",
            type_expression: cstr!("HTMLElementTagNameMap[\"{tag_name}\"]"),
            documentation_url: cstr!(
                "https://developer.mozilla.org/en-US/docs/Web/HTML/Element/{tag_name}"
            ),
        });
    }
    if has_svg_element_tag_name_map_entry(tag_name) {
        return Some(NativeDomTagInfo {
            category: "SVG element",
            type_expression: cstr!("SVGElementTagNameMap[\"{tag_name}\"]"),
            documentation_url: cstr!(
                "https://developer.mozilla.org/en-US/docs/Web/SVG/Element/{tag_name}"
            ),
        });
    }
    if has_math_ml_element_tag_name_map_entry(tag_name) {
        return Some(NativeDomTagInfo {
            category: "MathML element",
            type_expression: cstr!("MathMLElementTagNameMap[\"{tag_name}\"]"),
            documentation_url: cstr!(
                "https://developer.mozilla.org/en-US/docs/Web/MathML/Element/{tag_name}"
            ),
        });
    }
    if vize_s0::is_math_ml_tag(tag_name) {
        return Some(NativeDomTagInfo {
            category: "MathML element",
            type_expression: String::from("MathMLElement"),
            documentation_url: cstr!(
                "https://developer.mozilla.org/en-US/docs/Web/MathML/Element/{tag_name}"
            ),
        });
    }
    None
}

fn dom_definition_symbol(tag_name: &str) -> Option<&'static str> {
    if has_html_element_tag_name_map_entry(tag_name) {
        Some("HTMLElementTagNameMap")
    } else if has_svg_element_tag_name_map_entry(tag_name) {
        Some("SVGElementTagNameMap")
    } else if has_math_ml_element_tag_name_map_entry(tag_name) {
        Some("MathMLElementTagNameMap")
    } else if vize_s0::is_math_ml_tag(tag_name) {
        Some("MathMLElement")
    } else {
        None
    }
}

fn has_html_element_tag_name_map_entry(tag_name: &str) -> bool {
    vize_s0::is_html_tag(tag_name) && !matches!(tag_name, "param")
}

fn has_svg_element_tag_name_map_entry(tag_name: &str) -> bool {
    vize_s0::is_svg_tag(tag_name)
        && !matches!(
            tag_name,
            "color-profile"
                | "discard"
                | "hatch"
                | "hatchpath"
                | "mesh"
                | "meshgradient"
                | "meshpatch"
                | "meshrow"
                | "solidcolor"
                | "unknown"
        )
}

fn has_math_ml_element_tag_name_map_entry(tag_name: &str) -> bool {
    vize_s0::is_math_ml_tag(tag_name)
        && matches!(
            tag_name,
            "annotation"
                | "annotation-xml"
                | "maction"
                | "math"
                | "merror"
                | "mfrac"
                | "mi"
                | "mmultiscripts"
                | "mn"
                | "mo"
                | "mover"
                | "mpadded"
                | "mphantom"
                | "mprescripts"
                | "mroot"
                | "mrow"
                | "ms"
                | "mspace"
                | "msqrt"
                | "mstyle"
                | "msub"
                | "msubsup"
                | "msup"
                | "mtable"
                | "mtd"
                | "mtext"
                | "mtr"
                | "munder"
                | "munderover"
                | "semantics"
        )
}

#[cfg(test)]
mod tests {
    #[test]
    fn html_tag_virtual_document_queries_lib_dom_types() {
        let doc = super::html_tag_virtual_document("button").expect("html tag doc");

        assert!(doc.content.contains("HTMLElementTagNameMap[\"button\"]"));
        assert_eq!(
            &doc.content
                [doc.definition_offset..doc.definition_offset + "HTMLElementTagNameMap".len()],
            "HTMLElementTagNameMap",
        );
        assert_eq!(
            &doc.content[doc.hover_offset..doc.hover_offset + "__vizeDomElement".len()],
            "__vizeDomElement",
        );
    }

    #[test]
    fn html_tag_virtual_document_supports_svg_tags() {
        let doc = super::html_tag_virtual_document("svg").expect("svg tag doc");

        assert!(doc.content.contains("SVGElementTagNameMap[\"svg\"]"));
        assert_eq!(
            &doc.content
                [doc.definition_offset..doc.definition_offset + "SVGElementTagNameMap".len()],
            "SVGElementTagNameMap",
        );
    }

    #[test]
    fn html_tag_virtual_document_supports_mathml_tag_map_entries() {
        let doc = super::html_tag_virtual_document("math").expect("math tag doc");

        assert!(doc.content.contains("MathMLElementTagNameMap[\"math\"]"));
        assert_eq!(
            &doc.content
                [doc.definition_offset..doc.definition_offset + "MathMLElementTagNameMap".len()],
            "MathMLElementTagNameMap",
        );
    }

    #[test]
    fn html_tag_virtual_document_uses_mathml_fallback_for_unmapped_tags() {
        let doc = super::html_tag_virtual_document("menclose").expect("menclose tag doc");

        assert!(
            doc.content
                .contains("type __VizeDomElement = MathMLElement;")
        );
        assert_eq!(
            &doc.content[doc.definition_offset..doc.definition_offset + "MathMLElement".len()],
            "MathMLElement",
        );
    }

    #[test]
    fn native_dom_tag_info_rejects_custom_elements() {
        assert!(super::native_dom_tag_info("my-element").is_none());
    }

    #[test]
    fn native_dom_tag_info_rejects_vue_builtins() {
        assert!(super::native_dom_tag_info("template").is_none());
        assert!(super::native_dom_tag_info("slot").is_none());
        assert!(super::native_dom_tag_info("component").is_none());
    }

    #[test]
    fn native_dom_tag_info_rejects_html_tags_missing_dom_map_entries() {
        assert!(super::native_dom_tag_info("param").is_none());
        assert!(super::html_tag_virtual_document("param").is_none());
    }

    #[test]
    fn native_dom_tag_info_rejects_svg_tags_missing_dom_map_entries() {
        for tag_name in [
            "color-profile",
            "discard",
            "hatch",
            "hatchpath",
            "mesh",
            "meshgradient",
            "meshpatch",
            "meshrow",
            "solidcolor",
            "unknown",
        ] {
            assert!(super::native_dom_tag_info(tag_name).is_none(), "{tag_name}");
            assert!(
                super::html_tag_virtual_document(tag_name).is_none(),
                "{tag_name}",
            );
        }
    }
}
