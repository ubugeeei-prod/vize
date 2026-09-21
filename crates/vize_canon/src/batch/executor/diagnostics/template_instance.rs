//! Batch checking reports a name a template cannot resolve with the instance
//! codes of [`crate::template_instance_names`].

use crate::batch::{OriginalPosition, SfcBlockType};
use crate::template_instance_names::instance_diagnostic;
use vize_carton::String;

/// The instance-level `(code, message)` for a lexical lookup failure inside a
/// template, or `None` when the diagnostic stays as TypeScript reported it.
pub(in crate::batch::executor) fn template_instance_diagnostic(
    code: Option<u32>,
    original: &OriginalPosition,
    message: &str,
) -> Option<(u32, String)> {
    if original.block_type != Some(SfcBlockType::Template) {
        return None;
    }
    instance_diagnostic(code?, message)
}

#[cfg(test)]
mod tests {
    use super::template_instance_diagnostic;
    use crate::batch::{OriginalPosition, SfcBlockType};
    use std::path::PathBuf;

    fn at(block_type: SfcBlockType) -> OriginalPosition {
        OriginalPosition {
            path: PathBuf::from("App.vue"),
            line: 0,
            column: 0,
            block_type: Some(block_type),
        }
    }

    #[test]
    fn only_template_names_are_instance_properties() {
        assert_eq!(
            template_instance_diagnostic(
                Some(2304),
                &at(SfcBlockType::Template),
                "Cannot find name 'missing'."
            ),
            Some((
                2339,
                "Property 'missing' does not exist on the component instance.".into()
            ))
        );
        assert_eq!(
            template_instance_diagnostic(
                Some(2304),
                &at(SfcBlockType::ScriptSetup),
                "Cannot find name 'missing'."
            ),
            None
        );
        assert_eq!(
            template_instance_diagnostic(
                None,
                &at(SfcBlockType::Template),
                "Cannot find name 'missing'."
            ),
            None
        );
    }
}
