//! `no-unnecessary-type-arguments` for generics the script does not declare
//! itself (imports, libraries, members): typescript-eslint's
//! `getTypeParametersFromNode` and `checker.getTypeAtLocation(param.default)`,
//! with the type parameter list read from the declaration's own syntax in the
//! checker's encoded source file.
use super::checker::Checker;
use super::{Located, Span};
use crate::linter::corsa_session::boolean::{CheckerSyntax, syntax_kind};
use corsa::api::{NodeHandle, SymbolResponse, TypeResponse};

/// Where typescript-eslint finds the type parameters of one instantiation.
pub(in crate::linter) enum DefaultSite {
    /// `getTypeParametersFromType(name)`: a type reference, heritage type or
    /// class `extends` expression. `type_context` sorts type aliases and
    /// interfaces first (`isInTypeContext`).
    Type { name: Span, type_context: bool },
    /// `getResolvedSignature(call).getDeclaration()`.
    Call { call: Span },
    /// A `new` expression: its resolved signature's declaration, otherwise
    /// the callee as a value-context type.
    New { call: Span, callee: Span },
}

/// One explicit type argument list whose generic the script does not
/// declare: whether its last argument (`argument`, the `index`th) equals
/// that type parameter's default.
pub(in crate::linter) struct DeclaredDefaultAsk {
    pub argument: Span,
    pub index: usize,
    pub site: DefaultSite,
}

impl DeclaredDefaultAsk {
    /// Every node span the check reads, and whether it is a type node.
    pub(in crate::linter) fn each_span(&self, add: &mut impl FnMut(Span, bool)) {
        add(self.argument, true);
        match self.site {
            DefaultSite::Type { name, .. } => add(name, false),
            DefaultSite::Call { call } => add(call, false),
            DefaultSite::New { call, callee } => {
                add(call, false);
                add(callee, false);
            }
        }
    }
}

/// `SymbolFlags.Alias`.
const ALIAS: u32 = 1 << 21;

pub(super) async fn check(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &DeclaredDefaultAsk,
    translate: &dyn Fn(Span) -> Span,
) -> Option<bool> {
    let argument = located.type_node(translate(ask.argument))?.clone();
    let parameter = match &ask.site {
        DefaultSite::Type { name, type_context } => {
            let name = located.handle(translate(*name))?;
            from_type(checker, name, *type_context, ask.index).await
        }
        DefaultSite::Call { call } => {
            let call = located.handle(translate(*call))?;
            let declaration = checker.resolved_signature(call).await?.declaration?;
            declared_parameter(checker, &declaration, ask.index).await
        }
        DefaultSite::New { call, callee } => {
            let call = located.handle(translate(*call))?;
            match checker
                .resolved_signature(call)
                .await
                .and_then(|signature| signature.declaration)
            {
                Some(declaration) => declared_parameter(checker, &declaration, ask.index).await,
                None => {
                    let callee = located.handle(translate(*callee))?;
                    from_type(checker, callee, false, ask.index).await
                }
            }
        }
    }??;
    let default = type_at(checker, parameter).await?;
    Some(checker.equals_default(&argument, &default).await)
}

/// typescript-eslint `getTypeParametersFromType`: the first declaration of
/// the (aliased) symbol that declares type parameters, type aliases and
/// interfaces first in a type context and last in a value context; a
/// variable answers through its type's first construct signature. Returns
/// the default node of the `index`th parameter, `Some(None)` when the chosen
/// declaration has no default there.
async fn from_type(
    checker: &mut Checker<'_>,
    name: &NodeHandle,
    type_context: bool,
    index: usize,
) -> Option<Option<NodeHandle>> {
    let at_location = checker
        .symbols_at(vec![name.clone()])
        .await
        .into_iter()
        .next()
        .flatten()?;
    let symbol = if at_location.flags & ALIAS != 0 {
        checker.aliased_symbol(&at_location).await?
    } else {
        at_location.clone()
    };
    let mut declarations = symbol
        .declarations
        .iter()
        .map(|declaration| (handle_kind(declaration), declaration))
        .collect::<Vec<_>>();
    // `sortDeclarationsByTypeValueContext`: a stable type-first sort,
    // reversed outside type contexts.
    declarations.sort_by_key(|(kind, _)| !is_type_declaration(*kind));
    if !type_context {
        declarations.reverse();
    }
    for (kind, declaration) in declarations {
        match kind {
            Some(
                syntax_kind::TYPE_ALIAS_DECLARATION
                | syntax_kind::INTERFACE_DECLARATION
                | syntax_kind::CLASS_DECLARATION
                | syntax_kind::CLASS_EXPRESSION,
            ) => {
                if let Some(parameter) = declared_parameter(checker, declaration, index).await {
                    return Some(parameter);
                }
            }
            Some(syntax_kind::VARIABLE_DECLARATION) => {
                if let Some(parameter) = construct_parameter(checker, &at_location, index).await {
                    return Some(parameter);
                }
            }
            _ => {}
        }
    }
    None
}

/// `getConstructSignatureDeclaration(symbol)?.typeParameters`.
async fn construct_parameter(
    checker: &mut Checker<'_>,
    symbol: &SymbolResponse,
    index: usize,
) -> Option<Option<NodeHandle>> {
    let ty = checker.type_of_symbol(symbol).await?;
    let signature = checker
        .signatures(&ty, super::checker::CONSTRUCT)
        .await
        .into_iter()
        .next()?;
    declared_parameter(checker, signature.declaration.as_ref()?, index).await
}

/// `declaration.typeParameters`, then the default of its `index`th entry:
/// `None` when the declaration has no type parameter list (upstream moves on
/// to the next declaration), `Some(None)` when that parameter has no default.
async fn declared_parameter(
    checker: &mut Checker<'_>,
    declaration: &NodeHandle,
    index: usize,
) -> Option<Option<NodeHandle>> {
    let path = declaration.declaring_path()?;
    let encoded = checker.source_file(path.as_str()).await?;
    let syntax = CheckerSyntax::read(encoded.as_bytes()).ok()?;
    let node = syntax.index_of(declaration)?;
    let list = syntax.property(node, type_parameters_bit(syntax.kind(node)?)?)?;
    let Some(parameter) = syntax.elements(list).get(index).copied() else {
        // More arguments than parameters: `typeParameters.at(i)` is
        // undefined and the check ends there.
        return Some(None);
    };
    if syntax.kind(parameter)? != syntax_kind::TYPE_PARAMETER {
        return Some(None);
    }
    Some(
        syntax
            .property(parameter, 4)
            .and_then(|default| syntax.handle(default)),
    )
}

/// The `typeParameters` property of each declaration kind, in the encoder's
/// visitor order.
fn type_parameters_bit(kind: u32) -> Option<u32> {
    Some(match kind {
        syntax_kind::CALL_SIGNATURE
        | syntax_kind::CONSTRUCT_SIGNATURE
        | syntax_kind::FUNCTION_TYPE => 0,
        syntax_kind::CONSTRUCTOR | syntax_kind::ARROW_FUNCTION | syntax_kind::CONSTRUCTOR_TYPE => 1,
        syntax_kind::CLASS_DECLARATION
        | syntax_kind::CLASS_EXPRESSION
        | syntax_kind::INTERFACE_DECLARATION
        | syntax_kind::TYPE_ALIAS_DECLARATION
        | syntax_kind::GET_ACCESSOR
        | syntax_kind::SET_ACCESSOR => 2,
        syntax_kind::FUNCTION_DECLARATION
        | syntax_kind::FUNCTION_EXPRESSION
        | syntax_kind::METHOD_SIGNATURE => 3,
        syntax_kind::METHOD_DECLARATION => 4,
        _ => return None,
    })
}

fn is_type_declaration(kind: Option<u32>) -> bool {
    matches!(
        kind,
        Some(syntax_kind::TYPE_ALIAS_DECLARATION | syntax_kind::INTERFACE_DECLARATION)
    )
}

/// The syntax kind a declaration handle carries, in either wire form.
fn handle_kind(handle: &NodeHandle) -> Option<u32> {
    if let Ok(parsed) = handle.parse() {
        return Some(u32::from(parsed.kind));
    }
    handle.as_str().split('.').nth(1)?.parse().ok()
}

/// `checker.getTypeAtLocation(node)` for a node of a declaring file.
async fn type_at(checker: &Checker<'_>, node: NodeHandle) -> Option<TypeResponse> {
    checker
        .session()
        .client()
        .get_type_at_location(
            checker.session().snapshot().handle.clone(),
            checker.session().project_handle(),
            node,
        )
        .await
        .ok()
        .flatten()
}
