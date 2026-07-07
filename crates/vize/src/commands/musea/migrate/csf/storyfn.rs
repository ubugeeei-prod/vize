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
    let object_bindings = collect_object_bindings(program);
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
                let story_kind = classify_story_object(export_name, object, &object_bindings);
                if story_kind == StoryObjectKind::Fixture {
                    continue;
                }
                let mut story = story_from_object(export_name, object);
                story.unsupported |= story_kind == StoryObjectKind::Unsupported;
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

fn collect_object_bindings<'a>(
    program: &'a Program<'a>,
) -> Vec<(&'a str, &'a ObjectExpression<'a>)> {
    let mut bindings = Vec::new();
    for stmt in &program.body {
        let decl = match stmt {
            Statement::VariableDeclaration(decl) => decl,
            Statement::ExportNamedDeclaration(export) => {
                let Some(Declaration::VariableDeclaration(decl)) = export.declaration.as_ref()
                else {
                    continue;
                };
                decl
            }
            _ => continue,
        };
        for declarator in &decl.declarations {
            let Some(name) = binding_name(declarator) else {
                continue;
            };
            let Some(object) = declarator.init.as_ref().and_then(unwrap_object) else {
                continue;
            };
            bindings.push((name, object));
        }
    }
    bindings
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum StoryObjectKind {
    Fixture,
    Story,
    Unsupported,
}

fn classify_story_object<'a>(
    export_name: &str,
    object: &'a ObjectExpression<'a>,
    object_bindings: &[(&'a str, &'a ObjectExpression<'a>)],
) -> StoryObjectKind {
    classify_story_object_at_depth(export_name, object, object_bindings, 0)
}

fn classify_story_object_at_depth<'a>(
    export_name: &str,
    object: &'a ObjectExpression<'a>,
    object_bindings: &[(&'a str, &'a ObjectExpression<'a>)],
    depth: usize,
) -> StoryObjectKind {
    const MAX_SPREAD_DEPTH: usize = 8;
    if depth > MAX_SPREAD_DEPTH {
        return StoryObjectKind::Unsupported;
    }

    let mut has_story_shape = object.properties.is_empty();
    let mut has_unsupported_spread = false;

    for property in &object.properties {
        match property {
            ObjectPropertyKind::SpreadProperty(spread) => {
                match spread_story_object(export_name, &spread.argument, object_bindings, depth + 1)
                {
                    StoryObjectKind::Fixture => {}
                    StoryObjectKind::Story => has_story_shape = true,
                    StoryObjectKind::Unsupported => has_unsupported_spread = true,
                }
            }
            ObjectPropertyKind::ObjectProperty(prop) => {
                if !prop.computed
                    && property_key_name(&prop.key).is_some_and(is_story_annotation_key)
                {
                    has_story_shape = true;
                }
            }
        }
    }

    if has_unsupported_spread {
        StoryObjectKind::Unsupported
    } else if has_story_shape {
        StoryObjectKind::Story
    } else {
        StoryObjectKind::Fixture
    }
}

fn spread_story_object<'a>(
    export_name: &str,
    expression: &'a Expression<'a>,
    object_bindings: &[(&'a str, &'a ObjectExpression<'a>)],
    depth: usize,
) -> StoryObjectKind {
    let expression = unwrap_expression(expression);
    if let Expression::Identifier(ident) = expression
        && let Some(object) = find_object_by_name(object_bindings, ident.name.as_str())
    {
        return classify_story_object_at_depth(export_name, object, object_bindings, depth);
    }

    if unresolved_spread_may_be_story(export_name, expression) {
        StoryObjectKind::Unsupported
    } else {
        StoryObjectKind::Fixture
    }
}

fn unresolved_spread_may_be_story(export_name: &str, expression: &Expression<'_>) -> bool {
    story_like_name(export_name)
        || match expression {
            Expression::Identifier(ident) => story_like_name(ident.name.as_str()),
            Expression::StaticMemberExpression(member) => {
                story_like_name(member.property.name.as_str())
                    || matches!(
                        unwrap_expression(&member.object),
                        Expression::Identifier(object) if story_like_name(object.name.as_str())
                    )
            }
            _ => false,
        }
}

fn story_like_name(name: &str) -> bool {
    name.contains("Story") || name.chars().next().is_some_and(|ch| ch.is_uppercase())
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
