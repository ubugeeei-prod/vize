use vize_carton::String;
use vize_croquis::{CroquisSemanticSnapshot, CroquisSemanticSnapshotBuilder, SemanticSourceRange};
use vize_relief::{
    ElementType, ReliefSnapshot, ReliefSnapshotNode, SnapshotCompoundChild,
    SnapshotCompoundExpression, SnapshotExpression, SnapshotProp, SnapshotTextCallContent,
};

pub(super) fn project_template_semantics(
    script: CroquisSemanticSnapshot,
    relief: &ReliefSnapshot,
    offset: u32,
) -> CroquisSemanticSnapshot {
    let mut builder = CroquisSemanticSnapshotBuilder::from_snapshot(script);
    for node in relief.nodes() {
        match node {
            ReliefSnapshotNode::Element(element) => {
                if element.tag_type == ElementType::Component {
                    builder.add_component_usage(
                        &element.tag,
                        absolute_range(&element.location, offset),
                        0,
                        element.props.iter().any(|property| {
                            matches!(property, SnapshotProp::Directive(directive) if directive.name == "bind" && directive.argument.is_none())
                        }),
                    );
                }
                for property in &element.props {
                    if let SnapshotProp::Directive(directive) = property
                        && let Some(expression) = &directive.expression
                    {
                        add_expression(&mut builder, expression, "vue-directive", offset);
                    }
                }
            }
            ReliefSnapshotNode::Interpolation(interpolation) => {
                add_expression(
                    &mut builder,
                    &interpolation.content,
                    "interpolation",
                    offset,
                );
            }
            ReliefSnapshotNode::IfBranch(branch) => {
                if let Some(condition) = &branch.condition {
                    add_expression(&mut builder, condition, "v-if", offset);
                }
            }
            ReliefSnapshotNode::For(node) => {
                add_expression(&mut builder, &node.source, "v-for", offset);
                add_for_bindings(&mut builder, node, offset);
            }
            ReliefSnapshotNode::TextCall(call) => match &call.content {
                SnapshotTextCallContent::Interpolation(interpolation) => add_expression(
                    &mut builder,
                    &interpolation.content,
                    "interpolation",
                    offset,
                ),
                SnapshotTextCallContent::Compound(compound) => {
                    add_compound(&mut builder, compound, "compound", offset)
                }
                SnapshotTextCallContent::Text(_) => {}
            },
            ReliefSnapshotNode::CompoundExpression(compound) => {
                add_compound(&mut builder, compound, "compound", offset);
            }
            ReliefSnapshotNode::Text(_)
            | ReliefSnapshotNode::Comment(_)
            | ReliefSnapshotNode::If(_)
            | ReliefSnapshotNode::Hoisted(_) => {}
        }
    }
    builder.finish()
}

fn add_for_bindings(
    builder: &mut CroquisSemanticSnapshotBuilder,
    node: &vize_relief::SnapshotFor,
    offset: u32,
) {
    for binding in [
        node.value_alias.as_ref(),
        node.key_alias.as_ref(),
        node.object_index_alias.as_ref(),
    ]
    .into_iter()
    .flatten()
    {
        builder.add_binding(
            expression_code(binding).as_str(),
            "iteration",
            "template-local",
            Some(absolute_range(binding.location(), offset)),
        );
    }
}

fn add_expression(
    builder: &mut CroquisSemanticSnapshotBuilder,
    expression: &SnapshotExpression,
    kind: &'static str,
    offset: u32,
) {
    builder.add_template_expression(
        expression_code(expression).as_str(),
        kind,
        absolute_range(expression.location(), offset),
        0,
    );
}

fn add_compound(
    builder: &mut CroquisSemanticSnapshotBuilder,
    compound: &SnapshotCompoundExpression,
    kind: &'static str,
    offset: u32,
) {
    builder.add_template_expression(
        compound_code(compound).as_str(),
        kind,
        absolute_range(&compound.location, offset),
        0,
    );
}

fn expression_code(expression: &SnapshotExpression) -> String {
    match expression {
        SnapshotExpression::Simple(simple) => simple.content.clone(),
        SnapshotExpression::Compound(compound) => compound_code(compound),
    }
}

fn compound_code(compound: &SnapshotCompoundExpression) -> String {
    let mut code = String::new("");
    for child in &compound.children {
        match child {
            SnapshotCompoundChild::Simple(simple) => code.push_str(&simple.content),
            SnapshotCompoundChild::Compound(nested) => code.push_str(&compound_code(nested)),
            SnapshotCompoundChild::Interpolation(interpolation) => {
                code.push_str(&expression_code(&interpolation.content));
            }
            SnapshotCompoundChild::Text(text) => code.push_str(&text.content),
            SnapshotCompoundChild::String(value) => code.push_str(value),
            SnapshotCompoundChild::Symbol(symbol) => code.push_str(symbol.name()),
        }
    }
    code
}

fn absolute_range(location: &vize_relief::SourceLocation, offset: u32) -> SemanticSourceRange {
    SemanticSourceRange::new(
        location.start.offset.saturating_add(offset),
        location.end.offset.saturating_add(offset),
    )
}
