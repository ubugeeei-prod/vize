//! Per-rule relation asks: assertions, non-null, accessor pairs, reduce and `this` returns.
use super::super::{AssertionTarget, NonNullTarget};
use super::context::{
    context, generic_callees, member_name, narrowed_declaration, return_arguments,
};
use super::{Ancestor, Walk, annotation_type, is_assertion, key, kind, literal, span};
use crate::linter::corsa_session::{
    AssertionAsk, AssertionContext, ContextSource, Literal, NonNullAsk, PropertyContext, ReturnAsk,
};
use serde_json::{Value, json};

impl<'v> Walk<'_, '_, 'v> {
    pub(super) fn shape(&mut self, value: &Value) {
        let (Some(key), Some(range)) = (key(value), span(value).and_then(|span| (self.map)(span)))
        else {
            return;
        };
        self.pending.shapes.push(key);
        self.pending.request.shapes.push(range);
    }

    pub(super) fn assertion(&mut self, value: &Value, ancestors: &[Ancestor<'_>]) {
        let (Some(key), Some(expression)) = (key(value), value.get("expression")) else {
            return;
        };
        let narrowed_declaration = narrowed_declaration(expression, ancestors);
        let ask = (|| {
            let node = (self.map)(span(value)?)?;
            let expression_span = (self.map)(span(expression)?)?;
            let intersection =
                annotation_type(value).is_some_and(|ty| kind(ty) == "TSIntersectionType");
            let mut original = None;
            if is_assertion(expression) {
                let mut inner = expression;
                while is_assertion(inner) {
                    inner = inner.get("expression")?;
                }
                original = Some((self.map)(span(inner)?)?);
            }
            let iife = match expression.get("callee") {
                Some(callee)
                    if kind(expression) == "CallExpression"
                        && matches!(
                            kind(callee),
                            "ArrowFunctionExpression" | "FunctionExpression"
                        ) =>
                {
                    let annotated = callee.get("returnType").is_some_and(|ty| !ty.is_null());
                    Some(((self.map)(span(callee)?)?, annotated))
                }
                _ => None,
            };
            let original_node = {
                let mut inner = expression;
                while is_assertion(inner) {
                    inner = inner.get("expression")?;
                }
                inner
            };
            Some(AssertionAsk {
                node,
                expression: expression_span,
                original,
                intersection,
                literal: literal(expression),
                iife,
                context: self.assertion_context(value, expression, original_node, ancestors)?,
            })
        })();
        match ask {
            Some(ask) => {
                self.pending.assertions.push(AssertionTarget {
                    key,
                    narrowed_declaration,
                });
                self.pending.request.assertions.push(ask);
            }
            None => self
                .pending
                .fixed
                .push((key, "__typeFactsMissing", json!(true))),
        }
    }

    /// The syntax half of `shouldSkipContextualTypeFallback` and of the
    /// contextual verdict; the checker decides the rest.
    pub(super) fn assertion_context(
        &self,
        value: &Value,
        expression: &Value,
        original: &Value,
        ancestors: &[Ancestor<'_>],
    ) -> Option<AssertionContext> {
        let &(parent, field) = ancestors.last()?;
        let grandparent = ancestors.iter().rev().nth(1).map(|(node, _)| *node);
        let skip_parent = matches!(
            kind(parent),
            "TSAsExpression" | "TSTypeAssertion" | "SpreadElement" | "TSSatisfiesExpression"
        );
        let template_with_expressions = kind(expression) == "TemplateLiteral"
            && expression
                .get("expressions")
                .and_then(Value::as_array)
                .is_some_and(|expressions| !expressions.is_empty());
        let destructuring_init = kind(parent) == "VariableDeclarator"
            && field == "init"
            && parent
                .get("id")
                .is_some_and(|id| matches!(kind(id), "ObjectPattern" | "ArrayPattern"));
        let assignment_right = kind(parent) == "AssignmentExpression" && field == "right";
        let non_statement_assignment =
            assignment_right && grandparent.is_none_or(|node| kind(node) != "ExpressionStatement");
        let logical_assignment = assignment_right
            && matches!(
                parent.get("operator").and_then(Value::as_str),
                Some("&&=" | "||=" | "??=")
            );
        let call_argument =
            matches!(kind(parent), "CallExpression" | "NewExpression") && field == "arguments";
        let overload = if call_argument {
            let own = span(value)?;
            let index = parent
                .get("arguments")?
                .as_array()?
                .iter()
                .position(|argument| span(argument) == Some(own))?;
            Some(((self.map)(span(parent.get("callee")?)?)?, index))
        } else {
            None
        };
        let property = if kind(parent) == "Property" && field == "value" {
            let object = grandparent.filter(|node| kind(node) == "ObjectExpression");
            match object {
                Some(object) => {
                    let mut above = ancestors.iter().rev().skip(2).map(|(node, _)| *node);
                    let object_parent = above.next();
                    let satisfies = object_parent.is_some_and(|node| {
                        kind(node) == "TSSatisfiesExpression"
                            || (kind(node) == "CallExpression"
                                && above
                                    .next()
                                    .is_some_and(|node| kind(node) == "TSSatisfiesExpression"))
                    });
                    Some(PropertyContext {
                        object: (self.map)(span(object)?)?,
                        satisfies,
                    })
                }
                None => None,
            }
        } else {
            None
        };
        let nullish_literal = (kind(expression) == "Literal"
            && expression.get("value").is_some_and(Value::is_null)
            && expression.get("regex").is_none()
            && expression.get("bigint").is_none())
            || (kind(expression) == "Identifier"
                && expression.get("name").and_then(Value::as_str) == Some("undefined"));
        let annotation = annotation_type(value);
        let const_assertion = annotation.is_some_and(|ty| {
            kind(ty) == "TSTypeReference"
                && ty
                    .get("typeName")
                    .and_then(|name| name.get("name"))
                    .and_then(Value::as_str)
                    == Some("const")
        });
        Some(AssertionContext {
            skip: skip_parent
                || template_with_expressions
                || kind(expression) == "ArrayExpression"
                || destructuring_init
                || non_statement_assignment
                || logical_assignment,
            skip_parent,
            parent_logical: kind(parent) == "LogicalExpression",
            parent_property: kind(parent) == "Property",
            literal_original: literal(original) != Literal::No,
            generic_callees: generic_callees(value, ancestors)
                .into_iter()
                .map(|callee| (self.map)(callee))
                .collect::<Option<Vec<_>>>()?,
            overload,
            call_argument,
            property,
            nullish_literal,
            const_assertion,
            narrowed_declaration: narrowed_declaration(expression, ancestors),
        })
    }

    pub(super) fn non_null(&mut self, value: &Value, ancestors: &[Ancestor<'_>]) {
        let (Some(key), Some(expression)) = (key(value), value.get("expression")) else {
            return;
        };
        if let Some((parent, field)) = ancestors.last()
            && kind(parent) == "AssignmentExpression"
        {
            if let Some(operator) = parent.get("operator").and_then(Value::as_str) {
                self.pending
                    .fixed
                    .push((key, "__parentOperator", json!(operator)));
            }
            self.pending
                .fixed
                .push((key, "__isAssignmentLeft", json!(*field == "left")));
        }
        let ask = (|| {
            let expression_span = (self.map)(span(expression)?)?;
            let context = match context(value, ancestors) {
                Some(ContextSource::Checker(span)) => {
                    Some(ContextSource::Checker((self.map)(span)?))
                }
                Some(ContextSource::TypeAt(span)) => Some(ContextSource::TypeAt((self.map)(span)?)),
                None => None,
            };
            Some(NonNullAsk {
                expression: expression_span,
                context,
                declared: None,
            })
        })();
        let Some(ask) = ask else {
            self.pending
                .fixed
                .push((key, "__typeFactsMissing", json!(true)));
            return;
        };
        let index = self.pending.request.non_null.len();
        self.pending.request.non_null.push(ask);
        self.pending.non_null.push(NonNullTarget {
            key,
            used_before_assigned: false,
        });
        if kind(expression) == "Identifier"
            && let Some(span) = span(expression)
        {
            self.identifiers.push((index, span));
        }
    }

    /// `related-getter-setter-pairs`: `isTypeAssignableTo(typeAt(getter),
    /// typeAt(setter.params[0]))` for each same-named accessor pair.
    pub(super) fn accessor_pairs(&mut self, value: &Value) {
        let members = value
            .get("body")
            .or_else(|| value.get("members"))
            .and_then(Value::as_array)
            .map(Vec::as_slice)
            .unwrap_or_default();
        let mut getters = Vec::new();
        let mut setters = Vec::new();
        for member in members {
            if !matches!(kind(member), "MethodDefinition" | "TSMethodSignature") {
                continue;
            }
            let function = if kind(member) == "MethodDefinition" {
                member.get("value")
            } else {
                Some(member)
            };
            let (Some(function), Some(name)) = (function, member_name(member)) else {
                continue;
            };
            match member.get("kind").and_then(Value::as_str) {
                Some("get") if function.get("returnType").is_some_and(|ty| !ty.is_null()) => {
                    getters.push((name, member, function));
                }
                Some("set") => {
                    if let Some([parameter]) = function
                        .get("params")
                        .and_then(Value::as_array)
                        .map(Vec::as_slice)
                    {
                        setters.push((name, parameter));
                    }
                }
                _ => {}
            }
        }
        for (name, member, function) in getters {
            let Some((_, parameter)) = setters.iter().rev().find(|(setter, _)| *setter == name)
            else {
                continue;
            };
            let Some(key) = key(member) else {
                continue;
            };
            let pair = (|| Some(((self.map)(span(function)?)?, (self.map)(span(parameter)?)?)))();
            match pair {
                Some(pair) => {
                    self.pending
                        .assignable
                        .push((key, "__getterTypeAssignableToSetter"));
                    self.pending.request.assignable.push(pair);
                }
                None => {
                    self.pending
                        .fixed
                        .push((key, "__getterTypeAssignableToSetter", json!(true)))
                }
            }
        }
    }

    pub(super) fn reduce(&mut self, value: &Value) {
        let Some(callee) = value
            .get("callee")
            .filter(|callee| kind(callee) == "MemberExpression")
        else {
            return;
        };
        let property = callee.get("property");
        let named_reduce = property.is_some_and(|property| {
            property.get("name").and_then(Value::as_str) == Some("reduce")
                || property.get("value").and_then(Value::as_str) == Some("reduce")
        });
        let Some(second) = value
            .get("arguments")
            .and_then(Value::as_array)
            .and_then(|arguments| arguments.get(1))
        else {
            return;
        };
        if !named_reduce || !is_assertion(second) {
            return;
        }
        if let Some(object) = callee.get("object") {
            self.shape(object);
        }
        let (Some(key), Some(expression)) = (key(second), second.get("expression")) else {
            return;
        };
        // The assertion's own type is the asserted type.
        let pair = (|| Some(((self.map)(span(expression)?)?, (self.map)(span(second)?)?)))();
        match pair {
            Some(pair) => {
                self.pending
                    .assignable
                    .push((key, "__initializerAssignable"));
                self.pending.request.assignable.push(pair);
            }
            None => self
                .pending
                .fixed
                .push((key, "__typeFactsMissing", json!(true))),
        }
    }

    pub(super) fn returns(&mut self, member: &Value, ancestors: &[Ancestor<'_>]) {
        let Some(function) = member.get("value").filter(|value| {
            matches!(
                kind(value),
                "FunctionExpression" | "ArrowFunctionExpression"
            )
        }) else {
            return;
        };
        if function.get("returnType").is_none_or(Value::is_null) {
            return;
        }
        let Some(class) = ancestors
            .iter()
            .rev()
            .nth(1)
            .map(|(class, _)| *class)
            .filter(|class| matches!(kind(class), "ClassDeclaration" | "ClassExpression"))
        else {
            return;
        };
        let (Some(key), Some(body)) = (key(function), function.get("body")) else {
            return;
        };
        let mut expressions = Vec::new();
        if kind(body) == "BlockStatement" {
            return_arguments(body, &mut expressions);
        } else {
            expressions.push(body);
        }
        let ask = (|| {
            let class = (self.map)(span(class)?)?;
            let expressions = expressions
                .iter()
                .map(|expression| span(expression).and_then(|span| (self.map)(span)))
                .collect::<Option<Vec<_>>>()?;
            Some(ReturnAsk { class, expressions })
        })();
        match ask {
            Some(ask) => {
                self.pending.returns.push(key);
                self.pending.request.returns.push(ask);
            }
            None => self
                .pending
                .fixed
                .push((key, "__typeFactsMissing", json!(true))),
        }
    }
}
