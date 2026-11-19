//! Owned reads of the top-level statements an Options API component is
//! resolved from. A statement event lends its syntax only for the event, so
//! each object literal that can serve as options, a group or a returned
//! object is read into an arena once; references between them stay names
//! until the whole Program is known.

use super::super::contract::string_key;
use super::class::{self, Class};
use alloc::vec::Vec;
use oxc_ast::ast::{
    Argument, ArrayExpressionElement, BindingPattern, CallExpression, ExportDefaultDeclarationKind,
    Expression, ObjectExpression, ObjectPropertyKind, PropertyKey, Statement,
};
use oxc_span::GetSpan;
use vize_l0::Span;

/// A member key: the name text and the key as written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct Key {
    pub(super) name: Span,
    pub(super) key: Span,
}

/// A key, or a string key whose name is not its source text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Entry {
    Key(Key),
    Escaped(Span),
}

/// An option group read from an options object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Group {
    Props,
    Inject,
    Computed,
    Methods,
    Filters,
    Data,
    AsyncData,
    Setup,
    Mixins,
    Extends,
}

impl Group {
    fn named(name: &str) -> Option<Self> {
        Some(match name {
            "props" => Self::Props,
            "inject" => Self::Inject,
            "computed" => Self::Computed,
            "methods" => Self::Methods,
            "filters" => Self::Filters,
            "data" => Self::Data,
            "asyncData" => Self::AsyncData,
            "setup" => Self::Setup,
            "mixins" => Self::Mixins,
            "extends" => Self::Extends,
            _ => return None,
        })
    }
}

/// An object literal the arena holds, or a same-file name for one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Target<'a> {
    Object(usize),
    Binding(&'a str),
}

/// What an option property's value can contribute.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum Value<'a> {
    /// `props`/`inject`: array strings or object keys; `computed`/
    /// `methods`/`filters`: object keys.
    Keys(Vec<Entry>),
    Binding(&'a str),
    /// `data`/`asyncData`/`setup`: the candidates in source order; the first
    /// that resolves to an object is returned.
    Returned(Vec<Target<'a>>),
    /// `mixins` elements, or the `extends` target.
    Targets(Vec<Target<'a>>),
    Unread,
}

/// An object literal: its static keys, and its option properties.
#[derive(Debug)]
pub(super) struct Object<'a> {
    pub(super) span: Span,
    pub(super) keys: Vec<Entry>,
    pub(super) options: Vec<(Group, Value<'a>)>,
}

/// A top-level variable declarator with an identifier and an initializer.
#[derive(Debug, Clone, Copy)]
pub(super) struct Declarator<'a> {
    pub(super) name: &'a str,
    /// The initializer unwrapped to an object literal.
    pub(super) object: Option<usize>,
    /// The options the initializer declares through component wrappers.
    pub(super) options: Option<Target<'a>>,
}

/// The reads of one Program's top-level statements.
#[derive(Debug, Default)]
pub(super) struct Reads<'a> {
    pub(super) objects: Vec<Object<'a>>,
    pub(super) declarators: Vec<Declarator<'a>>,
    pub(super) exports: Vec<Export<'a>>,
}

/// An `export default` declaration.
#[derive(Debug)]
pub(super) enum Export<'a> {
    /// The options it declares, when it declares any.
    Options(Option<Target<'a>>),
    Class(Class),
}

pub(super) type Authored<'f> = &'f mut dyn FnMut(oxc_span::Span) -> Span;

impl<'a> Reads<'a> {
    pub(super) fn statement(&mut self, statement: &Statement<'a>, authored: Authored<'_>) {
        match statement {
            Statement::VariableDeclaration(declaration) => {
                for declarator in &declaration.declarations {
                    let (BindingPattern::BindingIdentifier(id), Some(init)) =
                        (&declarator.id, &declarator.init)
                    else {
                        continue;
                    };
                    let object = match wrapped(init) {
                        Expression::ObjectExpression(object) => {
                            Some(self.object(object, true, authored))
                        }
                        _ => None,
                    };
                    let options = match object {
                        Some(index) => Some(Target::Object(index)),
                        None => self.component(init, authored),
                    };
                    self.declarators.push(Declarator {
                        name: id.name.as_str(),
                        object,
                        options,
                    });
                }
            }
            Statement::ExportDefaultDeclaration(export) => {
                if let Some(read) = class::exported(&export.declaration) {
                    let read = class::read(self, read, authored);
                    self.exports.push(Export::Class(read));
                    return;
                }
                let options = match &export.declaration {
                    ExportDefaultDeclarationKind::ObjectExpression(object) => {
                        Some(Target::Object(self.object(object, true, authored)))
                    }
                    ExportDefaultDeclarationKind::CallExpression(call) => {
                        self.wrapper(call, authored)
                    }
                    ExportDefaultDeclarationKind::Identifier(id) => {
                        Some(Target::Binding(id.name.as_str()))
                    }
                    other => other
                        .as_expression()
                        .and_then(|expression| self.component(expression, authored)),
                };
                self.exports.push(Export::Options(options));
            }
            _ => {}
        }
    }

    pub(super) fn len(&self) -> [usize; 3] {
        [
            self.objects.len(),
            self.declarators.len(),
            self.exports.len(),
        ]
    }

    pub(super) fn truncate(&mut self, [objects, declarators, exports]: [usize; 3]) {
        self.objects.truncate(objects);
        self.declarators.truncate(declarators);
        self.exports.truncate(exports);
    }

    /// Reads an object literal into the arena: its static keys, and with
    /// `options` its option properties.
    pub(super) fn object(
        &mut self,
        object: &ObjectExpression<'a>,
        options: bool,
        authored: Authored<'_>,
    ) -> usize {
        let mut keys = Vec::new();
        let mut groups = Vec::new();
        for property in &object.properties {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                continue;
            };
            let Some((name, entry)) = static_key(&property.key, property.computed, authored) else {
                continue;
            };
            keys.push(entry);
            if options && let Some(group) = name.and_then(Group::named) {
                let value = self.value(group, &property.value, authored);
                groups.push((group, value));
            }
        }
        self.objects.push(Object {
            span: authored(object.span),
            keys,
            options: groups,
        });
        self.objects.len() - 1
    }

    fn keys(object: &ObjectExpression<'a>, authored: Authored<'_>) -> Vec<Entry> {
        object
            .properties
            .iter()
            .filter_map(|property| match property {
                ObjectPropertyKind::ObjectProperty(property) => {
                    static_key(&property.key, property.computed, authored).map(|(_, entry)| entry)
                }
                ObjectPropertyKind::SpreadProperty(_) => None,
            })
            .collect()
    }

    fn value(&mut self, group: Group, value: &Expression<'a>, authored: Authored<'_>) -> Value<'a> {
        match group {
            Group::Props | Group::Inject => match wrapped(value) {
                Expression::ArrayExpression(array) => {
                    let mut keys = Vec::new();
                    for element in &array.elements {
                        let ArrayExpressionElement::StringLiteral(literal) = element else {
                            continue;
                        };
                        keys.push(match string_key(literal) {
                            Some(name) => Entry::Key(Key {
                                name: authored(name),
                                key: authored(literal.span),
                            }),
                            None => Entry::Escaped(authored(literal.span)),
                        });
                    }
                    Value::Keys(keys)
                }
                Expression::ObjectExpression(object) => Value::Keys(Self::keys(object, authored)),
                Expression::Identifier(id) => Value::Binding(id.name.as_str()),
                _ => Value::Unread,
            },
            Group::Computed | Group::Methods | Group::Filters => match value {
                Expression::Identifier(id) => Value::Binding(id.name.as_str()),
                other => match wrapped(other) {
                    Expression::ObjectExpression(object) => {
                        Value::Keys(Self::keys(object, authored))
                    }
                    _ => Value::Unread,
                },
            },
            Group::Data | Group::AsyncData | Group::Setup => {
                let mut candidates = Vec::new();
                self.returned(value, authored, &mut candidates);
                Value::Returned(candidates)
            }
            Group::Mixins => {
                let mut targets = Vec::new();
                if let Some(array) = array(value) {
                    for element in &array.elements {
                        if let Some(target) = element
                            .as_expression()
                            .and_then(|element| self.target(element, authored))
                        {
                            targets.push(target);
                        }
                    }
                }
                Value::Targets(targets)
            }
            Group::Extends => Value::Targets(self.target(value, authored).into_iter().collect()),
        }
    }

    /// The objects a `data`/`asyncData`/`setup` value can return: a concise
    /// arrow body, each `return` of a function body, an object, or a name.
    fn returned(
        &mut self,
        value: &Expression<'a>,
        authored: Authored<'_>,
        out: &mut Vec<Target<'a>>,
    ) {
        let statements = match value {
            Expression::ArrowFunctionExpression(arrow) if arrow.expression => {
                if let Some(Statement::ExpressionStatement(body)) = arrow.body.statements.first() {
                    self.candidate(&body.expression, authored, out);
                }
                return;
            }
            Expression::ArrowFunctionExpression(arrow) => &arrow.body.statements,
            Expression::FunctionExpression(function) => match &function.body {
                Some(body) => &body.statements,
                None => return,
            },
            Expression::ObjectExpression(object) => {
                let index = self.object(object, false, authored);
                out.push(Target::Object(index));
                return;
            }
            Expression::Identifier(id) => {
                out.push(Target::Binding(id.name.as_str()));
                return;
            }
            Expression::ParenthesizedExpression(inner) => {
                return self.returned(&inner.expression, authored, out);
            }
            Expression::TSAsExpression(inner) => {
                return self.returned(&inner.expression, authored, out);
            }
            Expression::TSSatisfiesExpression(inner) => {
                return self.returned(&inner.expression, authored, out);
            }
            Expression::TSNonNullExpression(inner) => {
                return self.returned(&inner.expression, authored, out);
            }
            _ => return,
        };
        for statement in statements {
            if let Statement::ReturnStatement(ret) = statement
                && let Some(argument) = &ret.argument
            {
                self.candidate(argument, authored, out);
            }
        }
    }

    /// A returned name, or a returned object inside wrappers.
    fn candidate(
        &mut self,
        expression: &Expression<'a>,
        authored: Authored<'_>,
        out: &mut Vec<Target<'a>>,
    ) {
        if let Expression::Identifier(id) = expression {
            out.push(Target::Binding(id.name.as_str()));
        } else if let Expression::ObjectExpression(object) = wrapped(expression) {
            let index = self.object(object, false, authored);
            out.push(Target::Object(index));
        }
    }

    /// A `mixins` element or the `extends` value.
    fn target(
        &mut self,
        expression: &Expression<'a>,
        authored: Authored<'_>,
    ) -> Option<Target<'a>> {
        match wrapped(expression) {
            Expression::ObjectExpression(object) => {
                Some(Target::Object(self.object(object, true, authored)))
            }
            Expression::Identifier(id) => Some(Target::Binding(id.name.as_str())),
            _ => None,
        }
    }

    /// The options a component expression declares: an object literal, a
    /// `defineComponent(..)`/`Vue.extend(..)` call's first argument, or a
    /// same-file component name.
    fn component(
        &mut self,
        expression: &Expression<'a>,
        authored: Authored<'_>,
    ) -> Option<Target<'a>> {
        match wrapped(expression) {
            Expression::ObjectExpression(object) => {
                Some(Target::Object(self.object(object, true, authored)))
            }
            Expression::CallExpression(call) => self.wrapper(call, authored),
            Expression::Identifier(id) => Some(Target::Binding(id.name.as_str())),
            _ => None,
        }
    }

    fn wrapper(&mut self, call: &CallExpression<'a>, authored: Authored<'_>) -> Option<Target<'a>> {
        if !is_component_wrapper(&call.callee) {
            return None;
        }
        match call.arguments.first()? {
            Argument::SpreadElement(_) => None,
            argument => self.component(argument.to_expression(), authored),
        }
    }
}

/// An identifier key, or a string literal's contents, with the key name;
/// an escaped string key has no name.
pub(super) fn static_key<'a>(
    key: &PropertyKey<'a>,
    computed: bool,
    authored: Authored<'_>,
) -> Option<(Option<&'a str>, Entry)> {
    let key_span = authored(key.span());
    match key {
        _ if computed => None,
        PropertyKey::StaticIdentifier(id) => Some((
            Some(id.name.as_str()),
            Entry::Key(Key {
                name: authored(id.span),
                key: key_span,
            }),
        )),
        PropertyKey::StringLiteral(literal) => Some(match string_key(literal) {
            Some(name) => (
                Some(literal.value.as_str()),
                Entry::Key(Key {
                    name: authored(name),
                    key: key_span,
                }),
            ),
            None => (None, Entry::Escaped(key_span)),
        }),
        _ => None,
    }
}

/// `defineComponent`, `_defineComponent` and a named `extend` import;
/// `<object>.defineComponent` and `Vue.extend`.
fn is_component_wrapper(callee: &Expression<'_>) -> bool {
    match callee {
        Expression::Identifier(id) => {
            matches!(
                id.name.as_str(),
                "defineComponent" | "_defineComponent" | "extend"
            )
        }
        Expression::StaticMemberExpression(member) => match member.property.name.as_str() {
            "defineComponent" | "_defineComponent" => true,
            "extend" => matches!(
                &member.object,
                Expression::Identifier(id) if matches!(id.name.as_str(), "Vue" | "_Vue")
            ),
            _ => false,
        },
        _ => false,
    }
}

/// The expression inside parentheses and `as`, `satisfies` and `!`. Vue
/// reads only a bare options object; the compiler has always also read
/// these wrappers, but not an angle-bracket assertion (unlike
/// `unwrap_ts`, which follows Vue's `<script setup>` macro reading).
pub(super) fn wrapped<'e, 'a>(expression: &'e Expression<'a>) -> &'e Expression<'a> {
    match expression {
        Expression::ParenthesizedExpression(inner) => wrapped(&inner.expression),
        Expression::TSAsExpression(inner) => wrapped(&inner.expression),
        Expression::TSSatisfiesExpression(inner) => wrapped(&inner.expression),
        Expression::TSNonNullExpression(inner) => wrapped(&inner.expression),
        other => other,
    }
}

/// A `mixins` array inside wrappers, angle-bracket assertions included.
fn array<'e, 'a>(expression: &'e Expression<'a>) -> Option<&'e oxc_ast::ast::ArrayExpression<'a>> {
    match expression {
        Expression::ArrayExpression(array) => Some(array),
        Expression::ParenthesizedExpression(inner) => array(&inner.expression),
        Expression::TSAsExpression(inner) => array(&inner.expression),
        Expression::TSTypeAssertion(inner) => array(&inner.expression),
        Expression::TSSatisfiesExpression(inner) => array(&inner.expression),
        Expression::TSNonNullExpression(inner) => array(&inner.expression),
        _ => None,
    }
}
