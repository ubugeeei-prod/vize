//! Class components (`vue-class-component`, `vue-property-decorator`): the
//! options object of an `@Component({..})` or `@Options({..})` decorator, and
//! the class's instance members. A method is a method, an accessor a
//! computed, and a field data unless a member decorator makes it a prop, a
//! computed or an injection: `@Prop` and `@Model` declare the field as the
//! prop itself, while `@PropSync`, `@ModelSync` and `@VModel` declare a prop
//! of another name and make the field the computed wrapper over it, which
//! the template reads through the instance.

use super::OptionMemberKind;
use super::read::{Authored, Entry, Reads, static_key, wrapped};
use alloc::vec::Vec;
use oxc_ast::ast::{
    Argument, Class as ClassSyntax, ClassElement, Decorator, ExportDefaultDeclarationKind,
    Expression, MethodDefinitionKind,
};
use vize_l0::Span;

/// A class component's reads.
#[derive(Debug)]
pub(in crate::framework::vue) struct Class {
    pub(super) span: Span,
    /// The decorators' options objects in the arena.
    pub(super) options: Vec<usize>,
    pub(super) members: Vec<(OptionMemberKind, Entry)>,
}

/// The class an `export default` declares, inside expression wrappers.
pub(super) fn exported<'e, 'a>(
    declaration: &'e ExportDefaultDeclarationKind<'a>,
) -> Option<&'e ClassSyntax<'a>> {
    match declaration {
        ExportDefaultDeclarationKind::ClassDeclaration(class) => Some(class),
        other => match wrapped(other.as_expression()?) {
            Expression::ClassExpression(class) => Some(class),
            _ => None,
        },
    }
}

pub(super) fn read<'a>(
    reads: &mut Reads<'a>,
    class: &ClassSyntax<'a>,
    authored: Authored<'_>,
) -> Class {
    let mut options = Vec::new();
    for decorator in &class.decorators {
        if let Expression::CallExpression(call) = &decorator.expression
            && matches!(&call.callee, Expression::Identifier(id) if matches!(id.name.as_str(), "Component" | "Options"))
            && let Some(Argument::ObjectExpression(object)) = call.arguments.first()
        {
            options.push(reads.object(object, true, authored));
        }
    }
    let mut members = Vec::new();
    for element in &class.body.body {
        let (kind, key) = match element {
            ClassElement::MethodDefinition(method)
                if !method.r#static && method.kind != MethodDefinitionKind::Constructor =>
            {
                let kind = match method.kind {
                    MethodDefinitionKind::Get | MethodDefinitionKind::Set => {
                        OptionMemberKind::Computed
                    }
                    _ => OptionMemberKind::Method,
                };
                (kind, static_key(&method.key, method.computed, authored))
            }
            ClassElement::PropertyDefinition(property)
                if !property.r#static && !property.declare =>
            {
                (
                    field(&property.decorators),
                    static_key(&property.key, property.computed, authored),
                )
            }
            ClassElement::AccessorProperty(accessor) if !accessor.r#static => (
                field(&accessor.decorators),
                static_key(&accessor.key, accessor.computed, authored),
            ),
            _ => continue,
        };
        if let Some((_, entry)) = key {
            members.push((kind, entry));
        }
    }
    Class {
        span: authored(class.span),
        options,
        members,
    }
}

/// A field's kind from its member decorators.
fn field(decorators: &[Decorator<'_>]) -> OptionMemberKind {
    let named = |names: &[&str]| {
        decorators.iter().any(|decorator| {
            let name = match &decorator.expression {
                Expression::Identifier(id) => Some(id.name.as_str()),
                Expression::CallExpression(call) => match &call.callee {
                    Expression::Identifier(id) => Some(id.name.as_str()),
                    _ => None,
                },
                _ => None,
            };
            name.is_some_and(|name| names.contains(&name))
        })
    };
    if named(&["Prop", "Model"]) {
        OptionMemberKind::Prop
    } else if named(&["PropSync", "ModelSync", "VModel"]) {
        OptionMemberKind::Computed
    } else if named(&["Inject", "InjectReactive"]) {
        OptionMemberKind::Inject
    } else {
        OptionMemberKind::Data
    }
}
