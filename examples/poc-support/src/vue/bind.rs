//! How the template sees the script's bindings: `ref` bindings read and write
//! `.value`, props read `__props`, everything else is read as is.

use oxc_ast::ast::{BindingIdentifier, IdentifierReference, ObjectProperty};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_l0::{Allocator, Span, String, cstr};

use crate::Error;

/// How the component exposes one script binding to the template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Binding {
    /// A `ref`: the template reads and writes `.value`.
    Ref,
    /// A constant or function, read as is.
    Plain,
    /// A declared prop, read from `__props`.
    Prop,
}

/// The script's bindings, in declaration order.
#[derive(Debug, Default)]
pub struct Bindings {
    names: Vec<(String, Binding)>,
}

impl Bindings {
    pub fn declare(&mut self, name: &str, binding: Binding) {
        self.names.push((String::from(name), binding));
    }

    #[must_use]
    pub fn get(&self, name: &str) -> Option<Binding> {
        self.names
            .iter()
            .find(|(declared, _)| declared == name)
            .map(|(_, binding)| *binding)
    }

    pub fn props(&self) -> impl Iterator<Item = &str> {
        self.names
            .iter()
            .filter(|(_, binding)| *binding == Binding::Prop)
            .map(|(name, _)| name.as_str())
    }
}

/// Bind script names in one JavaScript expression.
pub(super) fn bind(
    js: &str,
    span: Span,
    bindings: &Bindings,
    locals: &[&str],
) -> Result<String, Error> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, js, SourceType::mjs())
        .parse_expression()
        .map_err(|_| Error::new(cstr!("not an expression: {js}"), span))?;
    let mut binder = Binder {
        bindings,
        locals,
        edits: Vec::new(),
        refused: false,
    };
    binder.visit_expression(&parsed);
    if binder.refused {
        return Err(Error::new(
            "a template expression shadows a script binding",
            span,
        ));
    }
    binder.edits.sort_by_key(|(start, _)| *start);
    let mut out = String::default();
    let mut cursor = 0;
    for (start, (end, replacement)) in binder.edits.iter().map(|(start, rest)| (*start, rest)) {
        out.push_str(js.get(cursor..start as usize).unwrap_or_default());
        out.push_str(replacement);
        cursor = *end as usize;
    }
    out.push_str(js.get(cursor..).unwrap_or_default());
    Ok(out)
}

struct Binder<'b> {
    bindings: &'b Bindings,
    locals: &'b [&'b str],
    edits: Vec<(u32, (u32, String))>,
    refused: bool,
}

impl<'a> Visit<'a> for Binder<'_> {
    fn visit_identifier_reference(&mut self, it: &IdentifierReference<'a>) {
        let name = it.name.as_str();
        if self.locals.contains(&name) {
            return;
        }
        let replacement = match self.bindings.get(name) {
            Some(Binding::Ref) => cstr!("{name}.value"),
            Some(Binding::Prop) => cstr!("__props.{name}"),
            Some(Binding::Plain) | None => return,
        };
        self.edits.push((it.span.start, (it.span.end, replacement)));
    }

    fn visit_binding_identifier(&mut self, it: &BindingIdentifier<'a>) {
        self.refused |= self.bindings.get(it.name.as_str()).is_some();
    }

    fn visit_object_property(&mut self, it: &ObjectProperty<'a>) {
        self.refused |= it.shorthand;
        walk::walk_object_property(self, it);
    }
}
