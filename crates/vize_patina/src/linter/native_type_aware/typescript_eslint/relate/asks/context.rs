//! Syntax context the relation asks read off the ancestor chain: return arguments, narrowing, contextual declarations, imports and generic callees.
use super::{Ancestor, annotation_type, kind, span};
use crate::linter::corsa_session::{ContextSource, Span};
use serde_json::Value;
use vize_l0::String as CompactString;

/// `return` arguments of a function body, skipping nested functions.
pub(super) fn return_arguments<'v>(value: &'v Value, out: &mut Vec<&'v Value>) {
    match value {
        Value::Array(items) => {
            for item in items {
                return_arguments(item, out);
            }
        }
        Value::Object(map) => {
            match kind(value) {
                "FunctionDeclaration" | "FunctionExpression" | "ArrowFunctionExpression" => return,
                "ReturnStatement" => {
                    if let Some(argument) =
                        value.get("argument").filter(|argument| !argument.is_null())
                    {
                        out.push(argument);
                    }
                    return;
                }
                _ => {}
            }
            for (field, child) in map {
                if !matches!(field.as_str(), "type" | "start" | "end") {
                    return_arguments(child, out);
                }
            }
        }
        _ => {}
    }
}

/// `isImplicitlyNarrowedLiteralDeclaration` without the cast-type half.
pub(super) fn narrowed_declaration(expression: &Value, ancestors: &[Ancestor<'_>]) -> bool {
    if kind(expression) == "TemplateLiteral"
        && expression
            .get("expressions")
            .and_then(Value::as_array)
            .is_some_and(|expressions| !expressions.is_empty())
    {
        return false;
    }
    let mut parents = ancestors.iter().rev().map(|(node, _)| *node);
    let parent = parents.next();
    let grandparent = parents.next();
    grandparent.is_some_and(|node| {
        kind(node) == "VariableDeclaration"
            && node.get("kind").and_then(Value::as_str) == Some("const")
    }) || parent.is_some_and(|node| {
        kind(node) == "PropertyDefinition"
            && node.get("readonly").and_then(Value::as_bool) == Some(true)
    })
}

/// typescript-eslint's `getContextualType(checker, node)` util for `x!`.
pub(super) fn context(value: &Value, ancestors: &[Ancestor<'_>]) -> Option<ContextSource> {
    let (parent, field) = *ancestors.last()?;
    let own = span(value)?;
    match kind(parent) {
        "CallExpression" | "NewExpression" => {
            (field == "arguments").then_some(ContextSource::Checker(own))
        }
        "VariableDeclarator" if field == "init" => {
            annotated_name(parent.get("id")?, parent.get("id")?)
        }
        "PropertyDefinition" if field == "value" => {
            if parent.get("computed").and_then(Value::as_bool) == Some(true) {
                return None;
            }
            annotated_name(parent, parent.get("key")?)
        }
        "AssignmentPattern" if field == "right" => {
            let (holder, holder_field) = *ancestors.iter().rev().nth(1)?;
            let parameter = holder_field == "params"
                && matches!(
                    kind(holder),
                    "FunctionDeclaration"
                        | "FunctionExpression"
                        | "ArrowFunctionExpression"
                        | "TSDeclareFunction"
                );
            if !parameter {
                return None;
            }
            annotated_name(parent.get("left")?, parent.get("left")?)
        }
        "JSXExpressionContainer" => span(parent).map(ContextSource::Checker),
        "AssignmentExpression"
            if field == "right" && parent.get("operator").and_then(Value::as_str) == Some("=") =>
        {
            span(parent.get("left")?).map(ContextSource::TypeAt)
        }
        "TemplateLiteral" if field == "expressions" => Some(ContextSource::Checker(own)),
        _ => None,
    }
}

/// `getTypeFromTypeNode(declaration.type)`, read as the declared type of the
/// declaration's name: `holder` carries the annotation, `name` is the binding.
pub(super) fn annotated_name(holder: &Value, name: &Value) -> Option<ContextSource> {
    annotation_type(holder)?;
    if kind(name) != "Identifier" {
        return None;
    }
    span(name).map(ContextSource::TypeAt)
}

/// typescript-eslint `no-deprecated`'s `isDeclaration`, from the parent.
pub(super) fn is_declaration(field: &str, ancestors: &[Ancestor<'_>]) -> bool {
    let Some(&(parent, _)) = ancestors.last() else {
        return false;
    };
    let grandparent = ancestors.iter().rev().nth(1).map(|(node, _)| kind(node));
    match kind(parent) {
        "ArrayPattern" => field == "elements",
        "ClassExpression"
        | "ClassDeclaration"
        | "VariableDeclarator"
        | "TSEnumMember"
        | "TSImportEqualsDeclaration" => field == "id",
        "MethodDefinition" | "PropertyDefinition" | "AccessorProperty" => field == "key",
        "Property" => match field {
            "value" => grandparent == Some("ObjectPattern"),
            _ => grandparent == Some("ObjectExpression"),
        },
        "AssignmentPattern" => field == "left",
        "ArrowFunctionExpression"
        | "FunctionDeclaration"
        | "FunctionExpression"
        | "TSDeclareFunction"
        | "TSEmptyBodyFunctionExpression"
        | "TSEnumDeclaration"
        | "TSInterfaceDeclaration"
        | "TSMethodSignature"
        | "TSModuleDeclaration"
        | "TSParameterProperty"
        | "TSPropertySignature"
        | "TSTypeAliasDeclaration"
        | "TSTypeParameter" => true,
        _ => false,
    }
}

/// typescript-eslint `no-deprecated`'s `isInsideImport`.
pub(super) fn inside_import(ancestors: &[Ancestor<'_>]) -> bool {
    for (node, _) in ancestors.iter().rev() {
        match kind(node) {
            "ImportDeclaration" => return true,
            "ArrowFunctionExpression"
            | "ExportAllDeclaration"
            | "ExportNamedDeclaration"
            | "BlockStatement"
            | "ClassDeclaration"
            | "TSInterfaceDeclaration"
            | "FunctionDeclaration"
            | "FunctionExpression"
            | "Program"
            | "TSUnionType"
            | "VariableDeclarator" => return false,
            _ => {}
        }
    }
    false
}

/// typescript-eslint `no-deprecated`'s `getCallLikeNode`: the call-like node
/// whose callee is this identifier, possibly as the property of a member chain.
pub(super) fn call_like<'v>(ancestors: &[Ancestor<'v>]) -> Option<&'v Value> {
    let mut index = ancestors.len();
    loop {
        let (parent, field) = *ancestors.get(index.checked_sub(1)?)?;
        if kind(parent) == "MemberExpression" && field == "property" {
            index -= 1;
            continue;
        }
        let callee = match kind(parent) {
            "CallExpression" | "NewExpression" => field == "callee",
            "TaggedTemplateExpression" => field == "tag",
            "JSXOpeningElement" => field == "name",
            _ => false,
        };
        return callee.then_some(parent);
    }
}

/// `getNameFromMember`: a static key name, or the key's span for computed keys.
pub(super) fn member_name(member: &Value) -> Option<CompactString> {
    let key = member.get("key")?;
    let computed = member.get("computed").and_then(Value::as_bool) == Some(true);
    if !computed {
        if let Some(name) = key.get("name").and_then(Value::as_str) {
            return Some(CompactString::from(name));
        }
        match key.get("value") {
            Some(Value::String(text)) => return Some(CompactString::from(text.as_str())),
            Some(Value::Number(number)) => {
                return Some(vize_l0::cstr!("{number}"));
            }
            _ => {}
        }
    }
    let (start, end) = span(key)?;
    Some(vize_l0::cstr!("[{start}:{end}]"))
}

/// `isInGenericContext`: callees of the enclosing calls whose generic call
/// signature would infer type arguments from this assertion, in walk order.
pub(super) fn generic_callees(node: &Value, ancestors: &[Ancestor<'_>]) -> Vec<Span> {
    let mut callees = Vec::new();
    let mut seen_function = false;
    let own = span(node);
    for (index, (current, _)) in ancestors.iter().enumerate().rev() {
        match kind(current) {
            "FunctionDeclaration" => break,
            "FunctionExpression" | "ArrowFunctionExpression" => {
                if current
                    .get("body")
                    .is_some_and(|body| kind(body) == "BlockStatement")
                    || seen_function
                {
                    break;
                }
                seen_function = true;
            }
            "CallExpression" | "NewExpression" => {
                if current.get("typeArguments").is_some_and(|ty| !ty.is_null()) {
                    continue;
                }
                let direct_argument = index + 1 == ancestors.len()
                    && current
                        .get("arguments")
                        .and_then(Value::as_array)
                        .is_some_and(|arguments| {
                            arguments.iter().any(|argument| span(argument) == own)
                        });
                let member_callee = current
                    .get("callee")
                    .is_some_and(|callee| kind(callee) == "MemberExpression");
                if kind(current) == "CallExpression" && member_callee && direct_argument {
                    continue;
                }
                if let Some(callee) = current.get("callee").and_then(span) {
                    callees.push(callee);
                }
            }
            _ => {}
        }
    }
    callees
}
