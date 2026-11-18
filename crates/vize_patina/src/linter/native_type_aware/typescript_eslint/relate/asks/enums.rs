//! Syntax pairs for deep `no-unsafe-enum-assignment` checker relations.
use super::super::EnumTarget;
use super::{Ancestor, Walk, key, kind, span};
use crate::linter::corsa_session::{
    EnumCallArgument, EnumCallAsk, EnumCheckAsk, EnumReceiver, HeritageSource, Span,
};
use serde_json::Value;
use vize_l0::String as CompactString;

const ASSIGNMENT: &str = "The value assigned here does not have a shared enum type with the expected enum {{enumNames}}.";
const ARGUMENT: &str = "The argument passed here does not have a shared enum type with the expected enum {{enumNames}}.";
const ASSERTION: &str = "The value asserted here does not have a shared enum type with the expected enum {{enumNames}}.";
const RETURN: &str = "The value returned here does not have a shared enum type with the expected enum {{enumNames}}.";
const ACCESS: &str = "The computed key used here does not have a shared enum type with the expected enum {{enumNames}}.";

impl<'v> Walk<'_, '_, 'v> {
    pub(super) fn enum_assignment(&mut self, value: &Value, ancestors: &[Ancestor<'_>]) {
        match kind(value) {
            "VariableDeclarator" => {
                let id = value.get("id");
                let pattern =
                    id.is_some_and(|id| matches!(kind(id), "ArrayPattern" | "ObjectPattern"));
                // typescript-eslint compares the initializer with the type of
                // `node.id`. An unannotated pattern takes its initializer's
                // own type, so no enum can be lost; its contextual type is
                // only the pattern's implied shape, not a receiver.
                if pattern
                    && id
                        .and_then(|id| id.get("typeAnnotation"))
                        .is_none_or(Value::is_null)
                {
                    return;
                }
                self.enum_pair(
                    value,
                    value.get("init"),
                    if pattern { value.get("init") } else { id },
                    pattern,
                    ASSIGNMENT,
                );
            }
            "AssignmentExpression"
                if matches!(
                    value.get("operator").and_then(Value::as_str),
                    Some("=" | "&&=" | "&=" | "??=" | "^=" | "|=" | "||=")
                ) =>
            {
                self.enum_pair(
                    value,
                    value.get("right"),
                    value.get("left"),
                    false,
                    ASSIGNMENT,
                );
            }
            "AssignmentPattern" => self.enum_pair(
                value,
                value.get("right"),
                value.get("left"),
                false,
                ASSIGNMENT,
            ),
            "TSAsExpression" | "TSTypeAssertion" => {
                self.enum_pair(
                    value,
                    value.get("expression"),
                    Some(value),
                    false,
                    ASSERTION,
                );
            }
            "PropertyDefinition" | "AccessorProperty" => {
                self.enum_pair(
                    value,
                    value.get("value"),
                    value.get("key"),
                    false,
                    ASSIGNMENT,
                );
                self.enum_heritage(value, ancestors);
            }
            "ReturnStatement" => self.enum_return(value, value.get("argument"), ancestors),
            "ArrowFunctionExpression" => {
                let body = value.get("body");
                if body.is_some_and(|body| kind(body) != "BlockStatement") {
                    self.enum_return(value, body, ancestors);
                }
            }
            "ArrayExpression" => self.enum_array(value),
            "ObjectExpression" => self.enum_object(value),
            "CallExpression" | "NewExpression" | "TaggedTemplateExpression" => {
                self.enum_call(value);
            }
            "MemberExpression" if value.get("computed").and_then(Value::as_bool) == Some(true) => {
                self.enum_computed(value);
            }
            _ => {}
        }
    }

    fn enum_pair(
        &mut self,
        listener: &Value,
        sender: Option<&Value>,
        receiver: Option<&Value>,
        contextual: bool,
        message: &'static str,
    ) {
        self.enum_pair_at(listener, listener, sender, receiver, contextual, message);
    }

    fn enum_pair_at(
        &mut self,
        listener: &Value,
        report: &Value,
        sender: Option<&Value>,
        receiver: Option<&Value>,
        contextual: bool,
        message: &'static str,
    ) {
        let (Some(sender), Some(receiver), Some(target_key), Some(target_span)) =
            (sender, receiver, key(listener), span(report))
        else {
            return;
        };
        if matches!(kind(sender), "ArrayExpression" | "ObjectExpression") {
            return;
        }
        let Some(ask) = (|| {
            let sender_span = (self.map)(span(sender)?)?;
            let receiver_span = (self.map)(span(receiver)?)?;
            let safe_operands = self.mapped_operands(sender)?;
            Some(EnumCheckAsk::Pair {
                sender: sender_span,
                receiver: if contextual {
                    EnumReceiver::Contextual(receiver_span)
                } else {
                    EnumReceiver::TypeAt(receiver_span)
                },
                safe_operands,
            })
        })() else {
            return;
        };
        self.pending
            .enum_checks
            .push(EnumTarget::new(target_key, target_span, message));
        self.pending.request.enum_checks.push(ask);
    }

    /// The generated spans of `value`'s bit-flag leaves (empty unless it is
    /// a bitwise expression); `None` when one does not map.
    fn mapped_operands(&self, value: &Value) -> Option<Vec<Span>> {
        bitwise_operands(value)
            .into_iter()
            .map(|operand| (self.map)(operand))
            .collect()
    }

    fn enum_array(&mut self, value: &Value) {
        for element in value
            .get("elements")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
            .filter(|element| !element.is_null())
        {
            self.enum_pair_at(
                value,
                element,
                Some(element),
                Some(element),
                true,
                ASSIGNMENT,
            );
        }
    }

    fn enum_object(&mut self, value: &Value) {
        for property in value
            .get("properties")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if kind(property) == "SpreadElement" {
                self.enum_pair_at(
                    value,
                    property,
                    property.get("argument"),
                    Some(value),
                    true,
                    ASSIGNMENT,
                );
            } else if kind(property) == "Property" {
                self.enum_pair_at(
                    value,
                    property,
                    property.get("value"),
                    property.get("value"),
                    true,
                    ASSIGNMENT,
                );
            }
        }
    }

    fn enum_return(
        &mut self,
        listener: &Value,
        returned: Option<&Value>,
        ancestors: &[Ancestor<'_>],
    ) {
        let Some(returned) = returned else {
            return;
        };
        if matches!(kind(returned), "ArrayExpression" | "ObjectExpression") {
            return;
        }
        let arrow = kind(listener) == "ArrowFunctionExpression";
        let function = if arrow {
            Some(listener)
        } else {
            ancestors.iter().rev().find_map(|(node, _)| {
                matches!(
                    kind(node),
                    "ArrowFunctionExpression"
                        | "FunctionExpression"
                        | "FunctionDeclaration"
                        | "TSDeclareFunction"
                )
                .then_some(*node)
            })
        };
        // A concise arrow reports its body (`checkReturn(node.body, node.body)`).
        let report = if arrow { returned } else { listener };
        let (Some(function), Some(target_key), Some(target_span)) =
            (function, key(listener), span(report))
        else {
            return;
        };
        let Some(ask) = (|| {
            Some(EnumCheckAsk::Return {
                function: (self.map)(span(function)?)?,
                returned: (self.map)(span(returned)?)?,
                receiver: function
                    .get("returnType")
                    .and_then(|annotation| annotation.get("typeAnnotation"))
                    .and_then(span)
                    .and_then(|span| (self.map)(span)),
                is_async: function
                    .get("async")
                    .and_then(Value::as_bool)
                    .unwrap_or(false),
                safe_operands: self.mapped_operands(returned)?,
            })
        })() else {
            return;
        };
        self.pending
            .enum_checks
            .push(EnumTarget::new(target_key, target_span, RETURN));
        self.pending.request.enum_checks.push(ask);
    }

    fn enum_heritage(&mut self, member: &Value, ancestors: &[Ancestor<'_>]) {
        if member.get("static").and_then(Value::as_bool) == Some(true) {
            return;
        }
        let Some(value) = member.get("value").filter(|value| !value.is_null()) else {
            return;
        };
        let Some(name) = static_member_name(member) else {
            return;
        };
        let Some(class) = ancestors.iter().rev().find_map(|(node, _)| {
            matches!(kind(node), "ClassDeclaration" | "ClassExpression").then_some(*node)
        }) else {
            return;
        };
        let mut sources = Vec::new();
        if class
            .get("superClass")
            .is_some_and(|value| !value.is_null())
            && let Some(mapped) = span(class).and_then(|span| (self.map)(span))
        {
            sources.push(HeritageSource {
                span: mapped,
                base_types: true,
            });
        }
        for implemented in class
            .get("implements")
            .and_then(Value::as_array)
            .into_iter()
            .flatten()
        {
            if let Some(mapped) = span(implemented).and_then(|span| (self.map)(span)) {
                sources.push(HeritageSource {
                    span: mapped,
                    base_types: false,
                });
            }
        }
        let (Some(target_key), Some(target_span), Some(sender), Some(safe_operands)) = (
            key(member),
            span(member),
            span(value).and_then(|span| (self.map)(span)),
            self.mapped_operands(value),
        ) else {
            return;
        };
        if sources.is_empty() {
            return;
        }
        self.pending
            .enum_checks
            .push(EnumTarget::new(target_key, target_span, ASSIGNMENT));
        self.pending
            .request
            .enum_checks
            .push(EnumCheckAsk::Heritage {
                sender,
                member: name,
                sources,
                safe_operands,
            });
    }

    fn enum_call(&mut self, value: &Value) {
        let tagged = kind(value) == "TaggedTemplateExpression";
        let arguments = if tagged {
            value
                .get("quasi")
                .and_then(|quasi| quasi.get("expressions"))
        } else {
            value.get("arguments")
        }
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .unwrap_or_default();
        let (Some(target_key), Some(call)) =
            (key(value), span(value).and_then(|span| (self.map)(span)))
        else {
            return;
        };
        let mut asks = Vec::new();
        let mut targets = Vec::new();
        for argument in arguments {
            let spread = kind(argument) == "SpreadElement";
            let expression = if spread {
                argument.get("argument")
            } else {
                Some(argument)
            };
            let safe_operands = if spread {
                Some(Vec::new())
            } else {
                self.mapped_operands(argument)
            };
            let (Some(node_span), Some(expression_span), Some(report_span), Some(safe_operands)) = (
                span(argument).and_then(|span| (self.map)(span)),
                expression.and_then(span).and_then(|span| (self.map)(span)),
                span(argument),
                safe_operands,
            ) else {
                return;
            };
            asks.push(EnumCallArgument {
                node: node_span,
                expression: expression_span,
                spread,
                literal: matches!(kind(argument), "ArrayExpression" | "ObjectExpression"),
                safe_operands,
            });
            targets.push(EnumTarget::new(target_key, report_span, ARGUMENT));
        }
        if asks.is_empty() {
            return;
        }
        self.pending.request.enum_calls.push(EnumCallAsk {
            call,
            arguments: asks,
            tagged_template: tagged,
        });
        self.pending.enum_calls.push(targets);
    }

    fn enum_computed(&mut self, value: &Value) {
        let (Some(target_key), Some(target_span), Some(object), Some(property)) = (
            key(value),
            value.get("property").and_then(span),
            value
                .get("object")
                .and_then(span)
                .and_then(|span| (self.map)(span)),
            value
                .get("property")
                .and_then(span)
                .and_then(|span| (self.map)(span)),
        ) else {
            return;
        };
        self.pending
            .enum_checks
            .push(EnumTarget::new(target_key, target_span, ACCESS));
        self.pending
            .request
            .enum_checks
            .push(EnumCheckAsk::Computed { object, property });
    }
}

fn static_member_name(member: &Value) -> Option<CompactString> {
    if member.get("computed").and_then(Value::as_bool) == Some(true) {
        return None;
    }
    let key = member.get("key")?;
    // `#fruit` is not the inherited `fruit`: private names never implement
    // or override a heritage member.
    if kind(key) == "PrivateIdentifier" {
        return None;
    }
    key.get("name")
        .or_else(|| key.get("value"))
        .and_then(Value::as_str)
        .map(CompactString::from)
}

fn bitwise_operands(value: &Value) -> Vec<Span> {
    fn collect(value: &Value, out: &mut Vec<Span>) -> bool {
        match kind(value) {
            "BinaryExpression"
                if matches!(
                    value.get("operator").and_then(Value::as_str),
                    Some("&" | "^" | "|")
                ) =>
            {
                value.get("left").is_some_and(|left| collect(left, out))
                    && value.get("right").is_some_and(|right| collect(right, out))
            }
            "UnaryExpression" if value.get("operator").and_then(Value::as_str) == Some("~") => {
                value
                    .get("argument")
                    .is_some_and(|argument| collect(argument, out))
            }
            _ => span(value).is_some_and(|span| {
                out.push(span);
                true
            }),
        }
    }
    let mut out = Vec::new();
    if collect(value, &mut out) && matches!(kind(value), "BinaryExpression" | "UnaryExpression") {
        out
    } else {
        Vec::new()
    }
}
