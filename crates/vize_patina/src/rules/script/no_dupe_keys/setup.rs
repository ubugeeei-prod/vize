//! Prop collisions with actual setup-scope value declarations.
//!
//! The SFC dispatcher supplies its existing setup parse and an explicit sibling
//! packet. Unknown siblings do not prove macro ownership. Semantic references
//! distinguish prop-derived initializers from same-spelled nested locals.

mod props;

use oxc_ast::{
    AstKind,
    ast::{BindingPattern, CallExpression, Expression, Program, Statement},
};
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::{GetSpan, Span};
use oxc_syntax::symbol::SymbolId;

use super::{META, ScriptLintResult};
use crate::diagnostic::LintDiagnostic;
use vize_l0::cstr;

/// Normal-script evidence, borrowed from the existing SFC script pass.
#[derive(Clone, Copy)]
pub enum NoDupeKeysModuleScript<'a> {
    Absent,
    Unavailable,
    Parsed {
        program: &'a Program<'a>,
        offset: usize,
    },
}

pub(crate) fn has_candidate(program: &Program<'_>) -> bool {
    candidates(program).next().is_some()
}

pub(super) fn check<'a>(
    program: &'a Program<'a>,
    offset: usize,
    module: NoDupeKeysModuleScript<'a>,
    result: &mut ScriptLintResult,
) {
    if !has_candidate(program) || matches!(module, NoDupeKeysModuleScript::Unavailable) {
        return;
    }
    let built = SemanticBuilder::new().with_build_nodes(true).build(program);
    if !built.diagnostics.is_empty() {
        return;
    }
    let semantic = built.semantic;
    let module_semantic = match module {
        NoDupeKeysModuleScript::Parsed { program, .. } => {
            let built = SemanticBuilder::new().with_build_nodes(true).build(program);
            if !built.diagnostics.is_empty() {
                return;
            }
            Some(built.semantic)
        }
        _ => None,
    };
    let mut setup_result = ScriptLintResult::default();
    for candidate in candidates(program) {
        if macro_shadowed(program, &semantic, "defineProps")
            || (candidate.wrapped && macro_shadowed(program, &semantic, "withDefaults"))
            || module_semantic.as_ref().is_some_and(|semantic| {
                let NoDupeKeysModuleScript::Parsed { program, .. } = module else {
                    return false;
                };
                macro_shadowed(program, semantic, "defineProps")
                    || (candidate.wrapped && macro_shadowed(program, semantic, "withDefaults"))
            })
        {
            continue;
        }
        let prop_names = props::names(
            candidate.call,
            program,
            &semantic,
            module,
            module_semantic.as_ref(),
        );
        let prop_symbols: Vec<SymbolId> = candidate
            .pattern
            .into_iter()
            .flat_map(BindingPattern::get_binding_identifiers)
            .filter_map(|id| id.symbol_id.get())
            .collect();
        for name in prop_names {
            if renamed(candidate.pattern, &name) {
                continue;
            }
            if let Some(symbol) = root_value(&semantic, &name) {
                if derived_from_props(&semantic, symbol, &prop_symbols, candidate.call.span) {
                    continue;
                }
                report(&semantic, symbol, &name, offset, &mut setup_result);
            } else if let (Some(semantic), NoDupeKeysModuleScript::Parsed { offset, .. }) =
                (module_semantic.as_ref(), module)
                && let Some(symbol) = root_value(semantic, &name)
            {
                report(semantic, symbol, &name, offset, &mut setup_result);
            }
        }
    }
    setup_result
        .diagnostics
        .sort_by_key(|diagnostic| (diagnostic.start, diagnostic.end));
    for diagnostic in setup_result.diagnostics {
        result.add_diagnostic(diagnostic);
    }
}

struct Candidate<'a> {
    call: &'a CallExpression<'a>,
    pattern: Option<&'a BindingPattern<'a>>,
    wrapped: bool,
}

fn candidates<'a>(program: &'a Program<'a>) -> impl Iterator<Item = Candidate<'a>> {
    program.body.iter().flat_map(|statement| {
        let mut calls = Vec::new();
        match statement {
            Statement::ExpressionStatement(statement) => {
                if let Some(candidate) = candidate(&statement.expression, None) {
                    calls.push(candidate);
                }
            }
            Statement::VariableDeclaration(declaration) if !declaration.declare => {
                for declarator in &declaration.declarations {
                    if let Some(initializer) = &declarator.init
                        && let Some(candidate) = candidate(initializer, Some(&declarator.id))
                    {
                        calls.push(candidate);
                    }
                }
            }
            _ => {}
        }
        calls
    })
}

fn candidate<'a>(
    expression: &'a Expression<'a>,
    pattern: Option<&'a BindingPattern<'a>>,
) -> Option<Candidate<'a>> {
    let Expression::CallExpression(call) = expression.get_inner_expression() else {
        return None;
    };
    if named(call, "defineProps") {
        return Some(Candidate {
            call,
            pattern,
            wrapped: false,
        });
    }
    if named(call, "withDefaults")
        && let Some(Expression::CallExpression(props)) = call
            .arguments
            .first()
            .and_then(|argument| argument.as_expression())
            .map(Expression::get_inner_expression)
        && named(props, "defineProps")
    {
        return Some(Candidate {
            call: props,
            pattern,
            wrapped: true,
        });
    }
    None
}

fn named(call: &CallExpression<'_>, name: &str) -> bool {
    !call.optional && matches!(&call.callee, Expression::Identifier(id) if id.name == name)
}

fn root_value(semantic: &Semantic<'_>, name: &str) -> Option<SymbolId> {
    let scoping = semantic.scoping();
    scoping
        .get_binding(scoping.root_scope_id(), name.into())
        .filter(|symbol| {
            let flags = scoping.symbol_flags(*symbol);
            flags.is_value() && !flags.is_type_import() && !flags.is_ambient()
        })
}

fn macro_shadowed(program: &Program<'_>, semantic: &Semantic<'_>, name: &str) -> bool {
    if root_value(semantic, name).is_none() {
        return false;
    }
    // Vue's redundant named compiler-macro imports are compiler-owned too.
    !program.body.iter().any(|statement| {
        let Statement::ImportDeclaration(import) = statement else {
            return false;
        };
        import.source.value == "vue" && import.specifiers.iter().flatten().any(|specifier| {
            matches!(specifier, oxc_ast::ast::ImportDeclarationSpecifier::ImportSpecifier(specifier)
                if specifier.local.name == name && specifier.imported.name() == name)
        })
    })
}

fn derived_from_props(
    semantic: &Semantic<'_>,
    symbol: SymbolId,
    props: &[SymbolId],
    call: Span,
) -> bool {
    // A destructured prop or its own props-object declaration is an alias.
    if props.contains(&symbol) {
        return true;
    }
    let AstKind::VariableDeclarator(declaration) = semantic.symbol_declaration(symbol).kind()
    else {
        return false;
    };
    let Some(initializer) = &declaration.init else {
        return false;
    };
    let span = initializer.span();
    if span.contains_inclusive(call) {
        return true;
    }
    props.iter().any(|prop| {
        semantic.symbol_references(*prop).any(|reference| {
            reference.is_value()
                && span.contains_inclusive(
                    semantic.nodes().get_node(reference.node_id()).kind().span(),
                )
        })
    })
}

fn renamed(pattern: Option<&BindingPattern<'_>>, name: &str) -> bool {
    let Some(BindingPattern::ObjectPattern(pattern)) = pattern else {
        return false;
    };
    pattern.properties.iter().any(|property| {
        matches!((&property.key, &property.value),
            (oxc_ast::ast::PropertyKey::StaticIdentifier(key), BindingPattern::BindingIdentifier(value))
                if key.name == name && value.name != key.name)
    })
}

fn report(
    semantic: &Semantic<'_>,
    symbol: SymbolId,
    name: &str,
    offset: usize,
    result: &mut ScriptLintResult,
) {
    // Whole declaration spans match the native JS/TS declaration owner, including
    // destructuring where several prop collisions legitimately share one span.
    let span = semantic.symbol_declaration(symbol).kind().span();
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    result.add_diagnostic(
        LintDiagnostic::error(META.name, cstr!("Duplicated key '{name}'"), start, end)
            .with_label("binding shadows a prop", start, end)
            .with_help("A prop and a setup binding share the component instance namespace; rename the binding or derive it from the prop."),
    );
}

#[cfg(test)]
mod tests;
