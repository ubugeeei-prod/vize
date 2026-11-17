//! How a TSRX component of the Vue profile declares and reads its props,
//! after the pinned reference (`vue-jsx-vapor` macros, `defineComponent`).
//!
//! - An identifier first parameter declares every `props.name` it reads.
//!   Read as a value (`{...props}`), it is the props and the attributes
//!   together, as the reference's `useFullProps()`: the attributes alone when
//!   no key is read, else a live merged view (`_useFullProps`).
//! - A destructured first parameter declares its keys, read reactively as
//!   `__props.key` wherever the local name is read. A renamed key
//!   (`{ label: text }`) reads its key. A default becomes the prop's runtime
//!   options, typed from the default's syntax as the reference types it
//!   (`'a'` → `String`, `1` → `Number`, `true` → `Boolean`, `{}`/`[]` →
//!   `Object`/`Array` behind a factory, a function → `Function`, another name
//!   → `skipFactory`), and `x = y!` is `required`. A rest element is the
//!   component's attributes (`useAttrs()`), which then are not inherited.
//!
//! Refused (`Parameters`): more than one parameter, nested or computed
//! patterns, a default the reference drops (any other expression), a default
//! that reads a prop, a rest element on an expression-bodied arrow, and a
//! renamed local that is another prop's key. Refused (`PropsValue`): the props
//! identifier read as a value in an expression-bodied arrow, which has no
//! setup to bind it in, and a destructured prop used as a component tag
//! (#13280).

use alloc::vec::Vec;
use oxc_ast::ast::{BindingPattern, Expression, IdentifierReference, PropertyKey};
use oxc_ast_visit::Visit;
use oxc_span::GetSpan;
use vize_l0::Span;
use vize_l1::markup::tsrx::TsrxModule;

use super::EmitIssue;
use super::component::Component;
use super::names::Names;

/// How a component reads its props.
pub(super) enum Props<'a> {
    None,
    /// `({ a, b: c = 1, ...rest }) => …`.
    Destructured {
        param: Span,
        props: Vec<Prop<'a>>,
        /// The rest element's local, bound to the component's attributes.
        rest: Option<&'a str>,
    },
    /// `(props) => …`: every `props.name` read.
    Identifier {
        keys: Vec<&'a str>,
    },
    /// `(props) => …` reading `props` as a value: every `props.name` read is
    /// declared, and setup binds `name` to the props and the attributes.
    Full {
        param: Span,
        name: &'a str,
        keys: Vec<&'a str>,
    },
}

/// One destructured prop.
pub(super) struct Prop<'a> {
    pub key: &'a str,
    pub local: &'a str,
    pub options: Option<Options<'a>>,
}

/// A prop's runtime options.
pub(super) struct Options<'a> {
    pub required: bool,
    pub type_: Option<&'static str>,
    /// The `default` value's authored text.
    pub default: &'a str,
    /// The default is wrapped in a factory (`() => (value)`).
    pub factory: bool,
    pub skip_factory: bool,
}

impl<'a> Props<'a> {
    /// The declared keys with their options, in authored order.
    pub(super) fn declared(&self) -> Vec<(&'a str, Option<&Options<'a>>)> {
        match self {
            Self::None => Vec::new(),
            Self::Destructured { props, .. } => props
                .iter()
                .map(|prop| (prop.key, prop.options.as_ref()))
                .collect(),
            Self::Identifier { keys } | Self::Full { keys, .. } => {
                keys.iter().map(|key| (*key, None)).collect()
            }
        }
    }

    /// The prop a local name reads, if it is a destructured prop.
    pub(super) fn local(&self, name: &str) -> Option<&Prop<'a>> {
        match self {
            Self::Destructured { props, .. } => props.iter().find(|prop| prop.local == name),
            Self::None | Self::Identifier { .. } | Self::Full { .. } => None,
        }
    }

    pub(super) fn rest(&self) -> Option<&'a str> {
        match self {
            Self::Destructured { rest, .. } => *rest,
            Self::None | Self::Identifier { .. } | Self::Full { .. } => None,
        }
    }

    /// The props identifier read as a value, and whether it declares keys.
    pub(super) fn full(&self) -> Option<(&'a str, bool)> {
        match self {
            Self::Full { name, keys, .. } => Some((*name, !keys.is_empty())),
            Self::None | Self::Destructured { .. } | Self::Identifier { .. } => None,
        }
    }
}

pub(super) fn props<'a>(
    module: &'a TsrxModule<'a>,
    component: &Component<'a>,
) -> Result<Props<'a>, (EmitIssue, Span)> {
    let source = module.program().source_text;
    let params = component.params;
    let refuse = Err((EmitIssue::Parameters, span(params.span)));
    if params.rest.is_some() || params.items.len() > 1 {
        return refuse;
    }
    let Some(param) = params.items.first() else {
        return Ok(Props::None);
    };
    if param.initializer.is_some() || !param.decorators.is_empty() {
        return refuse;
    }
    let names = Names::of_component(module, component);
    let pattern = match &param.pattern {
        BindingPattern::BindingIdentifier(id) => {
            let name = id.name.as_str();
            let keys = names.member_keys(name);
            if let Some(use_) = names.value_reads(name).next() {
                if component.expression {
                    return Err((EmitIssue::PropsValue, use_));
                }
                return Ok(Props::Full {
                    param: span(param.span),
                    name,
                    keys,
                });
            }
            return Ok(Props::Identifier { keys });
        }
        BindingPattern::ObjectPattern(pattern) => pattern,
        _ => return refuse,
    };
    let rest = match &pattern.rest {
        None => None,
        Some(_) if component.expression => return refuse,
        Some(rest) => match &rest.argument {
            BindingPattern::BindingIdentifier(id) => Some(id.name.as_str()),
            _ => return refuse,
        },
    };
    let mut props = Vec::with_capacity(pattern.properties.len());
    for property in &pattern.properties {
        let PropertyKey::StaticIdentifier(key) = &property.key else {
            return refuse;
        };
        let (local, default) = match &property.value {
            BindingPattern::BindingIdentifier(local) => (local, None),
            BindingPattern::AssignmentPattern(assignment) => match &assignment.left {
                BindingPattern::BindingIdentifier(local) => (local, Some(&assignment.right)),
                _ => return refuse,
            },
            _ => return refuse,
        };
        if property.computed {
            return refuse;
        }
        props.push((key.name.as_str(), local.name.as_str(), default));
    }
    let locals: Vec<&str> = props.iter().map(|(_, local, _)| *local).collect();
    let mut declared = Vec::with_capacity(props.len());
    for (key, local, default) in props {
        let options = match default {
            Some(value) => match options(source, value, &locals) {
                Some(options) => Some(options),
                None => return refuse,
            },
            None => None,
        };
        if let Some(redeclared) = names.declarations(local).next() {
            return Err((EmitIssue::ShadowedProp, redeclared));
        }
        if let Some(tag) = names.tags(local).next() {
            return Err((EmitIssue::PropsValue, tag));
        }
        declared.push(Prop {
            key,
            local,
            options,
        });
    }
    // A renamed local that is another prop's key would read that key.
    for prop in &declared {
        if prop.local != prop.key && declared.iter().any(|other| other.key == prop.local) {
            return refuse;
        }
    }
    Ok(Props::Destructured {
        param: span(param.span),
        props: declared,
        rest,
    })
}

/// The runtime options of a default, or `None` for a default the reference
/// drops or that reads a prop.
fn options<'a>(source: &'a str, value: &'a Expression<'a>, locals: &[&str]) -> Option<Options<'a>> {
    let required = matches!(value, Expression::TSNonNullExpression(_));
    let mut value = value;
    loop {
        value = match value {
            Expression::TSNonNullExpression(inner) => &inner.expression,
            Expression::TSAsExpression(inner) => &inner.expression,
            _ => break,
        };
    }
    let text = source.get(value.span().start as usize..value.span().end as usize)?;
    let mut options = Options {
        required,
        type_: None,
        default: text,
        factory: false,
        skip_factory: false,
    };
    match value {
        Expression::StringLiteral(_) => options.type_ = Some("String"),
        Expression::BooleanLiteral(_) => options.type_ = Some("Boolean"),
        Expression::NumericLiteral(_) => options.type_ = Some("Number"),
        Expression::ObjectExpression(_) => {
            options.type_ = Some("Object");
            options.factory = true;
        }
        Expression::ArrayExpression(_) => {
            options.type_ = Some("Array");
            options.factory = true;
        }
        Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_) => {
            options.type_ = Some("Function");
        }
        Expression::Identifier(id) if id.name == "undefined" => {}
        Expression::Identifier(_) => options.skip_factory = true,
        Expression::NullLiteral(_) => {}
        _ => return None,
    }
    // The options are evaluated where the component is defined, outside it.
    let mut reads = Reads {
        locals,
        found: false,
    };
    reads.visit_expression(value);
    (!reads.found).then_some(options)
}

struct Reads<'l> {
    locals: &'l [&'l str],
    found: bool,
}

impl<'a> Visit<'a> for Reads<'_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        self.found |= self.locals.contains(&it.name.as_str());
    }
}

fn span(span: oxc_span::Span) -> Span {
    Span::new(span.start, span.end)
}
