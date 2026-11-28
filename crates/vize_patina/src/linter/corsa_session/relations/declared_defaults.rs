//! `no-unnecessary-type-arguments` for generics the script does not declare
//! (imported, global or library ones). typescript-eslint's
//! `getTypeParametersFromNode` picks the declaration through the checker (the
//! resolved signature of a call, else the declarations of the aliased symbol
//! of the name) and compares `checker.getTypeAtLocation(param.default)` with
//! the argument. The declaration and its type parameters are read from the
//! checker's own syntax tree of the declaring file.
use super::checker::{CONSTRUCT, Checker};
use super::{Located, Span};
use crate::linter::corsa_session::boolean::{CheckerSyntax, syntax_kind};
use corsa::api::{NodeHandle, SymbolResponse, TypeResponse};
use vize_l0::FxHashMap;

/// `SymbolFlags.Alias`.
const SYMBOL_ALIAS: u32 = 1 << 21;
/// `TypeParameterDeclaration.defaultType`, in the encoder's visitor order
/// (modifiers, name, constraint, expression, default).
const DEFAULT_TYPE: u32 = 4;

/// One explicit type argument list whose generic is declared elsewhere.
pub(in crate::linter) struct DeclaredDefaultAsk {
    /// The last type argument, the only one compared.
    pub argument: Span,
    pub index: usize,
    /// The instantiated name; calls that are not `new` read only their
    /// resolved signature.
    pub owner: Option<Span>,
    /// The call, `new` or tagged template the list belongs to.
    pub call: Option<Span>,
    /// A type reference, `implements` or interface `extends` clause
    /// (`isInTypeContext`).
    pub type_context: bool,
}

/// The facts `checkTSArgsAndParameters` reads for one list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(in crate::linter) struct DeclaredDefault {
    pub parameter_count: usize,
    pub has_default: bool,
    /// The argument and the default are the same checker type.
    pub same: bool,
}

impl DeclaredDefaultAsk {
    pub(super) fn each_span(&self, add: &mut dyn FnMut(Span, bool)) {
        add(self.argument, true);
        if let Some(owner) = self.owner {
            add(owner, false);
        }
        if let Some(call) = self.call {
            add(call, false);
        }
    }
}

/// Default node handles of a declaration's type parameters, `None` for a
/// parameter without one.
type Parameters = Vec<Option<NodeHandle>>;

#[derive(Default)]
struct Memo {
    parameters: FxHashMap<NodeHandle, Option<Parameters>>,
    defaults: FxHashMap<NodeHandle, Option<TypeResponse>>,
}

pub(super) async fn answer(
    checker: &mut Checker<'_>,
    located: &Located,
    asks: &[DeclaredDefaultAsk],
    translate: &dyn Fn(Span) -> Span,
) -> Vec<Option<DeclaredDefault>> {
    let mut memo = Memo::default();
    let mut replies = Vec::with_capacity(asks.len());
    for ask in asks {
        replies.push(check(checker, located, ask, translate, &mut memo).await);
    }
    replies
}

async fn check(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &DeclaredDefaultAsk,
    translate: &dyn Fn(Span) -> Span,
    memo: &mut Memo,
) -> Option<DeclaredDefault> {
    let argument = located.type_node(translate(ask.argument))?.clone();
    let parameters = owned_parameters(checker, located, ask, translate, memo).await?;
    let parameter_count = parameters.len();
    let Some(Some(default)) = parameters.get(ask.index) else {
        return Some(DeclaredDefault {
            parameter_count,
            ..DeclaredDefault::default()
        });
    };
    let default = default_type(checker, default, memo).await?;
    let same = checker.same_type(&argument, &default).await;
    Some(DeclaredDefault {
        parameter_count,
        has_default: true,
        same,
    })
}

/// `getTypeParametersFromNode`.
async fn owned_parameters(
    checker: &mut Checker<'_>,
    located: &Located,
    ask: &DeclaredDefaultAsk,
    translate: &dyn Fn(Span) -> Span,
    memo: &mut Memo,
) -> Option<Parameters> {
    if let Some(call) = ask.call {
        // `getTypeParametersFromCall`: the resolved signature's declaration,
        // or for `new` without one the constructed name.
        let call = located.handle(translate(call))?;
        if let Some(declaration) = checker
            .resolved_signature(call)
            .await
            .and_then(|signature| signature.declaration)
        {
            return parameters(checker, &declaration, memo).await;
        }
    }
    let owner = located.handle(translate(ask.owner?))?.clone();
    let symbol = checker.symbols_at(vec![owner]).await.pop().flatten()?;
    let target = if symbol.flags & SYMBOL_ALIAS != 0 {
        checker.aliased_symbol(&symbol).await?
    } else {
        symbol.clone()
    };
    // `sortDeclarationsByTypeValueContext`: a stable type-first sort,
    // reversed outside type contexts.
    let mut declarations = target
        .declarations
        .iter()
        .map(|declaration| (declaration, handle_kind(declaration)))
        .collect::<Vec<_>>();
    declarations.sort_by_key(|(_, kind)| !is_type_declaration(*kind));
    if !ask.type_context {
        declarations.reverse();
    }
    for (declaration, kind) in declarations {
        let found = match kind {
            Some(
                syntax_kind::TYPE_ALIAS_DECLARATION
                | syntax_kind::INTERFACE_DECLARATION
                | syntax_kind::CLASS_DECLARATION
                | syntax_kind::CLASS_EXPRESSION,
            ) => parameters(checker, declaration, memo).await,
            Some(syntax_kind::VARIABLE_DECLARATION) => {
                construct_parameters(checker, &symbol, memo).await
            }
            _ => None,
        };
        if found.is_some() {
            return found;
        }
    }
    None
}

fn is_type_declaration(kind: Option<u32>) -> bool {
    matches!(
        kind,
        Some(syntax_kind::TYPE_ALIAS_DECLARATION | syntax_kind::INTERFACE_DECLARATION)
    )
}

/// The syntax kind a declaration handle carries: `<pos>.<end>.<kind>.<path>`
/// or the compact `<index>.<kind>.<path>`.
fn handle_kind(handle: &NodeHandle) -> Option<u32> {
    if let Ok(parsed) = handle.parse() {
        return Some(u32::from(parsed.kind));
    }
    handle.as_str().split('.').nth(1)?.parse().ok()
}

/// `getConstructSignatureDeclaration`: the first construct signature of the
/// type of the name's own symbol.
async fn construct_parameters(
    checker: &mut Checker<'_>,
    symbol: &SymbolResponse,
    memo: &mut Memo,
) -> Option<Parameters> {
    let ty = checker.type_of_symbol(symbol).await?;
    let signature = checker
        .signatures(&ty, CONSTRUCT)
        .await
        .into_iter()
        .next()?;
    parameters(checker, signature.declaration.as_ref()?, memo).await
}

/// `declaration.typeParameters`, `None` when it declares none.
async fn parameters(
    checker: &mut Checker<'_>,
    declaration: &NodeHandle,
    memo: &mut Memo,
) -> Option<Parameters> {
    if let Some(known) = memo.parameters.get(declaration) {
        return known.clone();
    }
    let found = read_parameters(checker, declaration).await;
    memo.parameters.insert(declaration.clone(), found.clone());
    found
}

async fn read_parameters(
    checker: &mut Checker<'_>,
    declaration: &NodeHandle,
) -> Option<Parameters> {
    let path = declaration.declaring_path()?;
    let encoded = checker.source_file(path.as_str()).await?;
    let syntax = CheckerSyntax::read(encoded.as_bytes()).ok()?;
    let index = syntax.index_of(declaration)?;
    // The `typeParameters` property bit of each declaration kind.
    let bit = match syntax.kind(index)? {
        syntax_kind::CALL_SIGNATURE
        | syntax_kind::CONSTRUCT_SIGNATURE
        | syntax_kind::FUNCTION_TYPE => 0,
        syntax_kind::ARROW_FUNCTION | syntax_kind::CONSTRUCTOR | syntax_kind::CONSTRUCTOR_TYPE => 1,
        syntax_kind::TYPE_ALIAS_DECLARATION
        | syntax_kind::INTERFACE_DECLARATION
        | syntax_kind::CLASS_DECLARATION
        | syntax_kind::CLASS_EXPRESSION
        | syntax_kind::GET_ACCESSOR
        | syntax_kind::SET_ACCESSOR => 2,
        syntax_kind::FUNCTION_DECLARATION
        | syntax_kind::FUNCTION_EXPRESSION
        | syntax_kind::METHOD_SIGNATURE => 3,
        syntax_kind::METHOD_DECLARATION => 4,
        _ => return None,
    };
    let list = syntax.property(index, bit)?;
    Some(
        syntax
            .elements(list)
            .map(|parameter| {
                (syntax.kind(parameter) == Some(syntax_kind::TYPE_PARAMETER))
                    .then(|| syntax.property(parameter, DEFAULT_TYPE))
                    .flatten()
                    .and_then(|default| syntax.handle(default))
            })
            .collect(),
    )
}

/// `checker.getTypeAtLocation(param.default)`, once per default node.
async fn default_type(
    checker: &mut Checker<'_>,
    default: &NodeHandle,
    memo: &mut Memo,
) -> Option<TypeResponse> {
    if let Some(known) = memo.defaults.get(default) {
        return known.clone();
    }
    let session = checker.session();
    let found = session
        .client()
        .get_type_at_location(
            session.snapshot().handle.clone(),
            session.project_handle(),
            default.clone(),
        )
        .await
        .ok()
        .flatten();
    memo.defaults.insert(default.clone(), found.clone());
    found
}
