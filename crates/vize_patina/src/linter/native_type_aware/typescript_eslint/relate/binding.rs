//! `isPossiblyUsedBeforeAssigned` from typescript-eslint's
//! `no-unnecessary-type-assertion`, resolved with oxc's scope analysis.
use oxc_ast::{
    AstKind,
    ast::{
        BindingPattern, ClassElement, MethodDefinitionKind, Program, TSSignature, TSType,
        TSTypeAnnotation, TSTypeParameterDeclaration, VariableDeclarationKind,
    },
};
use oxc_semantic::SemanticBuilder;
use oxc_span::GetSpan;

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

/// Resolve each identifier reference span in the script `program` (the
/// listener tree's own parse).
pub(super) fn resolve(program: &Program<'_>, references: &[(u32, u32)]) -> Vec<Binding> {
    if references.is_empty() {
        return Vec::new();
    }
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(program)
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

/// Where a type argument list instantiates its name, which decides the
/// declaration typescript-eslint reads the type parameters from.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum Instantiated {
    /// A type reference, `implements` or interface `extends` clause.
    Type,
    /// A class `extends` clause.
    Value,
    /// A call or tagged template.
    Call,
    /// A `new` expression.
    New,
}

/// Type parameter default spans (`None` without a default) of the local
/// declaration each referenced name resolves to, chosen the way
/// typescript-eslint's `getTypeParametersFromNode` does: among a merged
/// symbol's declarations, type aliases and interfaces answer first in a type
/// context and last in a value context; a `declare var` answers through the
/// first construct signature of its type literal. Calls read the single
/// function declaration their signature resolves to. Overloads, classes with
/// an explicit constructor called through `new` (whose resolved signature owns
/// no type parameters) and declarations whose parameters need the checker
/// resolve to `None`.
pub(super) fn type_parameter_defaults(
    program: &Program<'_>,
    names: &[((u32, u32), Instantiated)],
) -> Vec<Option<Defaults>> {
    if names.is_empty() {
        return Vec::new();
    }
    let semantic = SemanticBuilder::new()
        .with_build_nodes(true)
        .build(program)
        .semantic;
    let nodes = semantic.nodes();
    let scoping = semantic.scoping();
    names
        .iter()
        .map(|&((start, end), context)| {
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
            let redeclarations = scoping.symbol_redeclarations(symbol);
            let mut declarations = if redeclarations.is_empty() {
                vec![semantic.symbol_declaration(symbol).kind()]
            } else {
                redeclarations
                    .iter()
                    .map(|redeclaration| nodes.kind(redeclaration.declaration))
                    .collect()
            };
            let parameters = match context {
                Instantiated::Call => call_parameters(&declarations)?,
                Instantiated::New if declarations.iter().any(has_constructor) => return None,
                Instantiated::Type | Instantiated::Value | Instantiated::New => {
                    // `sortDeclarationsByTypeValueContext`: a stable type-first
                    // sort, reversed outside type contexts.
                    declarations.sort_by_key(|kind| !is_type_declaration(kind));
                    if context != Instantiated::Type {
                        declarations.reverse();
                    }
                    declared_parameters(&declarations)?
                }
            };
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

type Parameters<'a> = &'a TSTypeParameterDeclaration<'a>;

fn is_type_declaration(kind: &AstKind<'_>) -> bool {
    matches!(
        kind,
        AstKind::TSTypeAliasDeclaration(_) | AstKind::TSInterfaceDeclaration(_)
    )
}

fn has_constructor(kind: &AstKind<'_>) -> bool {
    matches!(kind, AstKind::Class(class) if class.body.body.iter().any(|element| {
        matches!(element, ClassElement::MethodDefinition(method)
            if method.kind == MethodDefinitionKind::Constructor)
    }))
}

/// The resolved signature of a call is its one function declaration.
fn call_parameters<'a>(declarations: &[AstKind<'a>]) -> Option<Parameters<'a>> {
    let mut functions = declarations.iter().filter_map(|kind| match kind {
        AstKind::Function(function) => Some(function),
        _ => None,
    });
    let function = functions.next()?;
    if functions.next().is_some() {
        return None;
    }
    function.type_parameters.as_deref()
}

/// `findFirstResult` over the sorted declarations: the first one that owns
/// type parameters answers. A variable whose type the checker would have to
/// resolve stops the search unanswered.
fn declared_parameters<'a>(declarations: &[AstKind<'a>]) -> Option<Parameters<'a>> {
    for kind in declarations {
        let found = match kind {
            AstKind::TSTypeAliasDeclaration(alias) => alias.type_parameters.as_deref(),
            AstKind::TSInterfaceDeclaration(interface) => interface.type_parameters.as_deref(),
            AstKind::Class(class) => class.type_parameters.as_deref(),
            AstKind::VariableDeclarator(declarator) => {
                construct_parameters(declarator.type_annotation.as_deref()?)?
            }
            _ => None,
        };
        if found.is_some() {
            return found;
        }
    }
    None
}

/// The type parameters of the first construct signature of a variable
/// annotated with a type literal; `Some(None)` when the literal has no
/// construct signature with type parameters, `None` when the annotation is not
/// a literal and only the checker could tell.
fn construct_parameters<'a>(
    annotation: &'a TSTypeAnnotation<'a>,
) -> Option<Option<Parameters<'a>>> {
    let TSType::TSTypeLiteral(literal) = &annotation.type_annotation else {
        return None;
    };
    let first = literal.members.iter().find_map(|member| match member {
        TSSignature::TSConstructSignatureDeclaration(signature) => {
            Some(signature.type_parameters.as_deref())
        }
        _ => None,
    });
    Some(first.flatten())
}

#[cfg(test)]
mod tests {
    use super::super::super::tree;
    use super::{Binding, Defaults, Instantiated};

    fn resolve(source: &str, tsx: bool, references: &[(u32, u32)]) -> Vec<Binding> {
        tree::collect_with(source, tsx, &[], |_, ast| super::resolve(ast, references)).1
    }

    fn type_parameter_defaults(
        source: &str,
        tsx: bool,
        names: &[((u32, u32), Instantiated)],
    ) -> Vec<Option<Defaults>> {
        tree::collect_with(source, tsx, &[], |_, ast| {
            super::type_parameter_defaults(ast, names)
        })
        .1
    }

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
        let found = type_parameter_defaults(
            source,
            false,
            &[
                (
                    (at(source, "C<number>").0, at(source, "C<number>").0 + 1),
                    Instantiated::Type,
                ),
                (at(source, "Map"), Instantiated::Type),
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

    /// The default text the reference at the last `needle` reads, if any.
    fn last_default(source: &str, needle: &str, context: Instantiated) -> Option<String> {
        let start = at(source, needle).0;
        let name = needle.split('<').next().unwrap_or(needle);
        let found = type_parameter_defaults(
            source,
            false,
            &[((start, start + name.len() as u32), context)],
        );
        let (start, end) = found.into_iter().next()??.last().copied()??;
        source
            .get(start as usize..end as usize)
            .map(ToString::to_string)
    }

    #[test]
    fn merged_declarations_answer_in_their_context_order() {
        // typescript-eslint v8.72.2 no-unnecessary-type-arguments invalid[17..20].
        let variable_first = "interface Foo {}\ndeclare var Foo: {\n  new <T = string>(type: T): any;\n};\nclass Bar extends Foo<string> {}";
        let interface_last = "declare var Foo: {\n  new <T = string>(type: T): any;\n};\ninterface Foo {}\nclass Bar extends Foo<string> {}";
        let implemented =
            "class Foo<T> {}\ninterface Foo<T = string> {}\nclass Bar implements Foo<string> {}";
        let namespaced = "class Foo<T = string> {}\nnamespace Foo {\n  export class Bar {}\n}\nclass Bar extends Foo<string> {}";
        for source in [variable_first, interface_last, namespaced] {
            assert_eq!(
                last_default(source, "Foo<string>", Instantiated::Value).as_deref(),
                Some("string"),
                "{source}"
            );
        }
        assert_eq!(
            last_default(implemented, "Foo<string>", Instantiated::Type).as_deref(),
            Some("string")
        );
        // In a value context the class answers first, and it has no default.
        let class_first =
            "class Foo<T> {}\ninterface Foo<T = string> {}\nclass Bar extends Foo<string> {}";
        assert_eq!(
            last_default(class_first, "Foo<string>", Instantiated::Value),
            None
        );
    }

    #[test]
    fn unresolved_variables_and_overloads_stay_unanswered() {
        let referenced = "type Ctor = { new <T = string>(type: T): any };\ninterface Foo {}\ndeclare var Foo: Ctor;\nclass Bar extends Foo<string> {}";
        assert_eq!(
            last_default(referenced, "Foo<string>", Instantiated::Value),
            None
        );
        // A literal without a construct signature passes the search on.
        let callable = "interface Foo<T = string> {}\ndeclare var Foo: { (): void };\nclass Bar extends Foo<string> {}";
        assert_eq!(
            last_default(callable, "Foo<string>", Instantiated::Value).as_deref(),
            Some("string")
        );
        let overloaded = "function f<T = string>(): T;\nfunction f<T = number>(): T;\nfunction f() { return 1 as any; }\nf<string>();";
        assert_eq!(
            last_default(overloaded, "f<string>", Instantiated::Call),
            None
        );
        let constructed = "class C<T = string> { constructor() {} }\nnew C<string>();";
        assert_eq!(
            last_default(constructed, "C<string>", Instantiated::New),
            None
        );
    }
}
