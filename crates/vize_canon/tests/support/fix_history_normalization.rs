//! Exact normalization of generated ordinal names in the existing corpus.

use super::super::Diagnostic;

pub(crate) fn normalize_generated_check_ordinals(diagnostics: &[Diagnostic]) -> Vec<Diagnostic> {
    diagnostics
        .iter()
        .cloned()
        .map(|mut diagnostic| {
            let mut message = diagnostic.message.as_str();
            let mut normalized = std::string::String::with_capacity(message.len());
            while let Some(at) = message.find("_CheckProps_") {
                let after_marker = at + "_CheckProps_".len();
                normalized.push_str(&message[..after_marker]);
                let digit_count = message[after_marker..]
                    .bytes()
                    .take_while(u8::is_ascii_digit)
                    .count();
                if digit_count > 0 {
                    normalized.push('#');
                }
                message = &message[after_marker + digit_count..];
            }
            normalized.push_str(message);
            diagnostic.message = normalized;
            diagnostic
        })
        .collect()
}

pub(crate) fn assert_generated_check_ordinal_normalization_preserves_markers_without_digits() {
    let diagnostics = [Diagnostic {
        file: String::from("src/App.vue"),
        line: 1,
        column: 1,
        severity: 1,
        code: Some(2322),
        message: String::from("_CheckProps_ stays distinct from _CheckProps_12"),
    }];

    let normalized = normalize_generated_check_ordinals(&diagnostics);

    assert_eq!(
        normalized[0].message,
        "_CheckProps_ stays distinct from _CheckProps_#"
    );
}
