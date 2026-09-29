use super::document::TypeAwareDocument;
use super::{
    LintResult, Linter, RULE_NO_REACTIVITY_LOSS, markers::marker_insert_offset, push_warning,
};
use crate::diagnostic::LintDiagnostic;
use oxc_allocator::Allocator as OxcAllocator;
use oxc_ast::ast::{
    Argument, ArrowFunctionExpression, BindingPattern, CallExpression, ChainElement, Expression,
    Function, FunctionBody, ImportDeclaration, ImportDeclarationSpecifier, ModuleExportName,
    ObjectExpression, ObjectPropertyKind, PropertyKey, PropertyKind, SpreadElement,
    VariableDeclarator,
};
use oxc_ast_visit::{
    Visit,
    walk::{
        walk_arrow_function_expression, walk_call_expression, walk_function,
        walk_import_declaration, walk_spread_element, walk_variable_declarator,
    },
};
use oxc_parser::Parser as OxcParser;
use oxc_span::SourceType;
use oxc_syntax::scope::ScopeFlags;
use vize_croquis::{
    reactivity::{ReactivityLoss, ReactivityLossKind},
    script_parser::ScriptParseResult,
};
use vize_l0::{CompactString, FxHashSet, String, ToCompactString, cstr, profile};

#[derive(Clone)]
pub(super) struct ReactivityLossQuery {
    pub generated_offset: u32,
    pub source_start: u32,
    pub source_end: u32,
    message: CompactString,
    help: &'static str,
}

impl ReactivityLossQuery {
    #[inline]
    pub fn owner_key(&self) -> u64 {
        ((self.source_start as u64) << 32) | self.source_end as u64
    }

    pub fn diagnostic(&self, script_offset: u32) -> LintDiagnostic {
        LintDiagnostic::warn(
            RULE_NO_REACTIVITY_LOSS,
            self.message.clone(),
            script_offset + self.source_start,
            script_offset + self.source_end,
        )
        .with_help(self.help)
    }
}

pub(super) fn collect_reactivity_loss_queries(
    linter: &Linter,
    result: &mut LintResult,
    parse_result: &ScriptParseResult,
    script_content: &str,
    script_offset: u32,
    virtual_ts: &mut TypeAwareDocument,
) -> Vec<ReactivityLossQuery> {
    if !(linter.registry.has_rule(RULE_NO_REACTIVITY_LOSS)
        && linter.is_rule_enabled(RULE_NO_REACTIVITY_LOSS))
    {
        return Vec::new();
    }
    if !parse_result.reactivity.has_losses() {
        return Vec::new();
    }

    let mut queries = Vec::with_capacity(parse_result.reactivity.losses().len());
    let mut immediate = FxHashSet::default();
    let exempt_value_spreads = if parse_result
        .reactivity
        .losses()
        .iter()
        .any(|loss| matches!(loss.kind, ReactivityLossKind::ReactiveSpread { .. }))
    {
        computed_value_spread_spans(script_content)
    } else {
        Default::default()
    };

    for loss in parse_result.reactivity.losses() {
        if matches!(loss.kind, ReactivityLossKind::ReactiveSpread { .. })
            && exempt_value_spreads.contains(&(loss.start, loss.end))
        {
            continue;
        }
        let diagnostic = reactivity_loss_diagnostic(loss);
        let expressions = query_expressions_for_loss(loss, script_content);

        if expressions.is_empty() {
            let key = diagnostic_key(loss.start, loss.end);
            if immediate.insert(key) {
                push_warning(result, diagnostic.diagnostic(script_offset));
            }
            continue;
        }

        for expression in expressions {
            if let Some(query) =
                push_reactivity_loss_marker(virtual_ts, expression.as_str(), &diagnostic)
            {
                queries.push(query);
            }
        }
    }

    queries
}

fn push_reactivity_loss_marker(
    virtual_ts: &mut TypeAwareDocument,
    expression_source: &str,
    diagnostic: &ReactivityLossQuery,
) -> Option<ReactivityLossQuery> {
    let expression_source = expression_source.trim();
    if expression_source.is_empty() {
        return None;
    }
    let insert_offset = marker_insert_offset(&virtual_ts.content)?;

    let mut marker_name = String::with_capacity(32);
    marker_name.push_str("__vize_patina_reactivity_");
    marker_name.push_str(diagnostic.source_start.to_compact_string().as_str());
    marker_name.push('_');
    marker_name.push_str(diagnostic.source_end.to_compact_string().as_str());
    marker_name.push('_');
    marker_name.push_str(virtual_ts.content.len().to_compact_string().as_str());

    let mut line = String::with_capacity(marker_name.len() + expression_source.len() + 24);
    line.push_str("    const ");
    let name_offset = line.len() as u32;
    line.push_str(&marker_name);
    line.push_str(" = (");
    line.push_str(expression_source);
    line.push_str(");\n");

    let mut query = diagnostic.clone();
    query.generated_offset = insert_offset as u32 + name_offset;
    virtual_ts.content.insert_str(insert_offset, &line);
    Some(query)
}

fn query_expressions_for_loss(loss: &ReactivityLoss, script_content: &str) -> Vec<CompactString> {
    match &loss.kind {
        ReactivityLossKind::PropsDestructure { .. } => Vec::new(),
        ReactivityLossKind::RefValueExtract { .. }
        | ReactivityLossKind::ReactivePropertyExtract { .. }
        | ReactivityLossKind::FunctionArgumentExtract { .. }
        | ReactivityLossKind::GetterCallExtract { .. }
        | ReactivityLossKind::PlainValueAlias { .. } => script_content
            .get(loss.start as usize..loss.end as usize)
            .map(str::trim)
            .filter(|source| !source.is_empty())
            .map(|source| vec![CompactString::new(source)])
            .unwrap_or_default(),
        ReactivityLossKind::ReactiveDestructure { .. }
        | ReactivityLossKind::RefValueDestructure { .. }
        | ReactivityLossKind::ReactiveSpread { .. }
        | ReactivityLossKind::ReactiveReassign { .. } => Vec::new(),
    }
}

fn computed_value_spread_spans(script: &str) -> FxHashSet<(u32, u32)> {
    let allocator = OxcAllocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let parsed = profile!(
        "patina.type_aware.computed_spread.parse",
        OxcParser::new(&allocator, script, source_type).parse()
    );
    if parsed.panicked {
        return FxHashSet::default();
    }

    let mut collector = ComputedValueSpreadCollector::default();
    collector.visit_program(&parsed.program);
    collector.exempt
}

#[derive(Default)]
struct ComputedValueSpreadCollector {
    aliases: FxHashSet<String>,
    exempt: FxHashSet<(u32, u32)>,
    in_getter: bool,
    nested: u32,
}

impl ComputedValueSpreadCollector {
    fn is_computed_name(&self, name: &str) -> bool {
        name == "computed" || self.aliases.contains(name)
    }

    fn remember_alias(&mut self, name: &str) {
        self.aliases.insert(name.to_compact_string());
    }

    fn visit_computed_argument(&mut self, argument: &Argument<'_>) {
        if let Some(expression) = argument.as_expression() {
            self.visit_computed_expression(expression);
        } else {
            self.visit_argument(argument);
        }
    }

    fn visit_computed_expression(&mut self, expression: &Expression<'_>) {
        match expression {
            Expression::ArrowFunctionExpression(arrow) => self.visit_arrow_getter(arrow),
            Expression::FunctionExpression(function) => self.visit_function_getter(function),
            Expression::ObjectExpression(object) => self.visit_computed_options(object),
            Expression::ParenthesizedExpression(paren) => {
                self.visit_computed_expression(&paren.expression);
            }
            Expression::TSAsExpression(ts_as) => self.visit_computed_expression(&ts_as.expression),
            Expression::TSSatisfiesExpression(ts_satisfies) => {
                self.visit_computed_expression(&ts_satisfies.expression);
            }
            Expression::TSNonNullExpression(ts_non_null) => {
                self.visit_computed_expression(&ts_non_null.expression);
            }
            other => self.visit_expression(other),
        }
    }

    fn visit_arrow_getter(&mut self, arrow: &ArrowFunctionExpression<'_>) {
        if let Some(type_parameters) = &arrow.type_parameters {
            self.visit_ts_type_parameter_declaration(type_parameters);
        }
        self.visit_formal_parameters(&arrow.params);
        if let Some(return_type) = &arrow.return_type {
            self.visit_ts_type_annotation(return_type);
        }
        self.visit_getter_body(&arrow.body);
    }

    fn visit_function_getter(&mut self, function: &Function<'_>) {
        self.visit_formal_parameters(&function.params);
        if let Some(body) = &function.body {
            self.visit_getter_body(body);
        }
    }

    fn visit_getter_body(&mut self, body: &FunctionBody<'_>) {
        let previous = self.in_getter;
        self.in_getter = true;
        for statement in &body.statements {
            self.visit_statement(statement);
        }
        self.in_getter = previous;
    }

    fn visit_computed_options(&mut self, object: &ObjectExpression<'_>) {
        for property in &object.properties {
            match property {
                ObjectPropertyKind::ObjectProperty(property) => {
                    let is_getter = property.kind == PropertyKind::Get
                        || property_name(&property.key) == Some("get");
                    if let Some(key) = property.key.as_expression() {
                        self.visit_expression(key);
                    }
                    if is_getter {
                        self.visit_computed_expression(&property.value);
                    } else {
                        self.visit_expression(&property.value);
                    }
                }
                ObjectPropertyKind::SpreadProperty(spread) => self.visit_spread_element(spread),
            }
        }
    }
}

impl<'a> Visit<'a> for ComputedValueSpreadCollector {
    fn visit_import_declaration(&mut self, declaration: &ImportDeclaration<'a>) {
        if declaration.source.value.as_str() == "vue"
            && let Some(specifiers) = &declaration.specifiers
        {
            for specifier in specifiers {
                let ImportDeclarationSpecifier::ImportSpecifier(specifier) = specifier else {
                    continue;
                };
                let imported = match &specifier.imported {
                    ModuleExportName::IdentifierName(name) => name.name.as_str(),
                    ModuleExportName::IdentifierReference(name) => name.name.as_str(),
                    ModuleExportName::StringLiteral(name) => name.value.as_str(),
                };
                if imported == "computed" {
                    self.remember_alias(specifier.local.name.as_str());
                }
            }
        }
        walk_import_declaration(self, declaration);
    }

    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if let BindingPattern::BindingIdentifier(binding) = &declarator.id
            && let Some(init) = &declarator.init
            && let Some(name) = identifier_name(init)
            && self.is_computed_name(name)
        {
            self.remember_alias(binding.name.as_str());
        }
        walk_variable_declarator(self, declarator);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if callee_is_computed(&call.callee, self) {
            self.visit_expression(&call.callee);
            for (index, argument) in call.arguments.iter().enumerate() {
                if index == 0 {
                    self.visit_computed_argument(argument);
                } else {
                    self.visit_argument(argument);
                }
            }
            return;
        }
        walk_call_expression(self, call);
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if self.in_getter {
            self.nested += 1;
            walk_arrow_function_expression(self, arrow);
            self.nested -= 1;
            return;
        }
        walk_arrow_function_expression(self, arrow);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        if self.in_getter {
            self.nested += 1;
            walk_function(self, function, flags);
            self.nested -= 1;
            return;
        }
        walk_function(self, function, flags);
    }

    fn visit_spread_element(&mut self, spread: &SpreadElement<'a>) {
        if self.in_getter && self.nested == 0 && is_ref_value_expression(&spread.argument) {
            self.exempt.insert((spread.span.start, spread.span.end));
        }
        walk_spread_element(self, spread);
    }
}

fn callee_is_computed(callee: &Expression<'_>, collector: &ComputedValueSpreadCollector) -> bool {
    let mut expression = callee;
    loop {
        match expression {
            Expression::Identifier(identifier) => {
                return collector.is_computed_name(identifier.name.as_str());
            }
            Expression::ParenthesizedExpression(paren) => expression = &paren.expression,
            _ => return false,
        }
    }
}

fn identifier_name<'a>(expression: &'a Expression<'a>) -> Option<&'a str> {
    let mut expression = expression;
    loop {
        match expression {
            Expression::Identifier(identifier) => return Some(identifier.name.as_str()),
            Expression::ParenthesizedExpression(paren) => expression = &paren.expression,
            _ => return None,
        }
    }
}

fn property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(string) => Some(string.value.as_str()),
        _ => None,
    }
}

fn is_ref_value_expression(expression: &Expression<'_>) -> bool {
    match unwrap_value_expression(expression) {
        Expression::StaticMemberExpression(member) => member.property.name.as_str() == "value",
        Expression::ChainExpression(chain) => match &chain.expression {
            ChainElement::StaticMemberExpression(member) => {
                member.property.name.as_str() == "value"
            }
            _ => false,
        },
        _ => false,
    }
}

fn unwrap_value_expression<'a>(expression: &'a Expression<'a>) -> &'a Expression<'a> {
    let mut expression = expression;
    loop {
        expression = match expression {
            Expression::ParenthesizedExpression(paren) => &paren.expression,
            Expression::TSAsExpression(ts_as) => &ts_as.expression,
            Expression::TSSatisfiesExpression(ts_satisfies) => &ts_satisfies.expression,
            Expression::TSNonNullExpression(ts_non_null) => &ts_non_null.expression,
            other => return other,
        };
    }
}

fn reactivity_loss_diagnostic(loss: &ReactivityLoss) -> ReactivityLossQuery {
    let (message, help) = match &loss.kind {
        ReactivityLossKind::ReactiveDestructure {
            source_name,
            destructured_props,
        } => (
            cstr!(
                "Destructuring reactive value '{}' creates plain snapshots for: {}",
                source_name,
                destructured_props.join(", ")
            ),
            "Use `toRefs(...)`, `toRef(...)`, or access the property through the reactive object.",
        ),
        ReactivityLossKind::RefValueDestructure {
            source_name,
            destructured_props,
        } => (
            cstr!(
                "Destructuring '{}.value' creates plain snapshots for: {}",
                source_name,
                destructured_props.join(", ")
            ),
            "Keep the ref boundary and derive values through `computed(...)` or `toRef(...)`.",
        ),
        ReactivityLossKind::RefValueExtract {
            source_name,
            target_name,
        } => (
            cstr!(
                "Assigning '{}.value' to '{}' stores a plain snapshot",
                source_name,
                target_name
            ),
            "Pass the ref itself, use a getter `() => ref.value`, or wrap the derived value in `computed(...)`.",
        ),
        ReactivityLossKind::ReactivePropertyExtract {
            source_name,
            prop_name,
            target_name,
        } => (
            cstr!(
                "Assigning '{}.{}' to '{}' stores a plain snapshot",
                source_name,
                prop_name,
                target_name
            ),
            "Use `toRef(source, 'key')`, `toRefs(source)`, or access the property on the reactive object.",
        ),
        ReactivityLossKind::PropsDestructure { destructured_props } => (
            cstr!(
                "Destructuring props creates plain snapshots for: {}",
                destructured_props.join(", ")
            ),
            "Use `toRefs(props)`, `toRef(props, 'key')`, or pass a getter `() => prop` across call boundaries.",
        ),
        ReactivityLossKind::FunctionArgumentExtract {
            source_name,
            argument_name,
            callee_name,
        } => (
            cstr!(
                "Passing '{}' to '{}' cuts the reactive graph from '{}'",
                argument_name,
                callee_name,
                source_name
            ),
            "Pass `Ref<T>` or `ComputedRef<T>` instead, for example `toRef(source, 'key')` or `computed(() => value)`.",
        ),
        ReactivityLossKind::GetterCallExtract {
            context_name,
            getter_name,
            target_name,
            callee_name,
            source_name,
        } => (
            cstr!(
                "Assigning '{}.{}()' to '{}' stores a plain snapshot from '{}' returned by '{}'",
                context_name,
                getter_name,
                target_name,
                source_name,
                callee_name
            ),
            "Keep the getter lazy, wrap it in `computed(...)`, or have the composable return a ref-like value.",
        ),
        ReactivityLossKind::PlainValueAlias {
            source_name,
            alias_name,
            target_name,
        } if alias_name == "<mutation>" => (
            cstr!(
                "Mutating '{}' writes through a plain snapshot from '{}'",
                target_name,
                source_name
            ),
            "Mutate the reactive source directly, or keep the value as a ref/computed.",
        ),
        ReactivityLossKind::PlainValueAlias {
            source_name,
            alias_name,
            target_name,
        } => (
            cstr!(
                "Assigning plain snapshot '{}' to '{}' keeps reactivity lost from '{}'",
                alias_name,
                target_name,
                source_name
            ),
            "Pass the reactive source itself, a getter, `toRef(...)`, or `computed(...)` instead of aliasing the snapshot.",
        ),
        ReactivityLossKind::ReactiveSpread { source_name } => (
            cstr!("Spreading '{}' creates a non-reactive copy", source_name),
            "Keep the reactive object intact, or copy through refs with `toRefs(...)` when destructuring is intentional.",
        ),
        ReactivityLossKind::ReactiveReassign { source_name } => (
            cstr!(
                "Reassigning reactive binding '{}' breaks tracked identity",
                source_name
            ),
            "Mutate the reactive object in place or store replaceable state in a ref.",
        ),
    };

    ReactivityLossQuery {
        generated_offset: 0,
        source_start: loss.start,
        source_end: loss.end.max(loss.start.saturating_add(1)),
        message,
        help,
    }
}

#[inline]
fn diagnostic_key(start: u32, end: u32) -> u64 {
    ((start as u64) << 32) | end as u64
}

#[cfg(test)]
mod tests {
    use super::super::lint_sfc_with_corsa;
    use crate::{LintPreset, Linter};

    fn spread_messages(source: &str) -> Vec<std::string::String> {
        let linter = Linter::with_preset(LintPreset::Opinionated).with_type_aware_lint(true);
        let wrapped = format!("<script setup lang=\"ts\">\n{source}\n</script>\n");
        let result = lint_sfc_with_corsa(&linter, &wrapped, "Fixture.vue");
        result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "type/no-reactivity-loss")
            .map(|diagnostic| diagnostic.message.as_str().to_string())
            .collect()
    }

    #[test]
    fn value_spread_inside_computed_getter_is_not_reactivity_loss() {
        let messages = spread_messages(
            r#"
import { computed as useComputed, ref } from 'vue'
const state = ref({ count: 1, tags: ['a'] })
const alias = useComputed
const view = computed(() => ({ ...state.value }))
const block = computed(() => { return { ...state.value } })
const options = computed({ get() { return { ...state.value } } })
const viaAlias = alias(() => ({ ...state.value }))
const outside = { ...state.value }
const nested = computed(() => {
  const leak = () => ({ ...state.value })
  return leak()
})
const tags = { ...state.value.tags }
"#,
        );
        let mut value_spreads = 0;
        let mut tag_spreads = 0;
        for message in &messages {
            if message.contains("Spreading 'state.value.tags'") {
                tag_spreads += 1;
            } else if message.contains("Spreading 'state.value'") {
                value_spreads += 1;
            }
        }
        assert_eq!(value_spreads, 2, "{messages:?}");
        assert_eq!(tag_spreads, 1, "{messages:?}");
    }
}
