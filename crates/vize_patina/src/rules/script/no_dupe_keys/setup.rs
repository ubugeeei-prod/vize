//! Prop collisions with actual setup-scope value declarations.
//!
//! The SFC dispatcher supplies its existing setup parse and an explicit sibling
//! packet. Unknown siblings do not prove macro ownership. Semantic references
//! distinguish prop-derived initializers from same-spelled nested locals.

mod derivation;
pub(crate) mod props;

use oxc_ast::{
    AstKind,
    ast::{Argument, BindingPattern, CallExpression, Expression, Program, Statement},
};
use oxc_semantic::{Semantic, SemanticBuilder};
use oxc_span::{GetSpan, Span};
use oxc_syntax::symbol::SymbolId;

use super::{META, ScriptLintResult};
use crate::diagnostic::LintDiagnostic;
use vize_l0::{CompactString, FxHashSet, cstr};

/// Normal-script evidence, borrowed from the existing SFC script pass.
#[derive(Clone, Copy)]
pub enum NoDupeKeysModuleScript<'a> {
    /// The SFC has no normal `<script>` block.
    Absent,
    /// A normal script exists but could not be parsed safely.
    Unavailable,
    /// The normal script was parsed and can provide collision evidence.
    Parsed {
        /// Parsed normal-script program.
        program: &'a Program<'a>,
        /// Byte offset of the script block within the SFC source.
        offset: usize,
    },
}

pub(crate) fn has_candidate(program: &Program<'_>) -> bool {
    candidates(program).next().is_some() || model_candidates(program).next().is_some()
}

pub(super) fn check<'a>(
    program: &'a Program<'a>,
    source: &str,
    offset: usize,
    module: NoDupeKeysModuleScript<'a>,
    resolved_type_world: Option<&vize_croquis::types::ResolvedTypeWorld>,
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
    let mut declared_props = FxHashSet::default();
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
            source,
            resolved_type_world,
        );
        let prop_symbols: Vec<SymbolId> = candidate
            .pattern
            .into_iter()
            .flat_map(BindingPattern::get_binding_identifiers)
            .filter_map(|id| id.symbol_id.get())
            .collect();
        for name in prop_names {
            declared_props.insert(name.clone());
            if renamed(candidate.pattern, &name) {
                continue;
            }
            if let Some(symbol) = root_value(&semantic, &name) {
                if derived_from_props(&semantic, symbol, &prop_symbols, candidate.call.span, &name)
                {
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
    if !macro_shadowed(program, &semantic, "defineModel") {
        for model in model_candidates(program) {
            let (name, name_span) = model_name(model);
            if !declared_props.insert(name.clone()) {
                report_span(&name, name_span, offset, &mut setup_result);
                continue;
            }
            if let Some(symbol) = root_value(&semantic, &name) {
                if !model_owns_binding(&semantic, symbol, model.span) {
                    report(&semantic, symbol, &name, offset, &mut setup_result);
                }
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

fn model_owns_binding(semantic: &Semantic<'_>, symbol: SymbolId, call: Span) -> bool {
    let AstKind::VariableDeclarator(declaration) = semantic.symbol_declaration(symbol).kind()
    else {
        return false;
    };
    declaration
        .init
        .as_ref()
        .is_some_and(|initializer| initializer.span().contains_inclusive(call))
}

fn model_candidates<'a>(program: &'a Program<'a>) -> impl Iterator<Item = &'a CallExpression<'a>> {
    program.body.iter().flat_map(|statement| {
        let mut calls = Vec::new();
        match statement {
            Statement::ExpressionStatement(statement) => {
                if let Expression::CallExpression(call) =
                    statement.expression.get_inner_expression()
                    && named(call, "defineModel")
                {
                    calls.push(call.as_ref());
                }
            }
            Statement::VariableDeclaration(declaration) if !declaration.declare => {
                for declarator in &declaration.declarations {
                    if let Some(Expression::CallExpression(call)) = declarator
                        .init
                        .as_ref()
                        .map(Expression::get_inner_expression)
                        && named(call, "defineModel")
                    {
                        calls.push(call.as_ref());
                    }
                }
            }
            _ => {}
        }
        calls
    })
}

fn model_name(call: &CallExpression<'_>) -> (CompactString, Span) {
    if let Some(Argument::StringLiteral(literal)) = call.arguments.first() {
        return (CompactString::new(literal.value.as_str()), literal.span);
    }
    if let Some(Argument::TemplateLiteral(literal)) = call.arguments.first()
        && literal.expressions.is_empty()
        && let Some(name) = literal.single_quasi()
    {
        return (CompactString::new(name.as_str()), literal.span);
    }
    (CompactString::new("modelValue"), call.span)
}

pub(crate) struct Candidate<'a> {
    pub(crate) call: &'a CallExpression<'a>,
    pub(crate) pattern: Option<&'a BindingPattern<'a>>,
    pub(crate) wrapped: bool,
}

pub(crate) fn candidates<'a>(program: &'a Program<'a>) -> impl Iterator<Item = Candidate<'a>> {
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

pub(crate) fn macro_shadowed(program: &Program<'_>, semantic: &Semantic<'_>, name: &str) -> bool {
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
    name: &str,
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
    if matches!(declaration.id, BindingPattern::ObjectPattern(_))
        && (identifier_is_props(initializer, semantic, props)
            || to_refs_reads_props(initializer, semantic, props))
    {
        return pattern_maps_prop(&declaration.id, symbol, name);
    }
    if derivation::initializer_reads_prop(initializer, semantic, props, name)
        || member_reads_prop(initializer, semantic, props, name)
        || to_ref_reads_prop(initializer, semantic, props, name)
    {
        return true;
    }
    false
}

fn member_reads_prop(
    expression: &Expression<'_>,
    semantic: &Semantic<'_>,
    props: &[SymbolId],
    name: &str,
) -> bool {
    let expression = expression.get_inner_expression();
    let Some(member) = expression.as_member_expression() else {
        return false;
    };
    member.static_property_name() == Some(name)
        && identifier_is_props(member.object(), semantic, props)
}

fn identifier_is_props(
    expression: &Expression<'_>,
    semantic: &Semantic<'_>,
    props: &[SymbolId],
) -> bool {
    let Expression::Identifier(identifier) = expression.get_inner_expression() else {
        return false;
    };
    identifier
        .reference_id
        .get()
        .and_then(|reference| semantic.scoping().get_reference(reference).symbol_id())
        .is_some_and(|symbol| props.contains(&symbol))
}

fn to_ref_reads_prop(
    expression: &Expression<'_>,
    semantic: &Semantic<'_>,
    props: &[SymbolId],
    name: &str,
) -> bool {
    let Expression::CallExpression(call) = expression.get_inner_expression() else {
        return false;
    };
    named(call, "toRef")
        && call
            .arguments
            .first()
            .and_then(Argument::as_expression)
            .is_some_and(|argument| identifier_is_props(argument, semantic, props))
        && call.arguments.get(1).is_some_and(|argument| {
            matches!(argument, Argument::StringLiteral(literal) if literal.value == name)
        })
}

fn to_refs_reads_props(
    expression: &Expression<'_>,
    semantic: &Semantic<'_>,
    props: &[SymbolId],
) -> bool {
    let Expression::CallExpression(call) = expression.get_inner_expression() else {
        return false;
    };
    named(call, "toRefs")
        && call
            .arguments
            .first()
            .and_then(Argument::as_expression)
            .is_some_and(|argument| identifier_is_props(argument, semantic, props))
}

fn pattern_maps_prop(pattern: &BindingPattern<'_>, symbol: SymbolId, name: &str) -> bool {
    let BindingPattern::ObjectPattern(pattern) = pattern else {
        return false;
    };
    pattern.properties.iter().any(|property| {
        property.key.static_name().as_deref() == Some(name)
            && property
                .value
                .get_binding_identifiers()
                .into_iter()
                .any(|identifier| identifier.symbol_id.get() == Some(symbol))
    })
}

fn renamed(pattern: Option<&BindingPattern<'_>>, name: &str) -> bool {
    let Some(BindingPattern::ObjectPattern(pattern)) = pattern else {
        return false;
    };
    pattern.properties.iter().any(|property| {
        property.key.static_name().as_deref() == Some(name)
            && property
                .value
                .get_binding_identifiers()
                .into_iter()
                .any(|identifier| identifier.name != name)
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

fn report_span(name: &str, span: Span, offset: usize, result: &mut ScriptLintResult) {
    let start = offset as u32 + span.start;
    let end = offset as u32 + span.end;
    result.add_diagnostic(
        LintDiagnostic::error(META.name, cstr!("Duplicated key '{name}'"), start, end)
            .with_label("model prop duplicates an existing prop", start, end)
            .with_help("Props, models, and setup bindings share the component instance namespace; give each declaration a unique name."),
    );
}

#[cfg(test)]
mod tests;
