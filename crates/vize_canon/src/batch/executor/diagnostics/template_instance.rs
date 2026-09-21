//! A name a template cannot resolve is a property the component instance
//! lacks. The virtual module binds template names as lexical variables, so
//! TypeScript reports the lexical codes (`TS2304` / `TS2552`); the Vue
//! toolchain reads the same names from the instance and reports `TS2339` /
//! `TS2551`. Authored templates get the instance codes.

use crate::batch::{OriginalPosition, SfcBlockType};
use vize_carton::{String, cstr};

const CANNOT_FIND_NAME: &str = "Cannot find name '";

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
    let rest = message.strip_prefix(CANNOT_FIND_NAME)?;
    let (name, rest) = rest.split_once('\'')?;
    match code? {
        2304 => Some((
            2339,
            cstr!("Property '{name}' does not exist on the component instance."),
        )),
        2552 => {
            let suggestion = rest.strip_prefix(". Did you mean '")?.strip_suffix("'?")?;
            Some((
                2551,
                cstr!(
                    "Property '{name}' does not exist on the component instance. Did you mean '{suggestion}'?"
                ),
            ))
        }
        _ => None,
    }
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
    fn a_template_name_is_an_instance_property() {
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
                Some(2552),
                &at(SfcBlockType::Template),
                "Cannot find name 'cout'. Did you mean 'count'?"
            ),
            Some((
                2551,
                "Property 'cout' does not exist on the component instance. Did you mean 'count'?"
                    .into()
            ))
        );
    }

    #[test]
    fn script_names_and_other_codes_are_left_alone() {
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
                Some(2322),
                &at(SfcBlockType::Template),
                "Type 'string' is not assignable to type 'number'."
            ),
            None
        );
    }
}
