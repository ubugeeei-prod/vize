use oxc_ast::ast::VariableDeclarationKind;
use vize_carton::{CompactString, String, cstr};
use vize_relief::BindingType;

pub(in crate::script_parser::extract) fn component_name_from_source(source: &str) -> CompactString {
    let without_query = source.split(['?', '#']).next().unwrap_or(source);
    let filename = without_query
        .rsplit(['/', '\\'])
        .next()
        .filter(|name| !name.is_empty())
        .unwrap_or("Component");
    let stem = filename
        .rsplit_once('.')
        .map_or(filename, |(name, _extension)| name);

    let mut component_name = String::default();
    for segment in stem.split(|ch: char| !(ch.is_ascii_alphanumeric())) {
        if segment.is_empty() {
            continue;
        }

        let mut chars = segment.chars();
        if let Some(first) = chars.next() {
            component_name.push(first.to_ascii_uppercase());
            component_name.extend(chars);
        }
    }

    if component_name.is_empty() {
        cstr!("Component")
    } else {
        CompactString::new(component_name)
    }
}

pub fn get_binding_type_from_kind(kind: VariableDeclarationKind) -> BindingType {
    match kind {
        VariableDeclarationKind::Const => BindingType::SetupConst,
        VariableDeclarationKind::Let => BindingType::SetupLet,
        VariableDeclarationKind::Var => BindingType::SetupLet,
        VariableDeclarationKind::Using => BindingType::SetupConst,
        VariableDeclarationKind::AwaitUsing => BindingType::SetupConst,
    }
}
