//! The value a `components` registration names.

use oxc_ast::ast::Expression;
use vize_carton::CompactString;

/// The local binding, or a static member path rooted at one, that the
/// registration value references through parentheses and TS wrappers.
pub(super) fn local_name(expression: &Expression<'_>) -> Option<CompactString> {
    let mut path = CompactString::default();
    push_path(expression, &mut path).then_some(path)
}

fn push_path(expression: &Expression<'_>, path: &mut CompactString) -> bool {
    match expression {
        Expression::Identifier(identifier) => {
            path.push_str(identifier.name.as_str());
            true
        }
        Expression::StaticMemberExpression(member) if !member.optional => {
            if !push_path(&member.object, path) {
                return false;
            }
            path.push('.');
            path.push_str(member.property.name.as_str());
            true
        }
        Expression::ParenthesizedExpression(value) => push_path(&value.expression, path),
        Expression::TSAsExpression(value) => push_path(&value.expression, path),
        Expression::TSSatisfiesExpression(value) => push_path(&value.expression, path),
        Expression::TSNonNullExpression(value) => push_path(&value.expression, path),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn records_static_member_paths_but_not_computed_values() {
        let result = crate::script_parser::parse_script(
            r#"
import { defineComponent } from 'vue'
import * as ui from './ui'
import Menu from './Menu.vue'
export default defineComponent({
  components: {
    Wrapped: (Menu as any)!,
    Member: ui.forms.Menu,
    Optional: ui?.Menu,
    Computed: ui['Menu'],
    Called: ui.load(),
  },
})
"#,
        );
        let registrations: Vec<_> = result
            .component_registrations
            .iter()
            .map(|registration| (registration.name.as_str(), registration.local_name.as_str()))
            .collect();
        assert_eq!(
            registrations,
            [("Wrapped", "Menu"), ("Member", "ui.forms.Menu")]
        );
    }
}
