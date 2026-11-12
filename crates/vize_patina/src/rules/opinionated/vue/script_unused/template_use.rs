//! Template names that read or reassign a script binding.
//!
//! Only free names count. A `v-for` or slot alias shadows a script binding of
//! the same name. `counter++` is a read and a write. `counter.foo = 1` reads
//! `counter` and does not reassign it.

use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::SourceType;
use vize_croquis::drawer::{extract_slot_props, parse_v_for_scope_expression};
use vize_l0::{CompactString, FxHashSet, cstr};
use vize_relief::{
    ElementNode, ElementType, ExpressionNode, PropNode, RootNode, TemplateChildNode,
};

pub(super) struct TemplateUse {
    pub(super) reads: FxHashSet<CompactString>,
    pub(super) writes: FxHashSet<CompactString>,
}

impl TemplateUse {
    pub(super) fn empty() -> Self {
        Self {
            reads: FxHashSet::default(),
            writes: FxHashSet::default(),
        }
    }
}

pub(super) fn template_use(root: &RootNode<'_>) -> TemplateUse {
    let mut uses = TemplateUse::empty();
    let mut shadow = Vec::new();
    walk_children(&root.children, root.source, &mut shadow, &mut uses);
    uses
}

pub(super) fn walk_children(
    children: &[TemplateChildNode<'_>],
    source: &str,
    shadow: &mut Vec<CompactString>,
    uses: &mut TemplateUse,
) {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => walk_element(element, source, shadow, uses),
            TemplateChildNode::Interpolation(interpolation) => {
                note_expression(
                    expression_source(&interpolation.content, source),
                    shadow,
                    uses,
                );
            }
            TemplateChildNode::If(if_node) => {
                for branch in if_node.branches.iter() {
                    if let Some(condition) = branch.condition.as_ref() {
                        note_expression(expression_source(condition, source), shadow, uses);
                    }
                    walk_children(&branch.children, source, shadow, uses);
                }
            }
            TemplateChildNode::For(for_node) => {
                let mark = shadow.len();
                note_expression(expression_source(&for_node.source, source), shadow, uses);
                for alias in [
                    for_node.value_alias.as_ref(),
                    for_node.key_alias.as_ref(),
                    for_node.object_index_alias.as_ref(),
                ]
                .into_iter()
                .flatten()
                {
                    push_alias(expression_source(alias, source), shadow);
                }
                walk_children(&for_node.children, source, shadow, uses);
                shadow.truncate(mark);
            }
            _ => {}
        }
    }
}

fn walk_element(
    element: &ElementNode<'_>,
    source: &str,
    shadow: &mut Vec<CompactString>,
    uses: &mut TemplateUse,
) {
    if has_v_pre(element) {
        return;
    }
    // A script-setup component is used when the template names it as a tag.
    // `<router-view>` resolves the same binding as `<RouterView>`. Inside
    // `<svg>` a non-native tag such as `<Swatch>` still resolves a component.
    if element.tag_type == ElementType::Component
        || (element.tag_type == ElementType::Element && !vize_l0::is_native_tag(element.tag))
    {
        note_component_tag(element.tag, uses);
    }
    let mark = shadow.len();
    for prop in element.props.iter() {
        match prop {
            // `ref="panel"` fills the script binding `panel`.
            PropNode::Attribute(attribute) if attribute.name == "ref" => {
                if let Some(value) = attribute.value.as_ref() {
                    note_ref_name(value.content, uses);
                }
            }
            // `v-focus` resolves the script binding `vFocus`.
            PropNode::Directive(directive) if !is_builtin_directive(directive.name) => {
                note_directive(directive.name, uses);
            }
            _ => {}
        }
    }
    for prop in element.props.iter() {
        let PropNode::Directive(directive) = prop else {
            continue;
        };
        let Some(exp) = directive.exp.as_ref() else {
            continue;
        };
        let content = expression_source(exp, source);
        if directive.name == "for" {
            if let Some(parsed) = parse_v_for_scope_expression(content) {
                note_expression(parsed.source.as_str(), shadow, uses);
                for alias in parsed.value_bindings {
                    shadow.push(alias);
                }
                if let Some(alias) = parsed.key_alias {
                    shadow.push(alias);
                }
                if let Some(alias) = parsed.index_alias {
                    shadow.push(alias);
                }
            } else {
                note_expression(content, shadow, uses);
            }
        } else if directive.name == "slot" {
            for alias in extract_slot_props(content) {
                shadow.push(alias);
            }
        }
    }
    for prop in element.props.iter() {
        let PropNode::Directive(directive) = prop else {
            continue;
        };
        if let Some(argument) = directive.arg.as_ref()
            && !is_static(argument)
        {
            note_expression(expression_source(argument, source), shadow, uses);
        }
        if matches!(directive.name, "for" | "slot") {
            continue;
        }
        if let Some(exp) = directive.exp.as_ref() {
            note_expression(expression_source(exp, source), shadow, uses);
        }
    }
    walk_children(&element.children, source, shadow, uses);
    shadow.truncate(mark);
}

fn note_component_tag(tag: &str, uses: &mut TemplateUse) {
    if tag.is_empty() {
        return;
    }
    // Vue resolves a tag through the raw name, its camelCase form, and the
    // capitalized form. `<kebab>` and `<router-view>` both use `Kebab` / `RouterView`.
    let mut names = vec![CompactString::new(tag)];
    let mut camel = CompactString::default();
    let mut upper = false;
    for ch in tag.chars() {
        if ch == '-' {
            upper = true;
            continue;
        }
        if upper {
            camel.extend(ch.to_uppercase());
            upper = false;
        } else {
            camel.push(ch);
        }
    }
    if camel != tag {
        names.push(camel.clone());
    }
    let mut chars = camel.chars();
    if let Some(first) = chars.next() {
        let mut pascal = CompactString::default();
        pascal.extend(first.to_uppercase());
        pascal.push_str(chars.as_str());
        if pascal != tag && pascal != camel {
            names.push(pascal);
        }
    }
    uses.reads.extend(names);
}

fn is_builtin_directive(name: &str) -> bool {
    matches!(
        name,
        "bind"
            | "on"
            | "if"
            | "else"
            | "else-if"
            | "for"
            | "show"
            | "model"
            | "slot"
            | "text"
            | "html"
            | "cloak"
            | "once"
            | "pre"
            | "memo"
            | "is"
    )
}

/// `v-focus` and `v-my-dir` resolve `vFocus` and `vMyDir`.
fn note_directive(name: &str, uses: &mut TemplateUse) {
    if name.is_empty() {
        return;
    }
    let mut binding = CompactString::new("v");
    let mut upper = true;
    for ch in name.chars() {
        if ch == '-' {
            upper = true;
            continue;
        }
        if upper {
            binding.extend(ch.to_uppercase());
            upper = false;
        } else {
            binding.push(ch);
        }
    }
    uses.reads.insert(binding);
}

fn note_ref_name(value: &str, uses: &mut TemplateUse) {
    let value = value.trim();
    let mut chars = value.chars();
    let valid = chars
        .next()
        .is_some_and(|first| first.is_alphabetic() || first == '_' || first == '$')
        && chars.all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '$');
    if valid {
        uses.reads.insert(CompactString::new(value));
    }
}

fn has_v_pre(element: &ElementNode<'_>) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Attribute(attribute) => attribute.name == "v-pre",
        PropNode::Directive(directive) => directive.name == "pre",
    })
}

fn is_static(exp: &ExpressionNode<'_>) -> bool {
    matches!(exp, ExpressionNode::Simple(simple) if simple.is_static)
}

fn expression_source<'a>(exp: &'a ExpressionNode<'a>, source: &'a str) -> &'a str {
    match exp {
        ExpressionNode::Simple(simple) => simple.content,
        ExpressionNode::Compound(compound) => compound.loc.span.slice(source),
    }
}

fn push_alias(source: &str, shadow: &mut Vec<CompactString>) {
    let name = source.trim();
    if !name.is_empty() {
        shadow.push(CompactString::new(name));
    }
}

pub(super) fn note_expression(source: &str, shadow: &[CompactString], uses: &mut TemplateUse) {
    let source = source.trim();
    if source.is_empty() {
        return;
    }
    let allocator = Allocator::default();
    let wrapped = cstr!("({source})");
    let wrapped_parse = Parser::new(&allocator, &wrapped, SourceType::tsx()).parse();
    let wrapped_ok = !wrapped_parse.panicked && wrapped_parse.diagnostics.is_empty();
    let parsed = if wrapped_ok {
        wrapped_parse
    } else {
        Parser::new(&allocator, source, SourceType::tsx()).parse()
    };
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return;
    }
    let built = SemanticBuilder::new()
        .with_check_syntax_error(true)
        .with_build_nodes(true)
        .build(&parsed.program);
    if !built.diagnostics.is_empty() {
        return;
    }
    let semantic = &built.semantic;
    let scoping = semantic.scoping();
    for ids in scoping.root_unresolved_references_ids() {
        for id in ids {
            let reference = scoping.get_reference(id);
            let name = semantic.reference_name(reference);
            if shadow.iter().any(|alias| alias.as_str() == name) {
                continue;
            }
            let flags = reference.flags();
            if flags.is_write() {
                uses.writes.insert(CompactString::new(name));
            }
            if flags.is_read() || flags.is_type() || flags.is_value_as_type() {
                uses.reads.insert(CompactString::new(name));
            }
        }
    }
}
