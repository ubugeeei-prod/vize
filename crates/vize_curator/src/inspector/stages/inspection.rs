//! Failed inspections are observations, never substitute dump pages.

use core::fmt::Display;
use vize_l0::{String, ToCompactString};

use super::StageUnavailable;

pub(super) fn record<E: Display>(
    unavailable: &mut Vec<StageUnavailable>,
    path: &str,
    stage: &str,
    pass: &str,
    rendered: Result<String, E>,
) -> Option<String> {
    match rendered {
        Ok(text) => Some(text),
        Err(error) => {
            unavailable.push(StageUnavailable {
                path: Some(String::from(path)),
                stage: String::from(stage),
                pass: String::from(pass),
                reason: error.to_compact_string(),
            });
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::record;
    use vize_l0::String;
    use vize_l2::dump::NativeDumpError;

    #[test]
    fn failed_inspection_cannot_become_collector_text_and_success_stays_exact() {
        let mut unavailable = Vec::new();
        let result = record(
            &mut unavailable,
            "src/App.vue",
            "s2",
            "hoist-static",
            Err(NativeDumpError::JsBindingUnsupported {
                span: vize_l0::Span::new(4, 9),
            }),
        );
        assert!(result.is_none());
        assert_eq!(unavailable.len(), 1);
        let failure = unavailable.first().unwrap();
        assert_eq!(failure.pass.as_str(), "hoist-static");
        assert_eq!(
            failure.reason.as_str(),
            "native binding dump unsupported at bytes 4..9"
        );
        let result = record(
            &mut unavailable,
            "src/App.vue",
            "s2",
            "lower",
            Ok::<_, NativeDumpError>(String::from("canonical full dump\n")),
        );
        assert_eq!(result.as_deref(), Some("canonical full dump\n"));
        assert_eq!(unavailable.len(), 1);
    }
}
