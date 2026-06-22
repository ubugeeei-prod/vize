use tower_lsp::lsp_types::Url;
use vize_carton::{String, cstr};

pub(crate) struct HtmlTagVirtualDocument {
    pub(crate) content: String,
    pub(crate) hover_offset: usize,
    pub(crate) definition_offset: usize,
}

pub(crate) fn html_tag_request_path(uri: &Url) -> String {
    cstr!("{}.html_tag.ts", uri.path())
}

pub(crate) fn html_tag_virtual_document(tag_name: &str) -> Option<HtmlTagVirtualDocument> {
    if !is_native_html_tag_candidate(tag_name) {
        return None;
    }

    let content = cstr!(
        "/// <reference lib=\"es2022\" />\n\
         /// <reference lib=\"dom\" />\n\
         /// <reference lib=\"dom.iterable\" />\n\
         type __VizeHtmlElement = HTMLElementTagNameMap[\"{tag_name}\"];\n\
         declare const __vizeHtmlElement: __VizeHtmlElement;\n\
         __vizeHtmlElement;\n"
    );
    let definition_offset = content.find("HTMLElementTagNameMap")?;
    let hover_offset = content.rfind("__vizeHtmlElement")?;

    Some(HtmlTagVirtualDocument {
        content,
        hover_offset,
        definition_offset,
    })
}

fn is_native_html_tag_candidate(tag_name: &str) -> bool {
    !tag_name.is_empty()
        && !matches!(
            tag_name,
            "component" | "template" | "slot" | "teleport" | "suspense"
        )
        && tag_name
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
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
            &doc.content[doc.hover_offset..doc.hover_offset + "__vizeHtmlElement".len()],
            "__vizeHtmlElement",
        );
    }
}
