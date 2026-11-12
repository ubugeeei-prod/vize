//! Walk the script's ESTree once and turn the nodes the relation rules listen
//! to into checker asks, with the parent-dependent facts decided here.
mod checks;
mod context;
mod deprecated;
use super::binding::{self, Binding};
use super::{Key, Pending};
use crate::linter::corsa_session::{Literal, Span};
use oxc_ast::ast::Program;
use serde_json::{Value, json};

const ASSERTION: &str = "no-unnecessary-type-assertion";
const FOR_IN: &str = "no-for-in-array";
const ARRAY_DELETE: &str = "no-array-delete";
const REDUCE: &str = "prefer-reduce-type-parameter";
const RETURN_THIS: &str = "prefer-return-this-type";
const DEPRECATED: &str = "no-deprecated";
const ACCESSOR_PAIRS: &str = "related-getter-setter-pairs";
const TYPE_ARGUMENTS: &str = "no-unnecessary-type-arguments";

/// A node and the key it sits under in its parent.
type Ancestor<'a> = (&'a Value, &'a str);

struct Walk<'a, 'm, 'v> {
    rules: &'a [&'a str],
    map: &'m dyn Fn(Span) -> Option<Span>,
    pending: &'a mut Pending,
    /// Non-null operands whose binding still needs scope analysis.
    identifiers: Vec<(usize, Span)>,
    /// Type argument lists, the name they instantiate, and whether that name
    /// is constructed with `new`.
    instantiations: Vec<(&'v Value, Span, bool)>,
}

pub(super) fn collect(
    ast: &Program<'_>,
    program: &Value,
    rules: &[&str],
    map: &dyn Fn(Span) -> Option<Span>,
    pending: &mut Pending,
) {
    let mut walk = Walk {
        rules,
        map,
        pending,
        identifiers: Vec::new(),
        instantiations: Vec::new(),
    };
    let mut ancestors = Vec::new();
    walk.visit(program, &mut ancestors);
    let instantiations = std::mem::take(&mut walk.instantiations);
    let names = instantiations
        .iter()
        .map(|(_, name, constructed)| (*name, *constructed))
        .collect::<Vec<_>>();
    let defaults = binding::type_parameter_defaults(ast, &names);
    for ((list, _, _), defaults) in instantiations.into_iter().zip(defaults) {
        if let Some(defaults) = defaults {
            walk.type_arguments(list, &defaults);
        }
    }
    let identifiers = std::mem::take(&mut walk.identifiers);
    let spans = identifiers
        .iter()
        .map(|(_, span)| *span)
        .collect::<Vec<_>>();
    let bindings = binding::resolve(ast, &spans);
    for ((index, _), binding) in identifiers.into_iter().zip(bindings) {
        match binding {
            Binding::Assigned => {}
            Binding::MaybeUnassigned => {
                if let Some(target) = walk.pending.non_null.get_mut(index) {
                    target.used_before_assigned = true;
                }
            }
            Binding::Annotated { name } => {
                if let (Some(ask), Some(mapped)) =
                    (walk.pending.request.non_null.get_mut(index), map(name))
                {
                    ask.declared = Some(mapped);
                }
            }
        }
    }
}

fn kind(value: &Value) -> &str {
    value.get("type").and_then(Value::as_str).unwrap_or("")
}

fn span(value: &Value) -> Option<Span> {
    let start = u32::try_from(value.get("start")?.as_u64()?).ok()?;
    let end = u32::try_from(value.get("end")?.as_u64()?).ok()?;
    Some((start, end))
}

fn key(value: &Value) -> Option<Key> {
    let (start, end) = span(value)?;
    Some((static_kind(kind(value))?, start, end))
}

fn static_kind(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "TSAsExpression" => "TSAsExpression",
        "TSTypeAssertion" => "TSTypeAssertion",
        "TSNonNullExpression" => "TSNonNullExpression",
        "ForInStatement" => "ForInStatement",
        "FunctionExpression" => "FunctionExpression",
        "ArrowFunctionExpression" => "ArrowFunctionExpression",
        "Identifier" => "Identifier",
        "MemberExpression" => "MemberExpression",
        "ArrayExpression" => "ArrayExpression",
        "CallExpression" => "CallExpression",
        "NewExpression" => "NewExpression",
        "ThisExpression" => "ThisExpression",
        "ObjectExpression" => "ObjectExpression",
        "PrivateIdentifier" => "PrivateIdentifier",
        "Super" => "Super",
        "Property" => "Property",
        "TSTypeParameterInstantiation" => "TSTypeParameterInstantiation",
        "MethodDefinition" => "MethodDefinition",
        "TSMethodSignature" => "TSMethodSignature",
        _ => return None,
    })
}

/// The annotated type node of `TSTypeAnnotation` wrappers.
fn annotation_type(value: &Value) -> Option<&Value> {
    let annotation = value.get("typeAnnotation")?;
    if kind(annotation) == "TSTypeAnnotation" {
        annotation.get("typeAnnotation")
    } else {
        Some(annotation)
    }
}

fn is_assertion(value: &Value) -> bool {
    matches!(kind(value), "TSAsExpression" | "TSTypeAssertion")
}

fn literal(value: &Value) -> Literal {
    match kind(value) {
        "ObjectExpression" => Literal::Object {
            empty: value
                .get("properties")
                .and_then(Value::as_array)
                .is_none_or(Vec::is_empty),
        },
        "Literal"
        | "ArrayExpression"
        | "TemplateLiteral"
        | "ClassExpression"
        | "FunctionExpression"
        | "ArrowFunctionExpression"
        | "JSXElement"
        | "JSXFragment" => Literal::Other,
        _ => Literal::No,
    }
}

impl<'v> Walk<'_, '_, 'v> {
    fn enabled(&self, rule: &str) -> bool {
        self.rules.contains(&rule)
    }

    fn visit(&mut self, value: &'v Value, ancestors: &mut Vec<Ancestor<'v>>) {
        match value {
            Value::Array(items) => {
                for item in items {
                    self.visit(item, ancestors);
                }
                return;
            }
            Value::Object(_) => {}
            _ => return,
        }
        if value.get("type").is_some() {
            self.node(value, ancestors);
        }
        let Some(map) = value.as_object() else {
            return;
        };
        for (field, child) in map {
            if matches!(field.as_str(), "type" | "start" | "end" | "range" | "loc") {
                continue;
            }
            if child.is_object() || child.is_array() {
                ancestors.push((value, field.as_str()));
                self.visit(child, ancestors);
                ancestors.pop();
            }
        }
    }

    fn node(&mut self, value: &'v Value, ancestors: &[Ancestor<'v>]) {
        if kind(value) == "TSTypeParameterInstantiation" && self.enabled(TYPE_ARGUMENTS) {
            self.instantiation(value, ancestors);
        }
        if self.enabled(DEPRECATED) {
            match kind(value) {
                "Identifier" | "PrivateIdentifier" | "Super" => {
                    self.deprecated_use(value, ancestors)
                }
                "MemberExpression" => self.deprecated_member(value),
                "Property" => self.deprecated_key(value, ancestors),
                _ => {}
            }
        }
        match kind(value) {
            "TSAsExpression" | "TSTypeAssertion" if self.enabled(ASSERTION) => {
                self.assertion(value, ancestors);
            }
            "TSNonNullExpression" if self.enabled(ASSERTION) => self.non_null(value, ancestors),
            "ForInStatement" if self.enabled(FOR_IN) => {
                if let Some(right) = value.get("right") {
                    self.shape(right);
                }
            }
            "UnaryExpression" if self.enabled(ARRAY_DELETE) => {
                let argument = value.get("argument");
                if value.get("operator").and_then(Value::as_str) == Some("delete")
                    && argument.is_some_and(|argument| kind(argument) == "MemberExpression")
                    && let Some(object) = argument.and_then(|argument| argument.get("object"))
                {
                    self.shape(object);
                }
            }
            "CallExpression" if self.enabled(REDUCE) => self.reduce(value),
            "ClassBody" | "TSInterfaceBody" | "TSTypeLiteral" if self.enabled(ACCESSOR_PAIRS) => {
                self.accessor_pairs(value);
            }
            "MethodDefinition" | "PropertyDefinition" | "AccessorProperty"
                if self.enabled(RETURN_THIS) =>
            {
                self.returns(value, ancestors);
            }
            _ => {}
        }
    }

    /// `checkTSArgsAndParameters`: compare the last argument with its
    /// parameter's default.
    fn type_arguments(&mut self, list: &Value, defaults: &[Option<Span>]) {
        let Some(arguments) = list.get("params").and_then(Value::as_array) else {
            return;
        };
        let (Some(key), Some(last), Some(list_span)) = (key(list), arguments.last(), span(list))
        else {
            return;
        };
        let Some(Some(default)) = defaults.get(arguments.len() - 1) else {
            return;
        };
        let ranges = arguments
            .iter()
            .filter_map(span)
            .map(|(start, end)| json!({ "start": start, "end": end }))
            .collect::<Vec<_>>();
        let fixed = [
            ("__typeArgumentRanges", Value::Array(ranges)),
            (
                "__typeArgumentListRange",
                json!({ "start": list_span.0, "end": list_span.1 }),
            ),
            ("__typeParameterCount", json!(defaults.len())),
            ("__lastTypeParameterHasDefault", json!(true)),
        ];
        let pair = (|| Some(((self.map)(span(last)?)?, (self.map)(*default)?)))();
        let Some(pair) = pair else {
            return;
        };
        for (field, value) in fixed {
            self.pending.fixed.push((key, field, value));
        }
        self.pending.defaults.push(key);
        self.pending.request.defaults.push(pair);
    }

    /// A type argument list whose instantiated name may resolve to a local
    /// declaration with defaults (`TSTypeParameterInstantiation`).
    fn instantiation(&mut self, value: &'v Value, ancestors: &[Ancestor<'v>]) {
        let Some(&(parent, field)) = ancestors.last() else {
            return;
        };
        let name = match (kind(parent), field) {
            ("TSTypeReference", "typeArguments") => parent.get("typeName"),
            ("TSClassImplements" | "TSInterfaceHeritage", "typeArguments") => {
                parent.get("expression")
            }
            ("ClassDeclaration" | "ClassExpression", "superTypeArguments") => {
                parent.get("superClass")
            }
            ("CallExpression" | "NewExpression", "typeArguments") => parent.get("callee"),
            ("TaggedTemplateExpression", "typeArguments") => parent.get("tag"),
            _ => None,
        };
        let Some(name) = name
            .filter(|name| kind(name) == "Identifier")
            .and_then(span)
        else {
            return;
        };
        self.instantiations
            .push((value, name, kind(parent) == "NewExpression"));
    }
}

#[cfg(test)]
mod tests {
    use super::context::{call_like, generic_callees, inside_import, is_declaration};
    use serde_json::json;

    #[test]
    fn object_keys_and_pattern_values_are_declarations() {
        let object = json!({ "type": "ObjectExpression", "start": 0, "end": 9 });
        let pattern = json!({ "type": "ObjectPattern", "start": 0, "end": 9 });
        let property = json!({ "type": "Property", "start": 1, "end": 8 });
        assert!(is_declaration(
            "key",
            &[(&object, "properties"), (&property, "key")]
        ));
        assert!(!is_declaration(
            "value",
            &[(&object, "properties"), (&property, "value")]
        ));
        assert!(is_declaration(
            "value",
            &[(&pattern, "properties"), (&property, "value")]
        ));
    }

    #[test]
    fn member_chains_reach_their_call() {
        let call = json!({ "type": "CallExpression", "start": 0, "end": 7 });
        let member = json!({ "type": "MemberExpression", "start": 0, "end": 5 });
        let found = call_like(&[(&call, "callee"), (&member, "property")]);
        assert_eq!(found.and_then(|node| node.get("start")), Some(&json!(0)));
        assert!(call_like(&[(&call, "arguments")]).is_none());
    }

    #[test]
    fn imports_stop_at_the_first_scope() {
        let import = json!({ "type": "ImportDeclaration" });
        let block = json!({ "type": "BlockStatement" });
        assert!(inside_import(&[(&import, "specifiers")]));
        assert!(!inside_import(&[(&import, "specifiers"), (&block, "body")]));
    }

    #[test]
    fn generic_context_stops_at_block_bodied_functions() {
        let node = json!({ "type": "TSAsExpression", "start": 20, "end": 30 });
        let call = json!({
            "type": "CallExpression", "start": 0, "end": 40,
            "callee": { "type": "Identifier", "start": 0, "end": 3 },
            "arguments": []
        });
        let arrow = json!({
            "type": "ArrowFunctionExpression",
            "body": { "type": "BlockStatement" }
        });
        assert_eq!(
            generic_callees(&node, &[(&call, "arguments")]),
            vec![(0, 3)]
        );
        assert!(generic_callees(&node, &[(&call, "arguments"), (&arrow, "body")]).is_empty());
    }
}
