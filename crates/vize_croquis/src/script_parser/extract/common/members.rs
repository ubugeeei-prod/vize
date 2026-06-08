use oxc_ast::ast::Expression;
use vize_carton::CompactString;

/// Check for ref.value extraction to a plain variable (loses reactivity)
/// e.g., `const x = someRef.value` or `const primitiveValue = countRef.value`
pub(in crate::script_parser::extract) fn extract_member_chain_root(
    expr: &Expression<'_>,
) -> Option<(CompactString, CompactString)> {
    match expr {
        Expression::StaticMemberExpression(member) => {
            if let Some((root, prop_name)) = extract_member_chain_root(&member.object) {
                Some((root, prop_name))
            } else {
                let root = member_chain_root_identifier(&member.object)?;
                Some((root, CompactString::new(member.property.name.as_str())))
            }
        }
        _ => None,
    }
}

pub(in crate::script_parser::extract) fn member_chain_root_identifier(
    expr: &Expression<'_>,
) -> Option<CompactString> {
    match expr {
        Expression::Identifier(id) => Some(CompactString::new(id.name.as_str())),
        Expression::StaticMemberExpression(member) => member_chain_root_identifier(&member.object),
        _ => None,
    }
}
