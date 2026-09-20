//! Native JSX document coordinates and the shared generator used by tests.

use vize_canon::virtual_ts::VizeMapping;

pub(in crate::ide) struct JsxVirtualTs {
    pub(in crate::ide) code: String,
    pub(in crate::ide) mappings: Vec<VizeMapping>,
    pub(in crate::ide) import_source_map: vize_canon::batch::ImportSourceMap,
}

#[cfg(test)]
pub(in crate::ide) fn generate_jsx_virtual_ts(
    source: &str,
    lang: vize_atelier_jsx::JsxLang,
) -> Option<JsxVirtualTs> {
    let generated = vize_canon::batch::generate_jsx_document_virtual_ts(
        std::path::Path::new("Component.tsx"),
        source,
        lang,
    )
    .ok()?;
    Some(JsxVirtualTs {
        code: generated.code.into(),
        mappings: generated.mappings,
        import_source_map: Default::default(),
    })
}
