//! Single-file props shapes, and the names a component may declare as props.

use oxc_ast::ast::{
    Argument, Expression, ObjectExpression, ObjectPropertyKind, Program, Statement,
};
use vize_croquis::{Croquis, types::TypeDefinitions};
use vize_l0::{CompactString, FxHashSet};

/// Whether the component may declare `name` as a prop.
///
/// Resolved, runtime and destructured props come from croquis. Authored
/// `withDefaults` keys and `Pick<_, 'name'>` keys declare the name even when
/// the picked type is out of reach, and a props type the file cannot resolve
/// may declare any name.
pub(crate) fn may_declare_prop(
    analysis: &Croquis,
    script_setup: Option<&Program<'_>>,
    name: &str,
) -> bool {
    let macros = &analysis.macros;
    macros.props().iter().any(|prop| prop.name.as_str() == name)
        || macros
            .props_destructure()
            .is_some_and(|destructure| destructure.get(name).is_some())
        || script_setup.is_some_and(|program| with_defaults_declares(program, name))
        || picks_key(analysis, name)
        || has_unresolved_props(analysis)
}

/// Whether the authored `withDefaults` object names `name`, or spreads or
/// computes a key that may. Croquis keeps these keys only for resolved props,
/// so they are read from the script setup AST the lint pass already holds.
fn with_defaults_declares(program: &Program<'_>, name: &str) -> bool {
    program
        .body
        .iter()
        .find_map(with_defaults_object)
        .is_some_and(|object| {
            object.properties.iter().any(|property| match property {
                ObjectPropertyKind::SpreadProperty(_) => true,
                ObjectPropertyKind::ObjectProperty(property) => property
                    .key
                    .static_name()
                    .is_none_or(|key| key.as_ref() == name),
            })
        })
}

/// The object argument of a top-level `withDefaults(defineProps(...), { ... })`.
fn with_defaults_object<'a>(statement: &'a Statement<'a>) -> Option<&'a ObjectExpression<'a>> {
    let expression = match statement {
        Statement::ExpressionStatement(statement) => &statement.expression,
        Statement::VariableDeclaration(declaration) => declaration
            .declarations
            .iter()
            .find_map(|declarator| declarator.init.as_ref())?,
        _ => return None,
    };
    let Expression::CallExpression(call) = expression.get_inner_expression() else {
        return None;
    };
    if !matches!(&call.callee, Expression::Identifier(callee) if callee.name == "withDefaults") {
        return None;
    }
    match call.arguments.get(1)? {
        Argument::ObjectExpression(object) => Some(object),
        _ => None,
    }
}

/// Whether the props type, its local aliases or its interface heritage pick
/// `name` with `Pick<_, 'name'>`. Members of object types are not heritage.
fn picks_key(analysis: &Croquis, name: &str) -> bool {
    let Some(type_args) = analysis
        .macros
        .define_props()
        .and_then(|call| call.type_args.as_deref())
    else {
        return false;
    };
    let definitions = analysis.types.definitions();
    let mut pending = vec![CompactString::new(type_args)];
    let mut visited = FxHashSet::default();
    while let Some(shape) = pending.pop() {
        let shape = outside_object_types(&shape);
        if picked_keys(&shape).any(|key| key == name) {
            return true;
        }
        for word in shape
            .split(|character: char| {
                !(character.is_ascii_alphanumeric() || "_$".contains(character))
            })
            .filter(|word| !word.is_empty())
        {
            if visited.len() >= 32 || !visited.insert(CompactString::new(word)) {
                continue;
            }
            if let Some(body) = definitions.type_aliases.get(word) {
                pending.push(body.clone());
            }
            pending.extend(definitions.interface_extends(word));
        }
    }
    false
}

/// `shape` with every `{ ... }` object type blanked out.
fn outside_object_types(shape: &str) -> CompactString {
    let mut depth = 0usize;
    shape
        .chars()
        .map(|character| match character {
            '{' => {
                depth += 1;
                ' '
            }
            '}' => {
                depth = depth.saturating_sub(1);
                ' '
            }
            _ if depth > 0 => ' ',
            _ => character,
        })
        .collect()
}

/// The string-literal keys of every `Pick<Type, Keys>` in `shape`.
fn picked_keys(shape: &str) -> impl Iterator<Item = &str> {
    shape
        .match_indices("Pick<")
        .flat_map(move |(start, matched)| {
            let arguments = shape.get(start + matched.len()..).unwrap_or_default();
            let mut depth = 0usize;
            let mut keys_start = None;
            let mut end = arguments.len();
            for (index, character) in arguments.char_indices() {
                match character {
                    '<' | '(' | '[' => depth += 1,
                    '>' if depth == 0 => {
                        end = index;
                        break;
                    }
                    '>' | ')' | ']' => depth = depth.saturating_sub(1),
                    ',' if depth == 0 && keys_start.is_none() => keys_start = Some(index + 1),
                    _ => {}
                }
            }
            let keys = keys_start
                .and_then(|from| arguments.get(from..end))
                .unwrap_or_default();
            keys.split('|').filter_map(|key| {
                let key = key.trim();
                key.strip_prefix('\'')
                    .and_then(|key| key.strip_suffix('\''))
                    .or_else(|| key.strip_prefix('"').and_then(|key| key.strip_suffix('"')))
            })
        })
}

pub(crate) fn has_unresolved_props(analysis: &Croquis) -> bool {
    if let Some(complete) = analysis.types.resolved_props_complete() {
        return !complete;
    }
    let Some(type_args) = analysis
        .macros
        .define_props()
        .and_then(|call| call.type_args.as_deref())
    else {
        return false;
    };
    let type_args = type_args
        .strip_prefix('<')
        .and_then(|inner| inner.strip_suffix('>'))
        .unwrap_or(type_args);
    imported_shape(
        analysis.types.definitions(),
        type_args,
        &mut FxHashSet::default(),
    )
}

fn imported_shape(
    definitions: &TypeDefinitions,
    shape: &str,
    visiting: &mut FxHashSet<CompactString>,
) -> bool {
    let shape = shape.trim();
    // Imported member value types cannot contribute additional prop names.
    if shape.starts_with('{') {
        return false;
    }
    let (name, arguments) = shape
        .split_once('<')
        .map_or((shape, None), |(name, args)| (name.trim(), Some(args)));
    if definitions.is_imported(name) {
        return true;
    }
    if matches!(name, "Partial" | "Required" | "Readonly" | "Pick" | "Omit") {
        return arguments.is_some_and(|arguments| {
            let base = arguments
                .trim_end_matches('>')
                .split(',')
                .next()
                .unwrap_or_default();
            imported_shape(definitions, base, visiting)
        });
    }
    if shape.contains(['&', '|']) {
        return shape
            .split(['&', '|'])
            .any(|part| imported_shape(definitions, part, visiting));
    }
    if !visiting.insert(CompactString::new(name)) {
        return false;
    }
    let imported = definitions
        .type_aliases
        .get(name)
        .is_some_and(|body| imported_shape(definitions, body, visiting))
        || definitions
            .interface_extends(name)
            .iter()
            .any(|base| imported_shape(definitions, base, visiting));
    visiting.remove(name);
    imported
}
