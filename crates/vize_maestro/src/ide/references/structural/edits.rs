//! Preserve object keys and public import/export names when renaming a local.

use oxc_ast::AstKind;
use oxc_semantic::Semantic;
use oxc_syntax::node::NodeId;

#[derive(Clone, Copy)]
pub(super) enum Kind {
    Identifier,
    ObjectShorthand,
    ImportShorthand,
    ExportShorthand,
}

pub(super) fn kind(semantic: &Semantic<'_>, node: NodeId) -> Kind {
    let nodes = semantic.nodes();
    let parent = nodes.parent_kind(node);
    let parent = if matches!(parent, AstKind::AssignmentPattern(_)) {
        nodes.parent_kind(nodes.parent_id(node))
    } else {
        parent
    };
    match parent {
        AstKind::ObjectProperty(property) if property.shorthand => Kind::ObjectShorthand,
        AstKind::BindingProperty(property) if property.shorthand => Kind::ObjectShorthand,
        AstKind::ImportSpecifier(specifier)
            if specifier.imported.name() == specifier.local.name =>
        {
            Kind::ImportShorthand
        }
        AstKind::ExportSpecifier(specifier)
            if specifier.local.name() == specifier.exported.name() =>
        {
            Kind::ExportShorthand
        }
        _ => Kind::Identifier,
    }
}

pub(super) fn render(kind: Kind, original: &str, new: &str) -> std::string::String {
    match kind {
        Kind::Identifier => new.into(),
        Kind::ObjectShorthand => vize_s0::cstr!("{original}: {new}").into(),
        Kind::ImportShorthand => vize_s0::cstr!("{original} as {new}").into(),
        Kind::ExportShorthand => vize_s0::cstr!("{new} as {original}").into(),
    }
}
