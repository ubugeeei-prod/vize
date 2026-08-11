//! Vue-specific semantic diagnostics and request URI ownership.

use std::path::{Path, PathBuf};

use vize_carton::cstr;

use super::Diagnostic;

pub(super) fn uri_to_path(uri: &str, working_dir: &Path) -> Option<PathBuf> {
    if uri.starts_with("file://") {
        return crate::file_uri::file_uri_to_path(uri);
    }
    if uri.contains("://") {
        return None;
    }
    let path = Path::new(uri);
    if path.is_absolute() {
        Some(path.to_path_buf())
    } else {
        Some(working_dir.join(path))
    }
}

pub(super) fn collect_sfc_compile_diagnostic(
    _uri: &str,
    source: &str,
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
) -> Option<Diagnostic> {
    let script_setup = descriptor.script_setup.as_ref()?;
    if !vize_atelier_sfc::script_setup_has_semantic_validator_candidates(&script_setup.content) {
        return None;
    }

    let Err(error) = vize_atelier_sfc::validate_script_setup_semantics_located(
        &script_setup.content,
        script_setup.loc.start,
        source,
    ) else {
        return None;
    };

    let (line, column) = if let Some(loc) = error.loc.as_ref() {
        (
            (loc.start_line as u32).saturating_sub(1),
            (loc.start_column as u32).saturating_sub(1),
        )
    } else {
        let offset = crate::batch::sfc_block_fallback_offset(descriptor)
            .map_or(0, |(offset, _block)| offset);
        vize_carton::line_index::offset_to_line_col(source, offset)
    };

    let message = match error.code.as_deref() {
        Some(code) => cstr!("[{}] {}", code, error.message),
        None => error.message.clone(),
    };

    Some(Diagnostic {
        message,
        severity: "error".into(),
        line,
        column,
        code: error.code.clone(),
    })
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::uri_to_path;

    #[test]
    fn uri_to_path_decodes_multi_byte_utf8_escapes() {
        assert_eq!(
            uri_to_path(
                "file:///Users/foo/%E3%83%86%E3%82%B9%E3%83%88/App.vue",
                Path::new("/wd")
            ),
            Some(PathBuf::from("/Users/foo/テスト/App.vue"))
        );
    }

    #[test]
    fn uri_to_path_decodes_spaces() {
        assert_eq!(
            uri_to_path("file:///work/my%20app/App.vue", Path::new("/wd")),
            Some(PathBuf::from("/work/my app/App.vue"))
        );
    }

    #[test]
    fn uri_to_path_rejects_invalid_utf8_escapes() {
        assert_eq!(
            uri_to_path("file:///work/%FF%FE/App.vue", Path::new("/wd")),
            None
        );
    }

    #[test]
    fn uri_to_path_resolves_relative_paths_against_working_dir() {
        assert_eq!(
            uri_to_path("src/App.vue", Path::new("/workspace/project")),
            Some(PathBuf::from("/workspace/project/src/App.vue"))
        );
    }

    #[test]
    fn uri_to_path_rejects_non_file_schemes() {
        assert_eq!(uri_to_path("untitled://buffer-1", Path::new("/wd")), None);
    }
}
