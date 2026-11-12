//! `isPossiblyUsedBeforeAssigned` from typescript-eslint's
//! `no-unnecessary-type-assertion`, resolved with oxc's scope analysis.
use oxc_allocator::Allocator;
use oxc_ast::{
    AstKind,
    ast::{BindingPattern, ClassElement, MethodDefinitionKind, VariableDeclarationKind},
};
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType};

/// Default spans of a declaration's type parameters, `None` without a default.
pub(super) type Defaults = Vec<Option<(u32, u32)>>;

/// What the declaration of a referenced identifier says about its use.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Binding {
    /// Not a plain variable declaration, or declared in another file.
    Assigned,
    /// A `var` used outside the scope it is declared in.
    MaybeUnassigned,
    /// `let x: T;` with no initializer or definite assignment: unassigned
    /// unless the checker narrowed the type at the use since the declaration.
    /// `name` is the declared binding.
    Annotated { name: (u32, u32) },
}

/// Resolve each identifier reference span in `source`.
pub(super) fn resolve(source: &str, tsx: bool, references: &[(u32, u32)]) -> Vec<Binding> {
    if references.is_empty() {
        return Vec::new();
    }
    let allocator = Allocator::default();
    let source_type = if tsx {
        SourceType::tsx()
    } else {
        SourceType::ts()
    };
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked {
        return vec![Binding::Assigned; references.len()];
    }
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program)
        .semantic;
    let nodes = semantic.nodes();
    let scoping = semantic.scoping();
    references
        .iter()
        .map(|&(start, end)| {
            let Some(use_node) = nodes.iter().find(|node| {
                matches!(node.kind(), AstKind::IdentifierReference(identifier)
                    if identifier.span.start == start && identifier.span.end == end)
            }) else {
                return Binding::Assigned;
            };
            let AstKind::IdentifierReference(identifier) = use_node.kind() else {
                return Binding::Assigned;
            };
            let Some(symbol) = identifier
                .reference_id
                .get()
                .and_then(|reference| scoping.get_reference(reference).symbol_id())
            else {
                return Binding::Assigned;
            };
            let declaration = semantic.symbol_declaration(symbol);
            let AstKind::VariableDeclarator(declarator) = declaration.kind() else {
                return Binding::Assigned;
            };
            if !matches!(declarator.id, BindingPattern::BindingIdentifier(_)) {
                return Binding::Assigned;
            }
            if declarator.kind == VariableDeclarationKind::Var {
                let use_scope = use_node.scope_id();
                let mut scope = scoping.scope_parent_id(declaration.scope_id());
                while let Some(current) = scope {
                    if current == use_scope {
                        return Binding::MaybeUnassigned;
                    }
                    scope = scoping.scope_parent_id(current);
                }
            }
            let declared = matches!(
                nodes.parent_kind(declaration.id()),
                AstKind::VariableDeclaration(statement) if statement.declare
            );
            match (&declarator.id, &declarator.type_annotation) {
                (BindingPattern::BindingIdentifier(name), Some(_))
                    if declarator.init.is_none() && !declarator.definite && !declared =>
                {
                    Binding::Annotated {
                        name: (name.span.start, name.span.end),
                    }
                }
                _ => Binding::Assigned,
            }
        })
        .collect()
}

/// Type parameter default spans (`None` without a default) of the one local
/// declaration each referenced name resolves to: a type alias, interface,
/// class or function. Merged or overloaded declarations, and classes with an
/// explicit constructor called through `new` (whose resolved signature owns no
/// type parameters), resolve to `None`.
pub(super) fn type_parameter_defaults(
    source: &str,
    tsx: bool,
    names: &[((u32, u32), bool)],
) -> Vec<Option<Defaults>> {
    if names.is_empty() {
        return Vec::new();
    }
    let allocator = Allocator::default();
    let source_type = if tsx {
        SourceType::tsx()
    } else {
        SourceType::ts()
    };
    let parsed = Parser::new(&allocator, source, source_type).parse();
    if parsed.panicked {
        return vec![None; names.len()];
    }
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(&parsed.program)
        .semantic;
    let nodes = semantic.nodes();
    let scoping = semantic.scoping();
    names
        .iter()
        .map(|&((start, end), constructed)| {
            let symbol = nodes.iter().find_map(|node| match node.kind() {
                AstKind::IdentifierReference(identifier)
                    if identifier.span.start == start && identifier.span.end == end =>
                {
                    identifier
                        .reference_id
                        .get()
                        .and_then(|reference| scoping.get_reference(reference).symbol_id())
                }
                _ => None,
            })?;
            if !scoping.symbol_redeclarations(symbol).is_empty() {
                return None;
            }
            let parameters = match semantic.symbol_declaration(symbol).kind() {
                AstKind::TSTypeAliasDeclaration(alias) => alias.type_parameters.as_deref(),
                AstKind::TSInterfaceDeclaration(interface) => interface.type_parameters.as_deref(),
                AstKind::Class(class) => {
                    let has_constructor = class.body.body.iter().any(|element| {
                        matches!(element, ClassElement::MethodDefinition(method)
                            if method.kind == MethodDefinitionKind::Constructor)
                    });
                    if constructed && has_constructor {
                        return None;
                    }
                    class.type_parameters.as_deref()
                }
                AstKind::Function(function) => function.type_parameters.as_deref(),
                _ => return None,
            }?;
            Some(
                parameters
                    .params
                    .iter()
                    .map(|parameter| {
                        parameter.default.as_ref().map(|default| {
                            let span = default.span();
                            (span.start, span.end)
                        })
                    })
                    .collect(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Binding, resolve};

    fn at(source: &str, needle: &str) -> (u32, u32) {
        let start = source.rfind(needle).unwrap() as u32;
        (start, start + needle.len() as u32)
    }

    #[test]
    fn a_var_read_outside_its_block_may_be_unassigned() {
        let source = "if (Math.random()) {\n  {\n    var x = 1;\n  }\n}\nx!;";
        assert_eq!(
            resolve(source, false, &[at(source, "x")]),
            [Binding::MaybeUnassigned]
        );
    }

    #[test]
    fn an_uninitialized_annotated_let_reports_its_name() {
        let source = "let bar: number;\nbar! + 1;";
        let Binding::Annotated { name } = resolve(source, false, &[at(source, "bar")])[0] else {
            panic!("expected an annotated binding");
        };
        assert_eq!(name, (4, 7));
    }

    #[test]
    fn declared_initialized_and_parameter_bindings_are_assigned() {
        let source = "declare const a: number;\nconst b: number = 1;\nfunction f(c: number) { c!; }\na!; b!;";
        let found = resolve(
            source,
            false,
            &[at(source, "c!"), at(source, "a!"), at(source, "b!")]
                .map(|(start, end)| (start, end - 1)),
        );
        assert_eq!(found, [Binding::Assigned; 3]);
    }

    #[test]
    fn defaults_come_from_the_single_local_declaration() {
        let source =
            "class C<T = number> {}\nfunction h(c: C<number>) {}\ntype B = Map<string, string>;";
        let found = super::type_parameter_defaults(
            source,
            false,
            &[
                (
                    (at(source, "C<number>").0, at(source, "C<number>").0 + 1),
                    false,
                ),
                (at(source, "Map"), false),
            ],
        );
        let defaults = found[0].as_ref().unwrap();
        let default = defaults[0].unwrap();
        assert_eq!(
            source.get(default.0 as usize..default.1 as usize),
            Some("number")
        );
        assert_eq!(found[1], None);
    }
}
