use std::path::Path;

use vize_atelier_sfc::{SfcCompileOptions, SfcCompileRequest};
use vize_carton::ToCompactString;

use super::CanonGraphSettings;

pub(super) fn compile_request(settings: &CanonGraphSettings, path: &Path) -> SfcCompileRequest {
    let mut options = SfcCompileOptions::default();
    options.parse.filename = path.to_string_lossy().to_compact_string();
    options.template.dialect = settings.dialect;
    options.template.compiler_options = Some(vize_atelier_dom::DomCompilerOptions {
        comments: true,
        experimental_in_tag_comments: settings.experimental_in_tag_comments,
        dialect: settings.dialect,
        ..Default::default()
    });
    SfcCompileRequest::new(options, settings.template_syntax)
}
