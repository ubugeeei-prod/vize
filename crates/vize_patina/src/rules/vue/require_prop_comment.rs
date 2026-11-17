//! vue/require-prop-comment
//! Checks local prop declarations for immediately preceding JSDoc comments.

use crate::context::LintContext;
use crate::diagnostic::{LintDiagnostic, Severity};
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, CallExpression, Declaration, ExportDefaultDeclarationKind,
    Expression, ObjectPropertyKind, Program, PropertyKey, Statement, TSSignature, TSType,
    TSTypeName,
};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};
use vize_l0::{FxHashMap, FxHashSet, String, cstr};

static META: RuleMeta = RuleMeta {
    name: "vue/require-prop-comment",
    description: "Require a JSDoc comment on each declared prop",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/require-prop-comment`. The default comment form is JSDoc.
#[derive(Default)]
pub struct RequirePropComment;

impl Rule for RequirePropComment {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let findings = {
            let Some(descriptor) = ctx.sfc_descriptor() else {
                return;
            };
            let Some(scripts) = ctx.sfc_scripts() else {
                return;
            };
            let mut blocks = Vec::with_capacity(2);
            if let (Some(block), Some(parsed)) = (&descriptor.script, scripts.script()) {
                blocks.push(ScriptBlock {
                    program: &parsed.program,
                    source: block.content.as_ref(),
                    offset: block.loc.start as u32,
                });
            }
            if let (Some(block), Some(parsed)) = (&descriptor.script_setup, scripts.script_setup())
            {
                blocks.push(ScriptBlock {
                    program: &parsed.program,
                    source: block.content.as_ref(),
                    offset: block.loc.start as u32,
                });
            }
            collect_blocks(&blocks)
        };
        for (message, start, end) in findings {
            ctx.report_in_sfc(LintDiagnostic::warn(META.name, message, start, end));
        }
    }
}

struct ScriptBlock<'a> {
    program: &'a Program<'a>,
    source: &'a str,
    offset: u32,
}

fn collect_blocks(blocks: &[ScriptBlock<'_>]) -> Vec<(String, u32, u32)> {
    let mut findings = Vec::new();
    let mut resolver = TypeResolver::new(blocks);
    for (current, current_block) in blocks.iter().enumerate() {
        let props = {
            let mut visitor = PropVisitor {
                resolver: &mut resolver,
                current,
                props: Vec::new(),
            };
            visitor.visit_program(current_block.program);
            visitor.props
        };
        findings.extend(props.into_iter().filter_map(|prop| {
            let block = blocks.get(prop.block)?;
            if has_jsdoc(block.source, &block.program.comments, prop.anchor) {
                return None;
            }
            Some((
                cstr!(
                    "The \"{}\" property should have a JSDoc comment.",
                    prop.name
                ),
                block.offset + prop.span.start,
                block.offset + prop.span.end,
            ))
        }));
    }
    findings
}

#[derive(Clone)]
struct Prop {
    name: String,
    span: Span,
    anchor: u32,
    block: usize,
}

struct PropVisitor<'resolver, 'ctx, 'ast> {
    resolver: &'resolver mut TypeResolver<'ctx, 'ast>,
    current: usize,
    props: Vec<Prop>,
}

impl<'resolver, 'ctx, 'ast> Visit<'ast> for PropVisitor<'resolver, 'ctx, 'ast> {
    fn visit_call_expression(&mut self, call: &CallExpression<'ast>) {
        match callee_name(&call.callee) {
            Some("defineProps") => self.define_props(call),
            Some("defineComponent") => self.component_arguments(call),
            _ => {}
        }
        walk::walk_call_expression(self, call);
    }

    fn visit_export_default_declaration(
        &mut self,
        declaration: &oxc_ast::ast::ExportDefaultDeclaration<'ast>,
    ) {
        if let ExportDefaultDeclarationKind::ObjectExpression(object) = &declaration.declaration {
            self.props_option(object);
        }
        walk::walk_export_default_declaration(self, declaration);
    }
}

impl<'resolver, 'ctx, 'ast> PropVisitor<'resolver, 'ctx, 'ast> {
    fn define_props(&mut self, call: &CallExpression<'ast>) {
        if let Some(types) = &call.type_arguments
            && let Some(ty) = types.params.first()
        {
            if let Some(props) = self.resolver.props_from_type(ty, 16, self.current) {
                self.extend_props(props);
            }
            return;
        }
        if let Some(argument) = call.arguments.first() {
            self.runtime_argument(argument);
        }
    }

    fn component_arguments(&mut self, call: &CallExpression<'ast>) {
        if let Some(Argument::ObjectExpression(object)) = call.arguments.first() {
            self.props_option(object);
        }
    }

    fn props_option(&mut self, object: &oxc_ast::ast::ObjectExpression<'ast>) {
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            if property_name(&property.key) != Some("props") {
                continue;
            }
            match &property.value {
                Expression::ObjectExpression(props) => {
                    for prop in &props.properties {
                        let ObjectPropertyKind::ObjectProperty(prop) = prop else {
                            continue;
                        };
                        self.note_key(&prop.key, prop.span, self.current);
                    }
                }
                Expression::ArrayExpression(array) => self.array_names(array),
                _ => {}
            }
        }
    }

    fn runtime_argument(&mut self, argument: &Argument<'ast>) {
        match argument {
            Argument::ObjectExpression(object) => {
                for property in &object.properties {
                    let ObjectPropertyKind::ObjectProperty(property) = property else {
                        continue;
                    };
                    self.note_key(&property.key, property.span, self.current);
                }
            }
            Argument::ArrayExpression(array) => self.array_names(array),
            _ => {}
        }
    }

    fn array_names(&mut self, array: &oxc_ast::ast::ArrayExpression<'ast>) {
        for element in &array.elements {
            let ArrayExpressionElement::StringLiteral(literal) = element else {
                continue;
            };
            self.note(
                &literal.value,
                literal.span,
                literal.span.start,
                self.current,
            );
        }
    }

    fn extend_props(&mut self, props: Vec<Prop>) {
        for prop in props {
            self.note(&prop.name, prop.span, prop.anchor, prop.block);
        }
    }

    fn note_key(&mut self, key: &PropertyKey<'ast>, node: Span, block: usize) {
        if let Some(prop) = named(key, node, block) {
            self.note(&prop.name, prop.span, prop.anchor, prop.block);
        }
    }

    fn note(&mut self, name: &str, span: Span, anchor: u32, block: usize) {
        if name.is_empty()
            || self
                .props
                .iter()
                .any(|prop| prop.block == block && prop.span.start == span.start)
        {
            return;
        }
        self.props.push(Prop {
            name: String::from(name),
            span,
            anchor,
            block,
        });
    }
}

#[derive(Clone, Copy)]
enum LocalTypeDeclaration<'ast> {
    Interface(usize, &'ast oxc_ast::ast::TSInterfaceDeclaration<'ast>),
    Alias(usize, &'ast oxc_ast::ast::TSTypeAliasDeclaration<'ast>),
}

impl LocalTypeDeclaration<'_> {
    fn block(self) -> usize {
        match self {
            Self::Interface(block, _) | Self::Alias(block, _) => block,
        }
    }
}

struct TypeResolver<'ctx, 'ast> {
    blocks: &'ctx [ScriptBlock<'ast>],
    declarations: FxHashMap<String, Vec<LocalTypeDeclaration<'ast>>>,
    cache: FxHashMap<(usize, String, u8), Option<Vec<Prop>>>,
    visiting: FxHashSet<(usize, String)>,
    cycle_hits: usize,
}

impl<'ctx, 'ast> TypeResolver<'ctx, 'ast> {
    fn new(blocks: &'ctx [ScriptBlock<'ast>]) -> Self {
        let mut declarations: FxHashMap<String, Vec<LocalTypeDeclaration<'ast>>> =
            FxHashMap::default();
        for (block, script) in blocks.iter().enumerate() {
            for statement in &script.program.body {
                let declaration = match statement {
                    Statement::TSInterfaceDeclaration(interface) => Some((
                        interface.id.name.as_str(),
                        LocalTypeDeclaration::Interface(block, interface),
                    )),
                    Statement::TSTypeAliasDeclaration(alias) => Some((
                        alias.id.name.as_str(),
                        LocalTypeDeclaration::Alias(block, alias),
                    )),
                    Statement::ExportNamedDeclaration(export) => export
                        .declaration
                        .as_ref()
                        .and_then(|declaration| match declaration {
                            Declaration::TSInterfaceDeclaration(interface) => Some((
                                interface.id.name.as_str(),
                                LocalTypeDeclaration::Interface(block, interface),
                            )),
                            Declaration::TSTypeAliasDeclaration(alias) => Some((
                                alias.id.name.as_str(),
                                LocalTypeDeclaration::Alias(block, alias),
                            )),
                            _ => None,
                        }),
                    _ => None,
                };
                if let Some((name, declaration)) = declaration {
                    declarations
                        .entry(String::from(name))
                        .or_default()
                        .push(declaration);
                }
            }
        }
        Self {
            blocks,
            declarations,
            cache: FxHashMap::default(),
            visiting: FxHashSet::default(),
            cycle_hits: 0,
        }
    }

    fn props_from_type(&mut self, ty: &TSType<'ast>, depth: u8, block: usize) -> Option<Vec<Prop>> {
        if depth == 0 {
            return None;
        }
        match ty {
            TSType::TSTypeLiteral(literal) => Some(self.members(&literal.members, block)),
            TSType::TSIntersectionType(intersection) => {
                Some(self.composite_props(&intersection.types, depth - 1, block))
            }
            TSType::TSUnionType(union) => {
                Some(self.composite_props(&union.types, depth - 1, block))
            }
            TSType::TSParenthesizedType(inner) => {
                self.props_from_type(&inner.type_annotation, depth - 1, block)
            }
            TSType::TSTypeReference(reference) => {
                let TSTypeName::IdentifierReference(identifier) = &reference.type_name else {
                    return None;
                };
                self.lookup(identifier.name.as_str(), depth - 1, block)
            }
            _ => None,
        }
    }

    fn composite_props(&mut self, types: &[TSType<'ast>], depth: u8, block: usize) -> Vec<Prop> {
        let mut props = Vec::new();
        for ty in types {
            if let Some(found) = self.props_from_type(ty, depth, block) {
                extend_unique(&mut props, found);
            }
        }
        props
    }

    fn members(&self, members: &[TSSignature<'ast>], block: usize) -> Vec<Prop> {
        let mut props = Vec::new();
        for member in members {
            match member {
                TSSignature::TSPropertySignature(signature) => {
                    if let Some(prop) = named(&signature.key, signature.span, block) {
                        props.push(prop);
                    }
                }
                TSSignature::TSMethodSignature(signature) => {
                    if let Some(prop) = named(&signature.key, signature.span, block) {
                        props.push(prop);
                    }
                }
                _ => {}
            }
        }
        props
    }

    fn lookup(&mut self, name: &str, depth: u8, origin: usize) -> Option<Vec<Prop>> {
        let key = (origin, String::from(name), depth);
        if let Some(cached) = self.cache.get(&key) {
            return cached.clone();
        }
        let visiting = (origin, String::from(name));
        if !self.visiting.insert(visiting.clone()) {
            self.cycle_hits += 1;
            return Some(Vec::new());
        }
        let cycle_hits = self.cycle_hits;
        let candidates = self.declarations.get(name);
        let mut declaration = None;
        for block in (0..self.blocks.len()).map(|index| (origin + index) % self.blocks.len()) {
            if let Some(candidate) = candidates
                .and_then(|entries| entries.iter().find(|entry| entry.block() == block))
                .copied()
            {
                declaration = Some(candidate);
                break;
            }
        }
        let result = declaration.map(|declaration| self.declaration_props(declaration, depth));
        self.visiting.remove(&visiting);
        // A cycle cut is path-dependent. Caching the partial result would make
        // a later lookup incorrectly inherit whichever edge happened to be
        // active first, so only memoize fully acyclic expansions.
        if self.cycle_hits == cycle_hits {
            self.cache.insert(key, result.clone());
        }
        result
    }

    fn declaration_props(
        &mut self,
        declaration: LocalTypeDeclaration<'ast>,
        depth: u8,
    ) -> Vec<Prop> {
        match declaration {
            LocalTypeDeclaration::Interface(block, interface) => {
                self.interface_props(interface, depth, block)
            }
            LocalTypeDeclaration::Alias(block, alias) => self
                .props_from_type(&alias.type_annotation, depth, block)
                .unwrap_or_default(),
        }
    }

    fn interface_props(
        &mut self,
        interface: &oxc_ast::ast::TSInterfaceDeclaration<'ast>,
        depth: u8,
        block: usize,
    ) -> Vec<Prop> {
        let mut props = self.members(&interface.body.body, block);
        if depth == 0 {
            return props;
        }
        let mut seen = FxHashSet::default();
        for heritage in &interface.extends {
            let Expression::Identifier(identifier) = &heritage.expression else {
                continue;
            };
            if !seen.insert(identifier.name.as_str()) {
                continue;
            }
            if let Some(inherited) = self.lookup(identifier.name.as_str(), depth - 1, block) {
                extend_unique(&mut props, inherited);
            }
        }
        props
    }
}

fn extend_unique(props: &mut Vec<Prop>, incoming: Vec<Prop>) {
    for prop in incoming {
        if props
            .iter()
            .any(|existing| existing.block == prop.block && existing.span == prop.span)
        {
            continue;
        }
        props.push(prop);
    }
}

/// eslint-plugin-vue reports the whole prop node, not its key.
fn named(key: &PropertyKey<'_>, node: Span, block: usize) -> Option<Prop> {
    let name = match key {
        PropertyKey::StaticIdentifier(identifier) => identifier.name.as_str(),
        PropertyKey::StringLiteral(literal) => literal.value.as_str(),
        _ => return None,
    };
    Some(Prop {
        name: String::from(name),
        span: node,
        anchor: node.start.min(key.span().start),
        block,
    })
}

fn has_jsdoc(source: &str, comments: &[oxc_ast::ast::Comment], start: u32) -> bool {
    let Some(comment) = comments
        .iter()
        .rev()
        .find(|comment| comment.span.end <= start)
    else {
        return false;
    };
    let gap = source
        .get(comment.span.end as usize..start as usize)
        .unwrap_or("");
    if gap.chars().any(|ch| !ch.is_whitespace()) || !comment.is_block() {
        return false;
    }
    let content = comment.content_span();
    source.as_bytes().get(content.start as usize).copied() == Some(b'*')
}

fn callee_name<'a>(callee: &'a Expression<'a>) -> Option<&'a str> {
    match callee.get_inner_expression() {
        Expression::Identifier(identifier) => Some(identifier.name.as_str()),
        _ => None,
    }
}

fn property_name<'a>(key: &'a PropertyKey<'a>) -> Option<&'a str> {
    match key {
        PropertyKey::StaticIdentifier(identifier) => Some(identifier.name.as_str()),
        PropertyKey::StringLiteral(literal) => Some(literal.value.as_str()),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
