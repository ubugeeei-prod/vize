//! Whole-module byte comparison for the production-parity oracle.

use vize_atelier_sfc::{SfcCompileResult, SfcError};

/// The first field on which the S2-selected and forced-legacy modules differ.
pub fn divergence(
    selected: &SfcCompileResult,
    legacy: &Result<SfcCompileResult, SfcError>,
) -> Option<String> {
    let legacy = match legacy {
        Ok(legacy) => legacy,
        Err(error) => return Some(format!("legacy lane failed: {error:?}")),
    };
    if selected.code != legacy.code {
        return Some(format!(
            "code differs at byte {}:\n--- s2\n{}\n--- legacy\n{}",
            first_diff(&selected.code, &legacy.code),
            window(&selected.code, &legacy.code),
            window(&legacy.code, &selected.code),
        ));
    }
    if selected.css != legacy.css {
        return Some("css differs".to_owned());
    }
    let messages = |errors: &[SfcError]| -> Vec<String> {
        errors
            .iter()
            .map(|error| error.message.to_string())
            .collect()
    };
    if messages(&selected.errors) != messages(&legacy.errors) {
        return Some(format!(
            "errors differ: s2={:?} legacy={:?}",
            messages(&selected.errors),
            messages(&legacy.errors)
        ));
    }
    if messages(&selected.warnings) != messages(&legacy.warnings) {
        return Some(format!(
            "warnings differ: s2={:?} legacy={:?}",
            messages(&selected.warnings),
            messages(&legacy.warnings)
        ));
    }
    None
}

/// A selected compile error must agree with the forced legacy result too.
pub fn error_divergence(
    selected: &SfcError,
    legacy: &Result<SfcCompileResult, SfcError>,
) -> Option<String> {
    match legacy {
        Err(legacy) if selected.code == legacy.code && selected.message == legacy.message => None,
        Err(legacy) => Some(format!(
            "compile errors differ: selected={selected:?} legacy={legacy:?}"
        )),
        Ok(_) => Some(format!(
            "selected lane failed but legacy lane compiled: {selected:?}"
        )),
    }
}

fn first_diff(left: &str, right: &str) -> usize {
    left.bytes()
        .zip(right.bytes())
        .position(|(left, right)| left != right)
        .unwrap_or_else(|| left.len().min(right.len()))
}

fn window(source: &str, other: &str) -> String {
    let diff = first_diff(source, other);
    let start = source[..diff]
        .char_indices()
        .rev()
        .nth(120)
        .map_or(0, |(index, _)| index);
    let end = source[diff..]
        .char_indices()
        .nth(200)
        .map_or(source.len(), |(index, _)| diff + index);
    source[start..end].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error(code: &str, message: &str) -> SfcError {
        SfcError {
            code: Some(code.into()),
            message: message.into(),
            loc: None,
        }
    }

    #[test]
    fn selected_errors_require_matching_legacy_errors() {
        let selected = error("parse", "invalid end tag");
        assert_eq!(
            error_divergence(&selected, &Err(error("parse", "invalid end tag"))),
            None
        );
        assert!(error_divergence(&selected, &Err(error("parse", "different"))).is_some());
        assert!(error_divergence(&selected, &Err(error("codegen", "invalid end tag"))).is_some());
        assert!(
            error_divergence(
                &selected,
                &Ok(SfcCompileResult {
                    code: "compiled".into(),
                    css: None,
                    map: None,
                    errors: Vec::new(),
                    warnings: Vec::new(),
                    bindings: None,
                    macro_artifacts: Vec::new(),
                })
            )
            .is_some()
        );
    }
}
