//! Allocate imports against real authored namespaces, never source spelling.

use oxc_ast::ast::{BindingIdentifier, IdentifierReference, TSType};
use oxc_ast_visit::Visit;
use oxc_parser::{ParseOptions, Parser};
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use oxc_syntax::symbol::SymbolFlags;
use vize_armature::{ExpressionNode, PropNode, TemplateChildNode};
use vize_l0::{
    Allocator, SmallVec, String, expression_guard::expression_is_safe_to_parse,
    profiler::global_profiler,
};

use super::{TemplateAssetUrl, push_usize};
use crate::types::SfcDescriptor;

/// A spelling check only admits work. Cooked AST names are the authority: an
/// identifier can be escaped, while a comment, string or property is not a name.
fn may_reference_import_name(source: &str) -> bool {
    source.contains("_imports_") || source.contains("\\u")
}

#[derive(Default)]
struct Names {
    indices: SmallVec<[usize; 8]>,
}

impl Names {
    fn reserve(&mut self, name: &str) {
        let Some(suffix) = name.strip_prefix("_imports_") else {
            return;
        };
        // `_imports_00` and `_imports_0` are distinct bindings.
        if suffix.is_empty()
            || (suffix.len() > 1 && suffix.starts_with('0'))
            || !suffix.bytes().all(|byte| byte.is_ascii_digit())
        {
            return;
        }
        if let Ok(index) = suffix.parse() {
            self.indices.push(index);
        }
    }

    fn script(&mut self, allocator: &Allocator, source: &str, lang: Option<&str>) {
        if !may_reference_import_name(source) {
            return;
        }
        let source_type = match lang {
            Some("ts") => SourceType::ts(),
            Some("tsx") => SourceType::tsx(),
            Some("jsx") => SourceType::jsx(),
            _ => SourceType::mjs(),
        };
        global_profiler().record_counter("sfc.asset_names.script_parse", 1);
        let parsed = Parser::new(allocator.as_oxc(), source, source_type).parse();
        global_profiler().record_counter("sfc.asset_names.script_semantic", 1);
        let semantic = SemanticBuilder::new().build(&parsed.program).semantic;
        let scopes = semantic.scoping();
        for (name, symbol) in scopes.get_bindings(scopes.root_scope_id()) {
            let flags = scopes.symbol_flags(*symbol);
            if !flags.contains(SymbolFlags::TypeImport)
                && (flags.is_value() || flags.contains(SymbolFlags::Import))
            {
                self.reserve(name);
            }
        }
        // An added module binding must not capture a formerly global read or
        // write, including references inside nested functions.
        for (name, references) in scopes.root_unresolved_references() {
            if references
                .iter()
                .any(|reference| scopes.get_reference(*reference).flags().is_value())
            {
                self.reserve(name);
            }
        }
    }

    fn expression<'a>(&mut self, expression: &ExpressionNode<'a>) {
        let ExpressionNode::Simple(node) = expression else {
            return;
        };
        if node.is_static {
            return;
        }
        if let Some(retained) = node.js_ast {
            global_profiler().record_counter("sfc.asset_names.retained_ast", 1);
            if let Some(expression) = retained.as_expression() {
                self.visit_expression(expression.ast);
            } else if let Some(Ok(parameters)) = retained.as_slot_parameters() {
                self.visit_formal_parameters(parameters);
            }
        }
    }

    fn template<'a>(&mut self, allocator: &'a Allocator, children: &[TemplateChildNode<'a>]) {
        for child in children {
            match child {
                TemplateChildNode::Element(element) => {
                    for prop in &element.props {
                        let PropNode::Directive(directive) = prop else {
                            continue;
                        };
                        if let Some(argument) = &directive.arg {
                            self.expression(argument);
                        }
                        let Some(expression) = &directive.exp else {
                            continue;
                        };
                        if directive.name == "for" {
                            // The original for-head owns aliases and iterable;
                            // `of`, destructuring, and holes are not JS Expr.
                            if let ExpressionNode::Simple(node) = expression
                                && may_reference_import_name(node.content)
                                && let Some((aliases, collection)) = vize_croquis::drawer::retain_v_for_binding_parameters_with_source(allocator, node.content)
                            {
                                global_profiler().record_counter("sfc.asset_names.for_head", 1);
                                self.visit_formal_parameters(&aliases.parameters);
                                if may_reference_import_name(collection)
                                    && expression_is_safe_to_parse(collection)
                                {
                                    global_profiler().record_counter("sfc.asset_names.for_collection_parse", 1);
                                    if let Ok(expression) = Parser::new(allocator.as_oxc(), collection, SourceType::ts()).parse_expression() {
                                        self.visit_expression(&expression);
                                    }
                                }
                            }
                        } else {
                            self.expression(expression);
                            if directive.name == "on"
                                && let ExpressionNode::Simple(node) = expression
                                && node.js_ast.is_none()
                                && may_reference_import_name(node.content)
                                && expression_is_safe_to_parse(node.content)
                            {
                                // A statement handler has no retained Expr.
                                global_profiler()
                                    .record_counter("sfc.asset_names.handler_parse", 1);
                                let parsed =
                                    Parser::new(allocator.as_oxc(), node.content, SourceType::ts())
                                        .with_options(ParseOptions {
                                            allow_return_outside_function: true,
                                            ..ParseOptions::default()
                                        })
                                        .parse();
                                self.visit_program(&parsed.program);
                            }
                        }
                    }
                    self.template(allocator, &element.children);
                }
                TemplateChildNode::Interpolation(node) => self.expression(&node.content),
                _ => {}
            }
        }
    }
}

impl<'a> Visit<'a> for Names {
    fn visit_binding_identifier(&mut self, identifier: &BindingIdentifier<'a>) {
        self.reserve(identifier.name.as_str());
    }

    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        self.reserve(identifier.name.as_str());
    }

    fn visit_ts_type(&mut self, _ty: &TSType<'a>) {}
}

pub(super) fn avoid_authored_names(descriptor: &SfcDescriptor<'_>, urls: &mut [TemplateAssetUrl]) {
    let template = descriptor
        .template
        .as_ref()
        .map(|block| block.content.as_ref());
    let scripts = [descriptor.script.as_ref(), descriptor.script_setup.as_ref()];
    if !scripts
        .iter()
        .flatten()
        .any(|block| may_reference_import_name(&block.content))
        && !template.is_some_and(|source| may_reference_import_name(source) || source.contains('&'))
    {
        return;
    }
    global_profiler().record_counter("sfc.asset_names.namespace", 1);
    let allocator = Allocator::default();
    let mut names = Names::default();
    for block in scripts.into_iter().flatten() {
        names.script(&allocator, &block.content, block.lang.as_deref());
    }
    if let Some(source) = template
        && (may_reference_import_name(source) || source.contains('&'))
    {
        global_profiler().record_counter("sfc.asset_names.template_parse", 1);
        let (root, _) = vize_armature::Parser::new(&allocator, source).parse();
        names.template(&allocator, &root.children);
    }
    names.indices.sort_unstable();
    names.indices.dedup();
    let mut next = 0usize;
    let mut reserved = names.indices.into_iter().peekable();
    for (previous, asset) in urls.iter_mut().enumerate() {
        while reserved.peek().is_some_and(|index| *index <= next) {
            if reserved.next() == Some(next) {
                next += 1;
            }
        }
        if next != previous {
            let mut name = String::from("_imports_");
            push_usize(&mut name, next);
            asset.var_name = name;
            global_profiler().record_counter("sfc.asset_names.renamed_import", 1);
        }
        next += 1;
    }
}
