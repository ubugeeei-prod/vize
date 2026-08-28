use vize_s0::String;

type DiagnosticSnapshot = Vec<(String, Option<u32>, String)>;

/// Normalize TypeScript's target-side parameter labels in function assignment
/// diagnostics while keeping the authored parameter, code, anchor, and type
/// text exact.
pub(super) fn normalize_target_parameter_names(
    snapshot: Option<DiagnosticSnapshot>,
) -> Option<DiagnosticSnapshot> {
    snapshot.map(|rows| {
        rows.into_iter()
            .map(|(file, code, message)| {
                (
                    file,
                    code,
                    normalize_target_parameter_name(message.as_str()),
                )
            })
            .collect()
    })
}

/// Replace only the generated side of `Types of parameters ...` diagnostic
/// rows. TypeScript may report that side as a tuple label, callback parameter,
/// or rest parameter name without changing the assignability behavior.
fn normalize_target_parameter_name(message: &str) -> String {
    let marker = "Types of parameters '";
    let separator = "' and '";
    let suffix = "' are incompatible.";
    let mut normalized = std::string::String::with_capacity(message.len());
    let mut rest = message;

    while let Some(marker_start) = rest.find(marker) {
        let parameter_start = marker_start + marker.len();
        let Some(separator_start) = rest[parameter_start..].find(separator) else {
            break;
        };
        let target_start = parameter_start + separator_start + separator.len();
        let Some(target_end) = rest[target_start..].find(suffix) else {
            break;
        };
        let target_end = target_start + target_end;

        normalized.push_str(&rest[..target_start]);
        normalized.push_str("<target>");
        rest = &rest[target_end..];
    }

    if normalized.is_empty() {
        String::from(message)
    } else {
        normalized.push_str(rest);
        String::from(normalized)
    }
}
