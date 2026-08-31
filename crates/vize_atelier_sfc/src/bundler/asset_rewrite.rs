mod predicates;
mod replacements;

use oxc_allocator::Allocator;
use oxc_ast::ast::{
    ArrowFunctionExpression, Expression, Function, ObjectProperty, ReturnStatement, StringLiteral,
    TemplateLiteral, VariableDeclarator,
};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType};
use oxc_syntax::scope::ScopeFlags;
use predicates::{is_render_function_name, is_template_hoist_declarator, property_key_name};
use replacements::{
    AssetReferenceReplacement, apply_asset_replacements, asset_expression, join_expression_parts,
    push_string_part,
};
use vize_s0::{SmallVec, String};

use super::assets::TemplateAssetUrl;

/// Rewrite compiled template asset references into import identifiers.
///
/// The Rspack builder compiles the SFC first, then assembles the final module
/// output. Template asset references can appear either in render functions, in
/// compiler hoists, or inside SSR HTML template literals. Restricting edits to
/// those AST-owned regions keeps same-valued strings from `<script>` intact.
pub fn rewrite_template_asset_references(code: &str, assets: &[TemplateAssetUrl]) -> String {
    if assets.is_empty() {
        return String::from(code);
    }

    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, code, SourceType::tsx().with_module(true)).parse();
    if !parsed.diagnostics.is_empty() {
        return String::from(code);
    }

    let mut collector = TemplateAssetReferenceCollector {
        code,
        assets,
        replacements: Vec::new(),
        render_depth: 0,
        setup_depth: 0,
        setup_function_depth: 0,
    };
    collector.visit_program(&parsed.program);
    apply_asset_replacements(code, collector.replacements)
}

struct TemplateAssetReferenceCollector<'a> {
    code: &'a str,
    assets: &'a [TemplateAssetUrl],
    replacements: Vec<AssetReferenceReplacement>,
    render_depth: usize,
    setup_depth: usize,
    setup_function_depth: usize,
}

impl<'a> TemplateAssetReferenceCollector<'a> {
    fn with_render_scope(&mut self, visit: impl FnOnce(&mut Self)) {
        self.render_depth += 1;
        visit(self);
        self.render_depth -= 1;
    }

    fn with_setup_scope(&mut self, visit: impl FnOnce(&mut Self)) {
        self.setup_depth += 1;
        visit(self);
        self.setup_depth -= 1;
    }

    fn with_setup_nested_function(&mut self, visit: impl FnOnce(&mut Self)) {
        self.setup_function_depth += 1;
        visit(self);
        self.setup_function_depth -= 1;
    }

    fn asset_for_value(&self, value: &str) -> Option<&'a TemplateAssetUrl> {
        self.assets.iter().find(|asset| asset.url.as_str() == value)
    }

    fn collect_string_literal(&mut self, literal: &StringLiteral<'_>) {
        if self.render_depth == 0 {
            return;
        }

        let Some(asset) = self.asset_for_value(literal.value.as_str()) else {
            return;
        };

        self.replacements.push(AssetReferenceReplacement {
            start: literal.span.start as usize,
            end: literal.span.end as usize,
            value: asset_expression(asset),
        });
    }

    fn collect_template_literal(&mut self, template: &TemplateLiteral<'_>) -> bool {
        if self.render_depth == 0 {
            return false;
        }

        let Some(value) = self.template_literal_expression(template) else {
            return false;
        };

        self.replacements.push(AssetReferenceReplacement {
            start: template.span.start as usize,
            end: template.span.end as usize,
            value,
        });
        true
    }

    fn template_literal_expression(&self, template: &TemplateLiteral<'_>) -> Option<String> {
        let mut parts: SmallVec<[String; 8]> = SmallVec::new();
        let mut changed = false;

        for (index, quasi) in template.quasis.iter().enumerate() {
            let text = quasi
                .value
                .cooked
                .as_ref()
                .map_or_else(|| quasi.value.raw.as_str(), |value| value.as_str());
            self.push_template_text_parts(text, &mut parts, &mut changed);

            if let Some(expression) = template.expressions.get(index)
                && let Some((source, expression_changed)) =
                    self.template_expression_part(expression)
            {
                changed |= expression_changed;
                parts.push(source);
            }
        }

        if !changed {
            return None;
        }

        Some(join_expression_parts(parts))
    }

    fn template_expression_part<'b>(&self, expression: &Expression<'b>) -> Option<(String, bool)> {
        if let Some(value) = self.template_expression_asset(expression) {
            return Some((value, true));
        }

        let span = expression.span();
        let start = span.start as usize;
        let end = span.end as usize;
        if start > end || end > self.code.len() {
            return None;
        }

        let mut source = String::from("(");
        source.push_str(&self.code[start..end]);
        source.push(')');
        Some((source, false))
    }

    fn template_expression_asset<'b>(&self, expression: &Expression<'b>) -> Option<String> {
        match expression {
            Expression::StringLiteral(literal) => self
                .asset_for_value(literal.value.as_str())
                .map(asset_expression),
            Expression::TemplateLiteral(template) => self.template_literal_expression(template),
            _ => None,
        }
    }

    fn push_template_text_parts(
        &self,
        text: &str,
        parts: &mut SmallVec<[String; 8]>,
        changed: &mut bool,
    ) {
        let mut cursor = 0usize;
        while cursor < text.len() {
            let Some((relative_start, asset)) = self.find_next_asset(&text[cursor..]) else {
                push_string_part(parts, &text[cursor..]);
                return;
            };

            let start = cursor + relative_start;
            push_string_part(parts, &text[cursor..start]);
            parts.push(asset_expression(asset));
            *changed = true;
            cursor = start + asset.url.len();
        }

        push_string_part(parts, "");
    }

    fn find_next_asset<'b>(&'a self, text: &'b str) -> Option<(usize, &'a TemplateAssetUrl)> {
        self.assets
            .iter()
            .filter_map(|asset| text.find(asset.url.as_str()).map(|index| (index, asset)))
            .min_by(|(left_index, left_asset), (right_index, right_asset)| {
                left_index
                    .cmp(right_index)
                    .then_with(|| right_asset.url.len().cmp(&left_asset.url.len()))
            })
    }
}

impl<'a> Visit<'a> for TemplateAssetReferenceCollector<'_> {
    fn visit_variable_declarator(&mut self, declarator: &VariableDeclarator<'a>) {
        if is_template_hoist_declarator(declarator) {
            self.with_render_scope(|this| {
                if let Some(init) = &declarator.init {
                    this.visit_expression(init);
                }
            });
            return;
        }

        walk::walk_variable_declarator(self, declarator);
    }

    fn visit_function(&mut self, function: &Function<'a>, flags: ScopeFlags) {
        if function
            .id
            .as_ref()
            .is_some_and(|id| is_render_function_name(id.name.as_str()))
        {
            self.with_render_scope(|this| walk::walk_function(this, function, flags));
            return;
        }

        if self.setup_depth > 0 {
            self.with_setup_nested_function(|this| walk::walk_function(this, function, flags));
            return;
        }

        walk::walk_function(self, function, flags);
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if self.setup_depth > 0 {
            self.with_setup_nested_function(|this| {
                walk::walk_arrow_function_expression(this, arrow)
            });
            return;
        }

        walk::walk_arrow_function_expression(self, arrow);
    }

    fn visit_object_property(&mut self, property: &ObjectProperty<'a>) {
        match property_key_name(&property.key) {
            Some("setup") => {
                if self.visit_setup_expression(&property.value) {
                    return;
                }
                self.with_setup_scope(|this| walk::walk_object_property(this, property));
            }
            Some("render" | "ssrRender") => {
                if self.visit_render_expression(&property.value) {
                    return;
                }
                walk::walk_object_property(self, property);
            }
            _ => walk::walk_object_property(self, property),
        }
    }

    fn visit_return_statement(&mut self, statement: &ReturnStatement<'a>) {
        if self.setup_depth > 0
            && self.setup_function_depth == 0
            && let Some(argument) = &statement.argument
            && self.visit_render_expression(argument)
        {
            return;
        }

        walk::walk_return_statement(self, statement);
    }

    fn visit_string_literal(&mut self, literal: &StringLiteral<'a>) {
        self.collect_string_literal(literal);
        walk::walk_string_literal(self, literal);
    }

    fn visit_template_literal(&mut self, template: &TemplateLiteral<'a>) {
        if self.collect_template_literal(template) {
            return;
        }

        walk::walk_template_literal(self, template);
    }
}

impl TemplateAssetReferenceCollector<'_> {
    fn visit_setup_expression<'a>(&mut self, expression: &Expression<'a>) -> bool {
        match expression {
            Expression::ArrowFunctionExpression(arrow) => {
                self.with_setup_scope(|this| walk::walk_arrow_function_expression(this, arrow));
                true
            }
            Expression::FunctionExpression(function) => {
                self.with_setup_scope(|this| {
                    walk::walk_function(this, function, ScopeFlags::Function)
                });
                true
            }
            _ => false,
        }
    }

    fn visit_render_expression<'a>(&mut self, expression: &Expression<'a>) -> bool {
        match expression {
            Expression::ArrowFunctionExpression(arrow) => {
                self.with_render_scope(|this| walk::walk_arrow_function_expression(this, arrow));
                true
            }
            Expression::FunctionExpression(function) => {
                self.with_render_scope(|this| {
                    walk::walk_function(this, function, ScopeFlags::Function)
                });
                true
            }
            _ => false,
        }
    }
}
