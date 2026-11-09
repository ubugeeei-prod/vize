//! Lint-template projection from the completed Davinci L1/L2 artifact.
//!
//! The existing template drawer still owns the compatibility semantics. This
//! module supplies its short-lived node view without running Armature again:
//! structure and authored coordinates come from L1, while namespace and
//! retained JavaScript expressions come from L2.

use vize_carton::FxHashMap;
use vize_l0::{Allocator, Box, Span, Vec};
use vize_l1::markup::directive::{ArgSyntax, DirectivePrefix, DirectiveSyntax, VueDirectives};
use vize_l1::markup::{Attribute, Element, SurfaceChild, SurfaceTree};
use vize_l2::{expr::ExprRef, op::Region};
use vize_relief::{
    AttributeNode, CommentNode, DirectiveNode, ElementNode, ElementType, ExpressionNode,
    InterpolationNode, Namespace, PropNode, RootNode, SimpleExpressionNode, SourceLocation,
    TemplateChildNode, TextNode,
};

use super::Drawer;

mod index;
use index::{ElementIdentity, collect_region};

mod compatibility;
use compatibility::{comment_body, element_type, is_v_pre_head, surface_namespace};

#[cfg(test)]
mod tests;

/// One completed template artifact for lint semantic analysis.
#[derive(Clone, Copy)]
pub struct LintTemplate<'t, 'a> {
    surface: &'t SurfaceTree<'a>,
    allocator: &'a Allocator,
    root: &'t Region<'a>,
    consumed_expressions: &'t [ExprRef<'a>],
    legacy_slot_bindings: bool,
}

impl<'t, 'a> LintTemplate<'t, 'a> {
    /// Pair the lossless L1 surface with the L2 lowering produced from it.
    #[must_use]
    pub const fn new(
        surface: &'t SurfaceTree<'a>,
        allocator: &'a Allocator,
        root: &'t Region<'a>,
    ) -> Self {
        Self {
            surface,
            allocator,
            root,
            consumed_expressions: &[],
            legacy_slot_bindings: false,
        }
    }

    /// Attach lowering facts that are not owned by neutral L2.
    ///
    /// Keeping these borrowed facts separate preserves Croquis's parse-only
    /// dependency boundary while allowing the compatibility projection to
    /// retain consumed expressions and dialect-specific slot syntax.
    #[doc(hidden)]
    #[must_use]
    #[inline]
    pub const fn with_lowering_facts(
        mut self,
        consumed_expressions: &'t [ExprRef<'a>],
        legacy_slot_bindings: bool,
    ) -> Self {
        self.consumed_expressions = consumed_expressions;
        self.legacy_slot_bindings = legacy_slot_bindings;
        self
    }

    /// Original template source shared by both stages.
    #[must_use]
    pub const fn source(self) -> &'a str {
        self.surface.source
    }

    /// Borrow the compatibility template view used by consumers that have not
    /// yet moved their emission APIs from Relief to the Davinci artifact.
    ///
    /// The view is projected from the resident L1/L2 pair; this never runs the
    /// template parser again. Keeping the root inside the callback also makes
    /// its intentionally short lifetime explicit.
    #[doc(hidden)]
    #[inline]
    pub fn with_compatibility_root<R>(self, use_root: impl FnOnce(&RootNode<'a>) -> R) -> R {
        let root = CompatibilityRoot::new(self).build();
        use_root(&root)
    }

    pub(super) fn with_compatibility_syntax<R>(
        self,
        use_syntax: impl FnOnce(
            &RootNode<'a>,
            &super::template_references::lint_syntax::LintSyntaxFacts<'a>,
        ) -> R,
    ) -> R {
        let projection = CompatibilityRoot::new(self);
        let root = projection.build();
        let syntax =
            super::template_references::lint_syntax::LintSyntaxFacts::collect(&root, &projection);
        use_syntax(&root, &syntax)
    }
}

impl Drawer {
    /// Draw lint facts from L1/L2 without a second template parse.
    #[inline]
    pub fn draw_lint_template(&mut self, template: LintTemplate<'_, '_>) -> &mut Self {
        template.with_compatibility_root(|root| self.draw_template(root));
        self
    }
}

pub(super) struct CompatibilityRoot<'t, 'a> {
    allocator: &'a Allocator,
    surface: &'t SurfaceTree<'a>,
    expressions: FxHashMap<(u32, u32), ExprRef<'a>>,
    elements: FxHashMap<u32, ElementIdentity>,
    legacy_slot_bindings: bool,
}

impl<'t, 'a> CompatibilityRoot<'t, 'a> {
    fn new(template: LintTemplate<'t, 'a>) -> Self {
        let mut expressions = std::vec::Vec::new();
        let mut elements = std::vec::Vec::new();
        collect_region(template.root, &mut expressions, &mut elements);
        let mut expression_index = FxHashMap::default();
        expression_index.reserve(expressions.len());
        for expression in expressions {
            let span = expression.span();
            expression_index
                .entry((span.start, span.end))
                .or_insert(expression);
        }
        for expression in template.consumed_expressions {
            let expression = *expression;
            let span = expression.span();
            expression_index
                .entry((span.start, span.end))
                .or_insert(expression);
        }
        let mut element_index = FxHashMap::default();
        element_index.reserve(elements.len());
        for identity in elements {
            element_index.entry(identity.start).or_insert(identity);
        }
        Self {
            allocator: template.allocator,
            surface: template.surface,
            expressions: expression_index,
            elements: element_index,
            legacy_slot_bindings: template.legacy_slot_bindings,
        }
    }

    pub(super) const fn allocator(&self) -> &'a Allocator {
        self.allocator
    }

    pub(super) const fn legacy_slot_bindings(&self) -> bool {
        self.legacy_slot_bindings
    }

    pub(super) fn js_expression(
        &self,
        span: Span,
        cooked: &str,
    ) -> Option<&'a vize_l2::expr::JsExpr<'a>> {
        match self.expressions.get(&(span.start, span.end)).copied() {
            Some(ExprRef::Js(expression)) if expression.source == cooked => Some(expression),
            _ => None,
        }
    }

    fn build(&self) -> RootNode<'a> {
        let mut root = RootNode::new(self.allocator, self.surface.source);
        self.push_children(
            &mut root.children,
            &self.surface.children,
            false,
            Namespace::Html,
            None,
        );
        root
    }

    fn push_children(
        &self,
        output: &mut Vec<'a, TemplateChildNode<'a>>,
        children: &[SurfaceChild<'a>],
        inherited_frozen: bool,
        parent_namespace: Namespace,
        parent_tag: Option<&str>,
    ) {
        for child in children {
            let node = match child {
                SurfaceChild::Element(element) => {
                    if element.open.is_ignored_start() {
                        self.push_children(
                            output,
                            &element.children,
                            inherited_frozen,
                            parent_namespace,
                            parent_tag,
                        );
                        continue;
                    }
                    TemplateChildNode::Element(Box::new_in(
                        vize_l0::ensure_sufficient_stack(|| {
                            self.element(element, inherited_frozen, parent_namespace, parent_tag)
                        }),
                        &self.allocator,
                    ))
                }
                SurfaceChild::Text(token) => TemplateChildNode::Text(Box::new_in(
                    TextNode::new(self.decoded_text(token.text), self.token_loc(token)),
                    &self.allocator,
                )),
                SurfaceChild::Comment(token) => TemplateChildNode::Comment(Box::new_in(
                    CommentNode::new(comment_body(token.text), self.token_loc(token)),
                    &self.allocator,
                )),
                SurfaceChild::Interpolation(interpolation) => {
                    if inherited_frozen {
                        let span = Span::new(
                            self.offset(interpolation.open.text),
                            self.offset(interpolation.close.text)
                                + interpolation.close.text.len() as u32,
                        );
                        let content = self
                            .surface
                            .source
                            .get(span.start as usize..span.end as usize)
                            .unwrap_or_default();
                        output.push(TemplateChildNode::Text(Box::new_in(
                            TextNode::new(content, SourceLocation { span }),
                            &self.allocator,
                        )));
                        continue;
                    }
                    let raw = interpolation.content.text;
                    let trimmed = raw.trim();
                    let leading = raw.len().saturating_sub(raw.trim_start().len()) as u32;
                    let start = self.offset(interpolation.content.text) + leading;
                    let span = Span::new(start, start + trimmed.len() as u32);
                    let expression = self.expression(trimmed, span, false);
                    let loc = SourceLocation {
                        span: Span::new(
                            self.offset(interpolation.open.text),
                            self.offset(interpolation.close.text)
                                + interpolation.close.text.len() as u32,
                        ),
                    };
                    TemplateChildNode::Interpolation(Box::new_in(
                        InterpolationNode::new(expression, loc, interpolation.is_raw_html()),
                        &self.allocator,
                    ))
                }
                SurfaceChild::Cdata(_)
                | SurfaceChild::ProcessingInstruction(_)
                | SurfaceChild::Unexpected(_) => continue,
            };
            output.push(node);
        }
    }

    fn element(
        &self,
        surface: &Element<'a>,
        inherited_frozen: bool,
        parent_namespace: Namespace,
        parent_tag: Option<&str>,
    ) -> ElementNode<'a> {
        let start = self.offset(surface.open.lt_name.text);
        let end = self.offset(surface.open.gt.text) + surface.open.gt.text.len() as u32;
        let authored_tag = surface.tag();
        let identity = self.elements.get(&start).copied();
        let mut element = ElementNode::new(
            self.allocator,
            authored_tag,
            SourceLocation {
                span: Span::new(start, end),
            },
        );
        let structural_namespace = surface_namespace(parent_namespace, parent_tag, authored_tag);
        element.ns = if structural_namespace == Namespace::Html {
            identity.map_or(Namespace::Html, |identity| identity.namespace)
        } else {
            structural_namespace
        };

        let frozen = inherited_frozen
            || surface.open.is_verbatim()
            || surface
                .open
                .attrs
                .iter()
                .any(|attribute| is_v_pre_head(attribute.name.text));
        element.tag_type = element_type(surface, frozen, element.ns);
        element.is_self_closing = surface.open.slash.is_some()
            && !(element.ns == vize_relief::Namespace::Html
                && element.tag_type == ElementType::Element
                && !vize_l0::is_void_tag(authored_tag));
        for attribute in &surface.open.attrs {
            if let Some(prop) = self.prop(attribute, frozen) {
                element.props.push(prop);
            }
        }
        self.push_children(
            &mut element.children,
            &surface.children,
            frozen,
            element.ns,
            Some(authored_tag),
        );
        element
    }

    fn prop(&self, attribute: &Attribute<'a>, frozen: bool) -> Option<PropNode<'a>> {
        let name_start = self.offset(attribute.name.text);
        let decomposition = VueDirectives
            .decompose(attribute.name.text, name_start)
            .ok()
            .flatten();
        if frozen {
            if decomposition.is_some_and(|name| {
                name.prefix == DirectivePrefix::Full
                    && name.name.slice(self.surface.source) == "pre"
            }) {
                return None;
            }
            return Some(PropNode::Attribute(Box::new_in(
                self.attribute(attribute),
                &self.allocator,
            )));
        }
        match decomposition {
            Some(name) => self
                .directive(attribute, name)
                .map(|directive| PropNode::Directive(Box::new_in(directive, &self.allocator))),
            None => Some(PropNode::Attribute(Box::new_in(
                self.attribute(attribute),
                &self.allocator,
            ))),
        }
    }

    fn attribute(&self, attribute: &Attribute<'a>) -> AttributeNode<'a> {
        let span = self.attribute_span(attribute);
        let mut node = AttributeNode::new(attribute.name.text, SourceLocation { span });
        node.name_loc = SourceLocation {
            span: self.token_span(&attribute.name),
        };
        node.value = attribute.value.as_ref().map(|value| {
            let span = self.token_span(&value.content);
            TextNode::new(self.decoded_value(value), SourceLocation { span })
        });
        node
    }

    fn directive(
        &self,
        attribute: &Attribute<'a>,
        decomposition: vize_l1::markup::directive::DirectiveName,
    ) -> Option<DirectiveNode<'a>> {
        let name = match decomposition.prefix {
            DirectivePrefix::Full => decomposition.name.slice(self.surface.source),
            DirectivePrefix::Bind | DirectivePrefix::Prop => "bind",
            DirectivePrefix::On => "on",
            DirectivePrefix::Slot => "slot",
        };
        if name.is_empty() {
            return None;
        }
        let mut node = DirectiveNode::new(
            self.allocator,
            name,
            SourceLocation {
                span: self.attribute_span(attribute),
            },
        );
        node.raw_name = Some(match decomposition.prefix {
            DirectivePrefix::Full => attribute
                .name
                .text
                .split_once([':', '.'])
                .map_or(attribute.name.text, |(head, _)| head),
            DirectivePrefix::Bind => ":",
            DirectivePrefix::Prop => ".",
            DirectivePrefix::On => "@",
            DirectivePrefix::Slot => "#",
        });
        node.arg = decomposition.arg.map(|argument| {
            let (span, is_static) = match argument {
                ArgSyntax::Static(span) => (span, true),
                ArgSyntax::Dynamic(span) => (span, false),
            };
            self.expression(span.slice(self.surface.source), span, is_static)
        });
        if decomposition.prefix == DirectivePrefix::Prop {
            node.modifiers.push(SimpleExpressionNode::new(
                "prop",
                true,
                SourceLocation {
                    span: Span::new(decomposition.modifiers.start, decomposition.modifiers.start),
                },
            ));
        }
        self.push_modifiers(&mut node.modifiers, decomposition.modifiers);

        if let Some(value) = attribute.value.as_ref() {
            let span = self.token_span(&value.content);
            let mut expression = self.expression(self.decoded_value(value), span, false);
            if name == "slot"
                && let ExpressionNode::Simple(expression) = &mut expression
            {
                expression.js_ast = Some(vize_armature::retain_slot_parameters_in(
                    self.allocator,
                    expression.content,
                ));
            }
            node.exp = Some(expression);
        } else if name == "bind"
            && let Some(ExpressionNode::Simple(argument)) = node.arg.as_ref()
            && argument.is_static
        {
            let content = self
                .allocator
                .alloc_str(vize_l0::camelize(argument.content).as_str());
            node.exp = Some(self.expression(content, argument.loc.span, false));
            node.shorthand = true;
        }
        Some(node)
    }

    fn push_modifiers(&self, output: &mut Vec<'a, SimpleExpressionNode<'a>>, span: Span) {
        let authored = span.slice(self.surface.source);
        let mut cursor = span.start;
        for part in authored.split('.') {
            if part.is_empty() {
                cursor = cursor.saturating_add(1);
                continue;
            }
            let start = cursor;
            let end = start + part.len() as u32;
            output.push(SimpleExpressionNode::new(
                part,
                true,
                SourceLocation {
                    span: Span::new(start, end),
                },
            ));
            cursor = end.saturating_add(1);
        }
    }
}
