use oxc_ast::ast::{BindingPattern, PropertyKey, VariableDeclarator};

pub(super) fn is_template_hoist_declarator(declarator: &VariableDeclarator<'_>) -> bool {
    let BindingPattern::BindingIdentifier(id) = &declarator.id else {
        return false;
    };

    let name = id.name.as_str();
    name.starts_with("_hoisted_")
        && name["_hoisted_".len()..]
            .bytes()
            .all(|byte| byte.is_ascii_digit())
}

pub(super) fn is_render_function_name(name: &str) -> bool {
    matches!(name, "render" | "_sfc_render" | "ssrRender")
}

pub(super) fn property_key_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}
