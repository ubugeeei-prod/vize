//! esquery matching over the JSON template tree.

use oxc_syntax::number::ToJsString;
use regex::Regex;
use serde_json::Value;
use vize_l0::config::esquery::{Combinator, Operator, Selector, Simple, Value as Expected};
use vize_l0::{FxHashMap, String};

/// Every node of the tree with its parent, field and position among siblings.
pub(super) struct Tree<'v> {
    pub(super) nodes: Vec<Node<'v>>,
}

pub(super) struct Node<'v> {
    pub(super) value: &'v Value,
    parent: Option<usize>,
    field: Option<&'v str>,
    /// Index in the parent's array field and that array's length.
    position: Option<(usize, usize)>,
    previous: Option<usize>,
}

impl<'v> Tree<'v> {
    pub(super) fn new(root: &'v Value) -> Self {
        let mut tree = Self { nodes: Vec::new() };
        tree.add(root, None, None, None, None);
        tree
    }

    fn add(
        &mut self,
        value: &'v Value,
        parent: Option<usize>,
        field: Option<&'v str>,
        position: Option<(usize, usize)>,
        previous: Option<usize>,
    ) -> usize {
        let index = self.nodes.len();
        self.nodes.push(Node {
            value,
            parent,
            field,
            position,
            previous,
        });
        let Value::Object(map) = value else {
            return index;
        };
        for (key, child) in map {
            if matches!(key.as_str(), "type" | "start" | "end") {
                continue;
            }
            match child {
                Value::Array(items) => {
                    let mut previous = None;
                    for (at, item) in items.iter().enumerate() {
                        if is_node(item) {
                            let added = self.add(
                                item,
                                Some(index),
                                Some(key.as_str()),
                                Some((at, items.len())),
                                previous,
                            );
                            previous = Some(added);
                        }
                    }
                }
                child if is_node(child) => {
                    self.add(child, Some(index), Some(key.as_str()), None, None);
                }
                _ => {}
            }
        }
        index
    }

    pub(super) fn matches(&self, selector: &Selector, regexes: &Regexes, node: usize) -> bool {
        match selector {
            Selector::Compound(parts) => parts.iter().all(|part| self.simple(part, regexes, node)),
            Selector::Relation(left, combinator, right) => {
                if !self.matches(right, regexes, node) {
                    return false;
                }
                let Some(this) = self.nodes.get(node) else {
                    return false;
                };
                match combinator {
                    Combinator::Child => this
                        .parent
                        .is_some_and(|parent| self.matches(left, regexes, parent)),
                    Combinator::Descendant => {
                        let mut ancestor = this.parent;
                        while let Some(index) = ancestor {
                            if self.matches(left, regexes, index) {
                                return true;
                            }
                            ancestor = self.nodes.get(index).and_then(|node| node.parent);
                        }
                        false
                    }
                    Combinator::Adjacent => this
                        .previous
                        .is_some_and(|previous| self.matches(left, regexes, previous)),
                    Combinator::Sibling => {
                        let mut sibling = this.previous;
                        while let Some(index) = sibling {
                            if self.matches(left, regexes, index) {
                                return true;
                            }
                            sibling = self.nodes.get(index).and_then(|node| node.previous);
                        }
                        false
                    }
                }
            }
        }
    }

    fn simple(&self, part: &Simple, regexes: &Regexes, node: usize) -> bool {
        let Some(this) = self.nodes.get(node) else {
            return false;
        };
        let kind = node_type(this.value);
        match part {
            Simple::Wildcard => true,
            Simple::Type(name) => kind.eq_ignore_ascii_case(name),
            Simple::Attribute(path, test) => {
                let found = path
                    .iter()
                    .try_fold(this.value, |value, key| value.get(key.as_str()));
                match test {
                    None => found.is_some_and(|value| !value.is_null()),
                    Some((operator, expected)) => compare(found, *operator, expected, regexes),
                }
            }
            Simple::Field(path) => {
                let mut current = Some(node);
                path.iter().rev().all(|segment| {
                    let Some(index) = current else {
                        return false;
                    };
                    let Some(node) = self.nodes.get(index) else {
                        return false;
                    };
                    current = node.parent;
                    node.field == Some(segment.as_str())
                })
            }
            Simple::Matches(selectors) => selectors
                .iter()
                .any(|selector| self.matches(selector, regexes, node)),
            Simple::Not(selectors) => !selectors
                .iter()
                .any(|selector| self.matches(selector, regexes, node)),
            Simple::Has(selectors) => (node + 1..self.nodes.len())
                .take_while(|index| self.is_descendant(*index, node))
                .any(|index| {
                    selectors
                        .iter()
                        .any(|selector| self.matches(selector, regexes, index))
                }),
            Simple::NthChild(index) => this.position.is_some_and(|(at, _)| at + 1 == *index),
            Simple::NthLastChild(index) => this
                .position
                .is_some_and(|(at, length)| length - at == *index),
            Simple::Class(class) => {
                let parent_kind = this
                    .parent
                    .and_then(|parent| self.nodes.get(parent))
                    .map(|parent| node_type(parent.value));
                is_class(class, kind, parent_kind)
            }
        }
    }

    fn is_descendant(&self, node: usize, ancestor: usize) -> bool {
        let mut current = self.nodes.get(node).and_then(|node| node.parent);
        while let Some(index) = current {
            if index == ancestor {
                return true;
            }
            current = self.nodes.get(index).and_then(|node| node.parent);
        }
        false
    }
}

/// Compiled `/regex/` values, keyed by pattern.
pub(super) type Regexes = FxHashMap<String, Regex>;

fn is_node(value: &Value) -> bool {
    value.get("type").and_then(Value::as_str).is_some()
}

pub(super) fn node_type(value: &Value) -> &str {
    value
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or_default()
}

/// esquery's attribute test: `${expected}` === `${found}`, a regex test of a
/// string, or a numeric comparison.
fn compare(
    found: Option<&Value>,
    operator: Operator,
    expected: &Expected,
    regexes: &Regexes,
) -> bool {
    let equal = |found: Option<&Value>| match expected {
        Expected::Regex(pattern) => found
            .and_then(Value::as_str)
            .zip(regexes.get(pattern.as_str()))
            .is_some_and(|(text, regex)| regex.is_match(text)),
        Expected::Literal(text) => js_string(found) == text.as_str(),
        Expected::Number(number) => js_string(found).as_str() == number.to_js_string(),
    };
    let number = || match expected {
        Expected::Number(number) => Some(*number),
        Expected::Literal(text) => text.parse::<f64>().ok(),
        Expected::Regex(_) => None,
    };
    let found_number = found.and_then(|value| match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse::<f64>().ok(),
        _ => None,
    });
    match operator {
        Operator::Equal => equal(found),
        Operator::NotEqual => !equal(found),
        operator => {
            let (Some(found), Some(expected)) = (found_number, number()) else {
                return false;
            };
            match operator {
                Operator::Less => found < expected,
                Operator::LessEqual => found <= expected,
                Operator::Greater => found > expected,
                _ => found >= expected,
            }
        }
    }
}

/// JavaScript's template-literal string of a JSON value.
fn js_string(value: Option<&Value>) -> String {
    match value {
        None => String::from("undefined"),
        Some(Value::Null) => String::from("null"),
        Some(Value::Bool(value)) => String::from(if *value { "true" } else { "false" }),
        Some(Value::Number(number)) => number.as_f64().map_or_else(String::default, |number| {
            String::from(number.to_js_string())
        }),
        Some(Value::String(text)) => String::from(text.as_str()),
        Some(Value::Array(items)) => {
            let mut joined = String::default();
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    joined.push(',');
                }
                if !item.is_null() {
                    joined.push_str(&js_string(Some(item)));
                }
            }
            joined
        }
        Some(Value::Object(_)) => String::from("[object Object]"),
    }
}

fn is_class(class: &str, kind: &str, parent_kind: Option<&str>) -> bool {
    let expression = || {
        kind.ends_with("Expression")
            || kind.ends_with("Literal")
            || kind == "Identifier" && parent_kind != Some("MetaProperty")
            || kind == "MetaProperty"
    };
    match class {
        "statement" => kind.ends_with("Statement") || kind.ends_with("Declaration"),
        "declaration" => kind.ends_with("Declaration"),
        "pattern" => kind.ends_with("Pattern") || expression(),
        "expression" => expression(),
        "function" => matches!(
            kind,
            "FunctionDeclaration" | "FunctionExpression" | "ArrowFunctionExpression"
        ),
        _ => false,
    }
}
