use vize_carton::String;

use super::js_string::push_js_string_literal;

/// Hash state used to classify Vite HMR updates.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct HmrHashes {
    /// Compiled script hash.
    pub script_hash: Option<String>,
    /// Compiled template hash.
    pub template_hash: Option<String>,
    /// Compiled style hash.
    pub style_hash: Option<String>,
}

/// Returns true when any tracked SFC section changed.
pub fn has_hmr_changes(prev: Option<&HmrHashes>, next: &HmrHashes) -> bool {
    let Some(prev) = prev else {
        return true;
    };

    did_hash_change(prev.script_hash.as_deref(), next.script_hash.as_deref())
        || did_hash_change(prev.template_hash.as_deref(), next.template_hash.as_deref())
        || did_hash_change(prev.style_hash.as_deref(), next.style_hash.as_deref())
}

/// Detect the least disruptive Vue HMR update type.
pub fn detect_hmr_update_type(prev: Option<&HmrHashes>, next: &HmrHashes) -> &'static str {
    let Some(prev) = prev else {
        return "full-reload";
    };

    if did_hash_change(prev.script_hash.as_deref(), next.script_hash.as_deref()) {
        return "full-reload";
    }

    let template_changed =
        did_hash_change(prev.template_hash.as_deref(), next.template_hash.as_deref());
    let style_changed = did_hash_change(prev.style_hash.as_deref(), next.style_hash.as_deref());

    if style_changed && !template_changed {
        return "style-only";
    }
    if template_changed {
        return "template-only";
    }
    "full-reload"
}

/// Generate the runtime HMR bridge appended to compiled SFC modules.
pub fn generate_hmr_code(scope_id: &str, update_type: &str) -> String {
    let mut code = String::with_capacity(900 + scope_id.len() + update_type.len());
    code.push_str("\nif (import.meta.hot) {\n  _sfc_main.__hmrId = ");
    push_js_string_literal(&mut code, scope_id);
    code.push_str(";\n  _sfc_main.__hmrUpdateType = ");
    push_js_string_literal(&mut code, update_type);
    code.push_str(
        ";\n\n  import.meta.hot.accept((mod) => {\n    if (!mod) return;\n    const { default: updated } = mod;\n    if (typeof __VUE_HMR_RUNTIME__ !== 'undefined') {\n      const updateType = updated.__hmrUpdateType || 'full-reload';\n      if (updateType === 'template-only') {\n        __VUE_HMR_RUNTIME__.rerender(updated.__hmrId, updated.render);\n      } else {\n        __VUE_HMR_RUNTIME__.reload(updated.__hmrId, updated);\n      }\n    }\n  });\n\n  import.meta.hot.on('vize:update', (data) => {\n    if (data.id !== _sfc_main.__hmrId) return;\n\n    if (data.type === 'style-only') {\n      const styleId = 'vize-style-' + _sfc_main.__hmrId;\n      const styleEl = document.getElementById(styleId);\n      if (styleEl && data.css) {\n        styleEl.textContent = data.css;\n      }\n    }\n  });\n\n  if (typeof __VUE_HMR_RUNTIME__ !== 'undefined') {\n    __VUE_HMR_RUNTIME__.createRecord(_sfc_main.__hmrId, _sfc_main);\n  }\n}",
    );
    code
}

fn did_hash_change(prev: Option<&str>, next: Option<&str>) -> bool {
    prev != next
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hashes(script: &str, template: &str, style: &str) -> HmrHashes {
        HmrHashes {
            script_hash: Some(script.into()),
            template_hash: Some(template.into()),
            style_hash: Some(style.into()),
        }
    }

    #[test]
    fn detects_hmr_update_types() {
        let base = hashes("script", "template", "style");

        assert_eq!(detect_hmr_update_type(None, &base), "full-reload");
        assert_eq!(
            detect_hmr_update_type(Some(&base), &hashes("script", "template-2", "style")),
            "template-only"
        );
        assert_eq!(
            detect_hmr_update_type(Some(&base), &hashes("script", "template", "style-2")),
            "style-only"
        );
        assert_eq!(
            detect_hmr_update_type(Some(&base), &hashes("script-2", "template", "style")),
            "full-reload"
        );
    }

    #[test]
    fn snapshots_hmr_code() {
        insta::assert_snapshot!(generate_hmr_code("abc123", "template-only"));
    }
}
