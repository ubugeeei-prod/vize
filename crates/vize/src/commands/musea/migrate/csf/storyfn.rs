use oxc_ast::ast::{
    AssignmentTarget, Declaration, Expression, ObjectExpression, ObjectPropertyKind, Program,
    Statement,
};
use oxc_syntax::operator::AssignmentOperator;

use super::{
    CsfRender, CsfStory, binding_name, property_key_name, render_body, story_from_object,
    unsupported_story, unwrap_expression, unwrap_object,
};

pub(super) fn collect_stories<'a>(program: &'a Program<'a>) -> Vec<CsfStory<'a>> {
    let templates = collect_template_renders(program);
    let story_args = collect_story_args(program);
    let mut stories = Vec::new();

    for stmt in &program.body {
        let Statement::ExportNamedDeclaration(decl) = stmt else {
            continue;
        };
        let Some(Declaration::VariableDeclaration(var)) = decl.declaration.as_ref() else {
            continue;
        };
        for declarator in &var.declarations {
            let Some(export_name) = binding_name(declarator) else {
                continue;
            };
            let Some(init) = declarator.init.as_ref() else {
                continue;
            };
            let assigned_args = find_object_by_name(&story_args, export_name);
            let story = if let Some(object) = unwrap_object(init) {
                if !is_story_object(object) {
                    continue;
                }
                let mut story = story_from_object(export_name, object);
                if story.args.is_none() {
                    story.args = assigned_args;
                }
                story
            } else {
                story_from_template_bind(export_name, init, &templates, assigned_args)
                    .unwrap_or_else(|| unsupported_story(export_name))
            };
            stories.push(story);
        }
    }

    stories
}

fn is_story_object(object: &ObjectExpression<'_>) -> bool {
    object.properties.is_empty()
        || object.properties.iter().any(|property| match property {
            ObjectPropertyKind::SpreadProperty(_) => true,
            ObjectPropertyKind::ObjectProperty(prop) => {
                !prop.computed && property_key_name(&prop.key).is_some_and(is_story_annotation_key)
            }
        })
}

fn is_story_annotation_key(key: &str) -> bool {
    matches!(
        key,
        "args"
            | "argTypes"
            | "beforeEach"
            | "decorators"
            | "experimental_afterEach"
            | "globals"
            | "loaders"
            | "name"
            | "parameters"
            | "play"
            | "render"
            | "storyName"
            | "tags"
    )
}

fn collect_template_renders<'a>(program: &'a Program<'a>) -> Vec<(&'a str, CsfRender<'a>)> {
    let mut templates = Vec::new();
    for stmt in &program.body {
        let Statement::VariableDeclaration(decl) = stmt else {
            continue;
        };
        for declarator in &decl.declarations {
            if let Some(name) = binding_name(declarator)
                && let Some(render) = declarator.init.as_ref().and_then(render_body)
            {
                templates.push((name, render));
            }
        }
    }
    templates
}

fn collect_story_args<'a>(program: &'a Program<'a>) -> Vec<(&'a str, &'a ObjectExpression<'a>)> {
    program
        .body
        .iter()
        .filter_map(story_args_assignment)
        .collect()
}

fn story_args_assignment<'a>(
    stmt: &'a Statement<'a>,
) -> Option<(&'a str, &'a ObjectExpression<'a>)> {
    let Statement::ExpressionStatement(stmt) = stmt else {
        return None;
    };
    let Expression::AssignmentExpression(assignment) = unwrap_expression(&stmt.expression) else {
        return None;
    };
    if assignment.operator != AssignmentOperator::Assign {
        return None;
    }
    let story_name = static_member_target_name(&assignment.left, "args")?;
    let args = unwrap_object(&assignment.right)?;
    Some((story_name, args))
}

fn story_from_template_bind<'a>(
    export_name: &str,
    init: &'a Expression<'a>,
    templates: &[(&'a str, CsfRender<'a>)],
    args: Option<&'a ObjectExpression<'a>>,
) -> Option<CsfStory<'a>> {
    let template_name = template_bind_name(init)?;
    let render = find_expression_by_name(templates, template_name);
    Some(CsfStory {
        name: export_name.into(),
        render,
        args,
        unsupported: render.is_none(),
    })
}

fn template_bind_name<'a>(expr: &'a Expression<'a>) -> Option<&'a str> {
    let Expression::CallExpression(call) = unwrap_expression(expr) else {
        return None;
    };
    let Expression::StaticMemberExpression(member) = &call.callee else {
        return None;
    };
    if member.property.name != "bind" {
        return None;
    }
    if let Expression::Identifier(ident) = unwrap_expression(&member.object) {
        Some(ident.name.as_str())
    } else {
        None
    }
}

fn static_member_target_name<'a>(
    target: &'a AssignmentTarget<'a>,
    property: &str,
) -> Option<&'a str> {
    let AssignmentTarget::StaticMemberExpression(member) = target else {
        return None;
    };
    if member.property.name != property {
        return None;
    }
    if let Expression::Identifier(ident) = unwrap_expression(&member.object) {
        Some(ident.name.as_str())
    } else {
        None
    }
}

fn find_expression_by_name<'a>(
    items: &[(&'a str, CsfRender<'a>)],
    name: &str,
) -> Option<CsfRender<'a>> {
    items
        .iter()
        .rev()
        .find_map(|(item_name, value)| (*item_name == name).then_some(*value))
}

fn find_object_by_name<'a>(
    items: &[(&'a str, &'a ObjectExpression<'a>)],
    name: &str,
) -> Option<&'a ObjectExpression<'a>> {
    items
        .iter()
        .rev()
        .find_map(|(item_name, value)| (*item_name == name).then_some(*value))
}
