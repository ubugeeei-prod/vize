//! Declaration merging for `no-mixed-enums`: a later `enum` declaration takes
//! its member kind from the first member of an earlier same-named declaration
//! it merges with (in the same scope, or in same-named namespaces merged
//! through `export`).
use super::super::tree;
use corsa::lint::LintNode;
use serde_json::Value;
use vize_l0::{FxHashMap, String as CompactString, cstr};

/// Attaches `__previousEnumFirstMember` to every enum declaration that merges
/// with an earlier one that has members.
pub(super) fn stamp_merged_enums(program: &Value, source: &str, nodes: &mut [LintNode]) {
    let mut declarations = Vec::new();
    walk(program, &Context::default(), false, &mut declarations);
    let mut first_members = FxHashMap::default();
    for (index, declaration) in declarations.iter().enumerate() {
        let earlier = declarations
            .get(..index)
            .unwrap_or_default()
            .iter()
            .find(|earlier| {
                earlier.key == declaration.key
                    && earlier.name == declaration.name
                    && earlier.first_member.is_some()
            });
        if let Some(member) = earlier.and_then(|earlier| earlier.first_member) {
            first_members.insert(declaration.start, member);
        }
    }
    for node in nodes {
        if node.kind != "TSEnumDeclaration" {
            continue;
        }
        let Some(member) = first_members.get(&node.range.start) else {
            continue;
        };
        if let Some(member) = tree::node_of(member, source, "TSEnumBody") {
            node.children
                .insert("__previousEnumFirstMember".into(), member);
        }
    }
}

struct Declaration<'a> {
    key: CompactString,
    name: CompactString,
    start: u32,
    first_member: Option<&'a Value>,
}

#[derive(Clone, Default)]
struct Context {
    /// Identity of the innermost block scope.
    scope: CompactString,
    /// Merge key of the namespace whose body is that scope.
    namespace: Option<CompactString>,
}

const BLOCKS: &[&str] = &[
    "BlockStatement",
    "StaticBlock",
    "SwitchStatement",
    "ForStatement",
    "ForInStatement",
    "ForOfStatement",
    "CatchClause",
];

fn walk<'a>(value: &'a Value, context: &Context, exported: bool, out: &mut Vec<Declaration<'a>>) {
    match value {
        Value::Array(items) => {
            for item in items {
                walk(item, context, false, out);
            }
            return;
        }
        Value::Object(_) => {}
        _ => return,
    }
    let kind = value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let start = value.get("start").and_then(Value::as_u64).unwrap_or(0);
    match kind {
        "ExportNamedDeclaration" => {
            if let Some(declaration) = value.get("declaration") {
                walk(declaration, context, true, out);
            }
            return;
        }
        "TSEnumDeclaration" => {
            let key = merge_key(context, exported);
            let name = identifier(value.get("id")).unwrap_or_default();
            let first_member = value
                .get("body")
                .and_then(|body| body.get("members"))
                .or_else(|| value.get("members"))
                .and_then(Value::as_array)
                .and_then(|members| members.first());
            out.push(Declaration {
                key,
                name,
                start: u32::try_from(start).unwrap_or(u32::MAX),
                first_member,
            });
        }
        "TSModuleDeclaration" => {
            let name = module_name(value.get("id"));
            let key = cstr!("{name}@{}", merge_key(context, exported));
            let inner = Context {
                scope: cstr!("module:{start}"),
                namespace: Some(key),
            };
            if let Some(body) = value.get("body") {
                walk_module_body(body, &inner, out);
            }
            return;
        }
        _ => {}
    }
    let inner = if BLOCKS.contains(&kind) {
        Context {
            scope: cstr!("block:{start}"),
            namespace: None,
        }
    } else {
        context.clone()
    };
    if let Some(map) = value.as_object() {
        for (key, child) in map {
            if key != "type" {
                walk(child, &inner, false, out);
            }
        }
    }
}

/// A `namespace A.B {}` body may itself be a nested module declaration.
fn walk_module_body<'a>(body: &'a Value, context: &Context, out: &mut Vec<Declaration<'a>>) {
    if body.get("type").and_then(Value::as_str) == Some("TSModuleDeclaration") {
        walk(body, context, true, out);
        return;
    }
    if let Some(statements) = body.get("body") {
        walk(statements, context, false, out);
    }
}

/// typescript-eslint `getEnclosingScopes`: an exported declaration in a
/// namespace body merges across every same-named namespace; anything else
/// stays in its own scope.
fn merge_key(context: &Context, exported: bool) -> CompactString {
    match (&context.namespace, exported) {
        (Some(namespace), true) => namespace.clone(),
        _ => context.scope.clone(),
    }
}

fn identifier(value: Option<&Value>) -> Option<CompactString> {
    value?
        .get("name")
        .and_then(Value::as_str)
        .map(CompactString::new)
}

/// `A`, `A.B` for a qualified name, or the quoted string of a module literal.
fn module_name(id: Option<&Value>) -> CompactString {
    let Some(id) = id else {
        return CompactString::default();
    };
    match id.get("type").and_then(Value::as_str) {
        Some("TSQualifiedName") => cstr!(
            "{}.{}",
            module_name(id.get("left")),
            identifier(id.get("right")).unwrap_or_default()
        ),
        Some("Literal") => id
            .get("value")
            .map(|value| cstr!("{value}"))
            .unwrap_or_default(),
        _ => identifier(Some(id)).unwrap_or_default(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::super::tree;
    use super::stamp_merged_enums;

    fn merged(source: &str) -> Vec<bool> {
        let mut collected = tree::collect(source, false, &["TSEnumDeclaration"]);
        stamp_merged_enums(&collected.program, source, &mut collected.nodes);
        collected
            .nodes
            .iter()
            .map(|node| node.child("__previousEnumFirstMember").is_some())
            .collect()
    }

    #[test]
    fn a_later_same_scope_declaration_follows_the_first() {
        assert_eq!(
            merged("enum Foo { A }\nenum Foo { B = 'b' }\nenum Bar { C }\n"),
            vec![false, true, false]
        );
    }

    #[test]
    fn exported_enums_merge_across_same_named_namespaces() {
        let source = "namespace T { export enum E { A = 1 } }\nnamespace T { export enum E { B = 'b' } }\nnamespace O { export enum E { C = 2 } }\n";
        assert_eq!(merged(source), vec![false, true, false]);
    }

    #[test]
    fn unexported_enums_in_separate_namespaces_stay_apart() {
        let source = "namespace T { enum E { A = 1 } }\nnamespace T { enum E { B = 'b' } }\n";
        assert_eq!(merged(source), vec![false, false]);
    }
}
