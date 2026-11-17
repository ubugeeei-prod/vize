//! typescript-eslint `no-unsafe-enum-assignment` reports a value once. A
//! report on a value marks the array and object literals inside it checked
//! (`markChecked`), and their own element and property checks then skip them:
//!
//! ```ts
//! const boxes: T = [{ fruit: 1 }] as const; // the declarator, not `fruit: 1`
//! ```
//!
//! The corsa rule and the checker relations report independently, so the
//! marking is replayed over their merged reports in listener order.
use serde_json::Value;
use vize_l0::{FxHashSet, String as CompactString};

/// One `type/no-unsafe-enum-assignment` report in script coordinates.
pub(super) struct Report {
    pub start: u32,
    pub end: u32,
    pub message: CompactString,
}

const ASSIGNED: &str = "The value assigned here ";
const ARGUMENT: &str = "The argument passed here ";
const RETURNED: &str = "The value returned here ";
const ASSERTED: &str = "The value asserted here ";

type Span = (u32, u32);

struct Marking {
    reported: FxHashSet<(Span, &'static str)>,
    /// Array and object literals inside a reported value.
    marked: FxHashSet<Span>,
    dropped: FxHashSet<(Span, &'static str)>,
}

/// Remove the reports upstream never makes because an enclosing report
/// already marked their literal checked.
pub(super) fn retain_unchecked(program: &Value, reports: &mut Vec<Report>) {
    if reports.len() < 2 {
        return;
    }
    let mut marking = Marking {
        reported: reports
            .iter()
            .filter_map(|report| Some(((report.start, report.end), prefix(&report.message)?)))
            .collect(),
        marked: FxHashSet::default(),
        dropped: FxHashSet::default(),
    };
    marking.walk(program);
    if marking.dropped.is_empty() {
        return;
    }
    reports.retain(|report| {
        prefix(&report.message).is_none_or(|prefix| {
            !marking
                .dropped
                .contains(&((report.start, report.end), prefix))
        })
    });
}

fn prefix(message: &str) -> Option<&'static str> {
    [ASSIGNED, ARGUMENT, RETURNED, ASSERTED]
        .into_iter()
        .find(|prefix| message.starts_with(prefix))
}

fn kind(value: &Value) -> &str {
    value.get("type").and_then(Value::as_str).unwrap_or("")
}

fn span(value: &Value) -> Option<Span> {
    let start = u32::try_from(value.get("start")?.as_u64()?).ok()?;
    let end = u32::try_from(value.get("end")?.as_u64()?).ok()?;
    Some((start, end))
}

fn list<'a>(value: &'a Value, field: &str) -> impl Iterator<Item = &'a Value> {
    value
        .get(field)
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter(|item| !item.is_null())
}

impl Marking {
    /// Parents before children: a mark only reaches a reported node's own
    /// value, which is visited after it.
    fn walk(&mut self, value: &Value) {
        match value {
            Value::Array(items) => {
                for item in items {
                    self.walk(item);
                }
            }
            Value::Object(fields) => {
                if fields.contains_key("type") {
                    self.visit(value);
                }
                for child in fields.values() {
                    if child.is_object() || child.is_array() {
                        self.walk(child);
                    }
                }
            }
            _ => {}
        }
    }

    fn reported(&self, node: &Value, prefix: &'static str) -> bool {
        span(node).is_some_and(|span| self.reported.contains(&(span, prefix)))
    }

    /// Mark `sender` when `report` (the node upstream reports) was reported.
    fn sender(&mut self, report: Option<&Value>, prefix: &'static str, sender: Option<&Value>) {
        if let (Some(report), Some(sender)) = (report, sender)
            && self.reported(report, prefix)
        {
            self.mark(sender);
        }
    }

    fn visit(&mut self, node: &Value) {
        match kind(node) {
            "VariableDeclarator" => self.sender(Some(node), ASSIGNED, node.get("init")),
            "AssignmentExpression" | "AssignmentPattern" => {
                self.sender(Some(node), ASSIGNED, node.get("right"));
            }
            "PropertyDefinition" | "AccessorProperty" => {
                self.sender(Some(node), ASSIGNED, node.get("value"));
            }
            "TSAsExpression" | "TSTypeAssertion" => {
                self.sender(Some(node), ASSERTED, node.get("expression"));
            }
            "ReturnStatement" => self.sender(Some(node), RETURNED, node.get("argument")),
            "ArrowFunctionExpression" => {
                let body = node
                    .get("body")
                    .filter(|body| kind(body) != "BlockStatement");
                self.sender(body, RETURNED, body);
            }
            // A spread argument is left unmarked: a reported tuple spread is
            // not marked upstream, and an array literal spread is a tuple.
            "CallExpression" | "NewExpression" => {
                for argument in list(node, "arguments").filter(|a| kind(a) != "SpreadElement") {
                    self.sender(Some(argument), ARGUMENT, Some(argument));
                }
            }
            "TaggedTemplateExpression" => {
                if let Some(quasi) = node.get("quasi") {
                    for expression in list(quasi, "expressions") {
                        self.sender(Some(expression), ARGUMENT, Some(expression));
                    }
                }
            }
            "ArrayExpression" => {
                let checked = span(node).is_some_and(|span| self.marked.contains(&span));
                for element in list(node, "elements") {
                    if !checked {
                        self.sender(Some(element), ASSIGNED, Some(element));
                    } else if kind(element) != "AssignmentExpression"
                        && let Some(span) = span(element)
                    {
                        // An assignment element reports itself with the same
                        // packet, so its report is never the element check's.
                        self.dropped.insert((span, ASSIGNED));
                    }
                }
            }
            "ObjectExpression" => {
                let checked = span(node).is_some_and(|span| self.marked.contains(&span));
                for property in list(node, "properties") {
                    if checked {
                        if let Some(span) = span(property) {
                            self.dropped.insert((span, ASSIGNED));
                        }
                    } else {
                        let sender = if kind(property) == "SpreadElement" {
                            property.get("argument")
                        } else {
                            property.get("value")
                        };
                        self.sender(Some(property), ASSIGNED, sender);
                    }
                }
            }
            _ => {}
        }
    }

    /// typescript-eslint `markChecked`.
    fn mark(&mut self, node: &Value) {
        match kind(node) {
            "ArrayExpression" => {
                if let Some(span) = span(node) {
                    self.marked.insert(span);
                }
                for element in list(node, "elements") {
                    self.mark(element);
                }
            }
            "ObjectExpression" => {
                if let Some(span) = span(node) {
                    self.marked.insert(span);
                }
                for property in list(node, "properties") {
                    self.mark(property);
                }
            }
            "ConditionalExpression" => {
                for field in ["consequent", "alternate"] {
                    if let Some(child) = node.get(field) {
                        self.mark(child);
                    }
                }
            }
            "LogicalExpression" => {
                for field in ["left", "right"] {
                    if let Some(child) = node.get(field) {
                        self.mark(child);
                    }
                }
            }
            "Property" => {
                if let Some(value) = node.get("value") {
                    self.mark(value);
                }
            }
            "SpreadElement" => {
                if let Some(argument) = node.get("argument") {
                    self.mark(argument);
                }
            }
            // typescript-estree has no parenthesized nodes.
            "TSAsExpression"
            | "TSNonNullExpression"
            | "TSSatisfiesExpression"
            | "TSTypeAssertion"
            | "ParenthesizedExpression" => {
                if let Some(expression) = node.get("expression") {
                    self.mark(expression);
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Report, retain_unchecked};

    const ASSIGNED: &str =
        "The value assigned here does not have a shared enum type with the expected enum 'Fruit'.";

    fn program(source: &str) -> serde_json::Value {
        super::super::tree::collect(source, false, &[]).program
    }

    fn report(source: &str, text: &str, message: &str) -> Report {
        let start = source.find(text).expect("reported text");
        Report {
            start: start as u32,
            end: (start + text.len()) as u32,
            message: message.into(),
        }
    }

    fn kept(source: &str, mut reports: Vec<Report>) -> Vec<String> {
        retain_unchecked(&program(source), &mut reports);
        reports
            .iter()
            .filter_map(|report| source.get(report.start as usize..report.end as usize))
            .map(str::to_string)
            .collect()
    }

    #[test]
    fn a_reported_value_hides_the_literals_inside_it() {
        let source = "const boxes: T = cond ? [{ fruit: 1 }] as const : other;\n";
        let reports = vec![
            report(
                source,
                "boxes: T = cond ? [{ fruit: 1 }] as const : other",
                ASSIGNED,
            ),
            report(source, "fruit: 1", ASSIGNED),
        ];
        assert_eq!(
            kept(source, reports),
            ["boxes: T = cond ? [{ fruit: 1 }] as const : other"]
        );
    }

    #[test]
    fn unreported_values_and_assignment_elements_keep_their_reports() {
        let source = "const box: T = { fruit: 1 };\nconst list: U = [a = 1] as const;\n";
        let reports = vec![
            report(source, "fruit: 1", ASSIGNED),
            report(source, "list: U = [a = 1] as const", ASSIGNED),
            report(source, "a = 1", ASSIGNED),
        ];
        assert_eq!(
            kept(source, reports),
            ["fruit: 1", "list: U = [a = 1] as const", "a = 1"]
        );
    }
}
