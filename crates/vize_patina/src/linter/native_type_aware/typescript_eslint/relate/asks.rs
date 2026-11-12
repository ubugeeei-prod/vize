//! Walk the script's ESTree once and turn the nodes the relation rules listen
//! to into checker asks, with the parent-dependent facts decided here.
use super::binding::{self, Binding};
use super::{AssertionTarget, Key, NonNullTarget, Pending};
use crate::linter::corsa_session::{
    AssertionAsk, AssertionContext, ContextSource, DeprecationAsk, DeprecationKind, Literal,
    NonNullAsk, PropertyContext, ReturnAsk, Span,
};
use serde_json::{Value, json};
use vize_l0::String as CompactString;

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
    source: &str,
    tsx: bool,
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
    let defaults = binding::type_parameter_defaults(source, tsx, &names);
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
    let bindings = binding::resolve(source, tsx, &spans);
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

    fn deprecation(&mut self, value: &Value, kind: Option<DeprecationKind>) {
        let Some(key) = key(value) else {
            return;
        };
        let ask = kind.and_then(|kind| {
            let node = (self.map)(span(value)?)?;
            Some(DeprecationAsk { node, kind })
        });
        match ask {
            Some(ask) => {
                self.pending.deprecations.push(key);
                self.pending.request.deprecations.push(ask);
            }
            None => self.pending.fixed.push((key, "__deprecated", json!(false))),
        }
    }

    /// typescript-eslint `no-deprecated`'s `Identifier`, `PrivateIdentifier`
    /// and `Super` listeners.
    fn deprecated_use(&mut self, value: &Value, ancestors: &[Ancestor<'_>]) {
        let Some(&(parent, field)) = ancestors.last() else {
            return;
        };
        let skip = match kind(parent) {
            "ExportNamedDeclaration" | "ExportAllDeclaration" => true,
            "MemberExpression" => {
                field == "property" && parent.get("computed").and_then(Value::as_bool) == Some(true)
            }
            "ExportSpecifier" => field != "exported",
            _ => false,
        };
        if skip || is_declaration(field, ancestors) || inside_import(ancestors) {
            self.deprecation(value, None);
            return;
        }
        let deprecation = (|| {
            if kind(parent) == "ExportSpecifier" {
                return Some(DeprecationKind::ExportAlias);
            }
            if let Some(call) = call_like(ancestors) {
                return Some(DeprecationKind::CallLike {
                    call: (self.map)(span(call)?)?,
                });
            }
            if kind(parent) == "JSXAttribute" {
                return None;
            }
            if kind(parent) == "Property" && kind(value) != "Super" {
                let (object, _) = *ancestors.iter().rev().nth(1)?;
                return Some(DeprecationKind::ObjectProperty {
                    object: (self.map)(span(object)?)?,
                    name: CompactString::from(value.get("name")?.as_str()?),
                });
            }
            Some(DeprecationKind::Symbol)
        })();
        self.deprecation(value, deprecation);
    }

    /// `checkMemberExpression`: `object[literal]`.
    fn deprecated_member(&mut self, value: &Value) {
        if value.get("computed").and_then(Value::as_bool) != Some(true) {
            return;
        }
        let deprecation = (|| {
            Some(DeprecationKind::Member {
                object: (self.map)(span(value.get("object")?)?)?,
                property: (self.map)(span(value.get("property")?)?)?,
            })
        })();
        self.deprecation_on(value, value.get("property"), deprecation);
    }

    /// `checkObjectLiteralProperty`: a key of an object literal names a
    /// property of the literal's contextual type.
    fn deprecated_key(&mut self, value: &Value, ancestors: &[Ancestor<'_>]) {
        let Some(&(parent, _)) = ancestors.last() else {
            return;
        };
        if kind(parent) != "ObjectExpression" {
            return;
        }
        let Some(key_node) = value.get("key") else {
            return;
        };
        let computed = value.get("computed").and_then(Value::as_bool) == Some(true);
        let name = if computed {
            None
        } else if kind(key_node) == "Identifier" {
            key_node
                .get("name")
                .and_then(Value::as_str)
                .map(CompactString::from)
        } else {
            match key_node.get("value") {
                Some(Value::String(text)) => Some(CompactString::from(text.as_str())),
                Some(Value::Number(number)) => Some(vize_l0::cstr!("{number}")),
                _ => None,
            }
        };
        let deprecation = (|| {
            let key = if computed {
                Some((self.map)(span(key_node)?)?)
            } else {
                None
            };
            if name.is_none() && key.is_none() {
                return None;
            }
            Some(DeprecationKind::ContextualProperty {
                object: (self.map)(span(parent)?)?,
                name: name.clone(),
                key,
            })
        })();
        if let (Some(key), Some(name)) = (key(value), &name) {
            self.pending
                .fixed
                .push((key, "__deprecatedName", json!(name.as_str())));
        }
        self.deprecation_on(value, Some(key_node), deprecation);
    }

    /// A deprecation reported on `node`'s `reported` child; the reported
    /// name is that child's static text.
    fn deprecation_on(
        &mut self,
        value: &Value,
        reported: Option<&Value>,
        kind: Option<DeprecationKind>,
    ) {
        if let (Some(key), Some(range)) = (key(value), reported.and_then(span)) {
            self.pending.fixed.push((
                key,
                "__deprecationReportRange",
                json!({ "start": range.0, "end": range.1 }),
            ));
        }
        self.deprecation(value, kind);
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

    fn shape(&mut self, value: &Value) {
        let (Some(key), Some(range)) = (key(value), span(value).and_then(|span| (self.map)(span)))
        else {
            return;
        };
        self.pending.shapes.push(key);
        self.pending.request.shapes.push(range);
    }

    fn assertion(&mut self, value: &Value, ancestors: &[Ancestor<'_>]) {
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
    fn assertion_context(
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

    fn non_null(&mut self, value: &Value, ancestors: &[Ancestor<'_>]) {
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
    fn accessor_pairs(&mut self, value: &Value) {
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

    fn reduce(&mut self, value: &Value) {
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

    fn returns(&mut self, member: &Value, ancestors: &[Ancestor<'_>]) {
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

/// `return` arguments of a function body, skipping nested functions.
fn return_arguments<'v>(value: &'v Value, out: &mut Vec<&'v Value>) {
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
fn narrowed_declaration(expression: &Value, ancestors: &[Ancestor<'_>]) -> bool {
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
fn context(value: &Value, ancestors: &[Ancestor<'_>]) -> Option<ContextSource> {
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
fn annotated_name(holder: &Value, name: &Value) -> Option<ContextSource> {
    annotation_type(holder)?;
    if kind(name) != "Identifier" {
        return None;
    }
    span(name).map(ContextSource::TypeAt)
}

/// typescript-eslint `no-deprecated`'s `isDeclaration`, from the parent.
fn is_declaration(field: &str, ancestors: &[Ancestor<'_>]) -> bool {
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
fn inside_import(ancestors: &[Ancestor<'_>]) -> bool {
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
fn call_like<'v>(ancestors: &[Ancestor<'v>]) -> Option<&'v Value> {
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
fn member_name(member: &Value) -> Option<CompactString> {
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
fn generic_callees(node: &Value, ancestors: &[Ancestor<'_>]) -> Vec<Span> {
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

#[cfg(test)]
mod tests {
    use super::{call_like, generic_callees, inside_import, is_declaration};
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
