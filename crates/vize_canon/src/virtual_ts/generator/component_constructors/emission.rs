//! Borrowed type references and the two public generic component signatures.
//!
//! These views format directly into the enclosing emission buffer. They do
//! not own intermediate TypeScript or interpret authored type expressions.

use core::fmt;

use crate::virtual_ts::generated_names as names;
use vize_carton::{String, cstr};

#[cfg(test)]
mod tests;

pub(super) struct AliasReference<'a> {
    name: &'a str,
    arguments: Option<&'a str>,
}

impl<'a> AliasReference<'a> {
    pub(super) const fn new(name: &'a str, generic: bool, arguments: &'a str) -> Self {
        Self {
            name,
            arguments: if generic { Some(arguments) } else { None },
        }
    }
}

impl fmt::Display for AliasReference<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.name)?;
        if let Some(arguments) = self.arguments {
            formatter.write_str("<")?;
            formatter.write_str(arguments)?;
            formatter.write_str(">")?;
        }
        Ok(())
    }
}

impl vize_l0::ts_emit::EmitLen for AliasReference<'_> {
    fn hole_len(&self) -> usize {
        let mut len = self.name.len();
        if let Some(arguments) = self.arguments {
            len += arguments.len() + 2;
        }
        len
    }
}

impl<B: vize_l0::ts_emit::TsBuf> vize_l0::ts_emit::EmitHole<B> for AliasReference<'_> {
    fn emit_hole(&self, buffer: &mut B) {
        buffer.push_ts_str(self.name);
        if let Some(arguments) = self.arguments {
            buffer.push_ts_str("<");
            buffer.push_ts_str(arguments);
            buffer.push_ts_str(">");
        }
    }
}

/// One generic setup context, streamed into the final module rather than
/// assembled from owned exposure, listener and emit type fragments.
pub(super) struct GenericContext<'a> {
    pub(super) parameters: &'a str,
    pub(super) arguments: &'a str,
    pub(super) props_type: &'a str,
    pub(super) emits: AliasReference<'a>,
    pub(super) slots: AliasReference<'a>,
    pub(super) exposed: AliasReference<'a>,
    pub(super) emit_props: bool,
    pub(super) emit_props_type: &'a str,
    pub(super) jsx_slots: bool,
    pub(super) root_element: bool,
    pub(super) template_refs: bool,
}

impl fmt::Display for GenericContext<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let Self {
            parameters,
            arguments,
            props_type,
            emits,
            slots,
            exposed,
            emit_props_type,
            ..
        } = self;
        let context = names::declared("$GenericContext");
        write!(
            formatter,
            "type {context}{parameters} = {{\n  props: {props_type}{arguments}"
        )?;
        if self.emit_props {
            write!(formatter, " & {emit_props_type}<{emits}>")?;
        }
        if self.jsx_slots {
            write!(
                formatter,
                " & {}<{slots}>",
                names::declared("$JsxSlotProps")
            )?;
        }
        formatter.write_str(" & import('vue').VNodeProps & import('vue').AllowedComponentProps & import('vue').ComponentCustomProps;\n  attrs: any;\n  emit: ")?;
        if self.emit_props {
            write!(
                formatter,
                "{}<{emits}>",
                names::helper("$PublicEmit", &[""])
            )?;
        } else {
            formatter.write_str("{}")?;
        }
        write!(
            formatter,
            ";\n  slots: {slots};\n  expose: (exposed: import('vue').ShallowUnwrapRef<{exposed}>"
        )?;
        if self.root_element {
            write!(
                formatter,
                " & {{ $el: {}['__vize_root_el'] }}",
                names::setup_return(arguments)
            )?;
        }
        if self.template_refs {
            write!(
                formatter,
                " & {{ $refs: {}['__vize_refs'] }}",
                names::setup_return(arguments)
            )?;
        }
        formatter.write_str(") => void;\n};\n")
    }
}

#[derive(Clone, Copy)]
pub(super) enum SignatureRole {
    Component,
    TemplateRefResolver,
}

/// The component checks its actual props. A ref probe repeats those props to
/// instantiate the return context while leaving extra attributes unchecked.
pub(super) struct GenericSignature<'a> {
    pub(super) role: SignatureRole,
    pub(super) parameters: &'a str,
    pub(super) arguments: &'a str,
    pub(super) vue_component_context: bool,
}

impl fmt::Display for GenericSignature<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (name, props_tail) = match self.role {
            SignatureRole::Component => ("$GenericComponent", String::default()),
            SignatureRole::TemplateRefResolver => (
                "$GenericRefResolver",
                cstr!(" & {}<string, unknown>", names::lib("Record")),
            ),
        };
        let Self {
            parameters,
            arguments,
            vue_component_context,
            ..
        } = self;
        let context = names::declared("$GenericContext");
        let pick = names::lib("Pick");
        if *vue_component_context {
            // Vue's generic SFC function overlaps `Component` when emitted
            // listeners are present. Keep its loose compatibility signature
            // first and the precise public signature last: conditional helpers
            // such as `ComponentEmit` and `ComponentSlots` infer from the last
            // overload, while the first one preserves the cast overlap
            // required by Vue's `Component` union (#11966).
            return writeln!(
                formatter,
                "type {name} = ({parameters}(props: {context}{arguments}['props']{props_tail}, ctx?: {{ attrs: any; emit: (...args: any[]) => any; slots: any }}, expose?: {context}{arguments}['expose']) => import('vue').VNode & {{ __ctx?: {context}{arguments} }}) & ({parameters}(props: {context}{arguments}['props']{props_tail}, ctx?: {pick}<{context}{arguments}, 'attrs' | 'emit' | 'slots'>, expose?: {context}{arguments}['expose']) => import('vue').VNode & {{ __ctx?: {context}{arguments} }});"
            );
        }
        writeln!(
            formatter,
            "type {name} = {parameters}(props: {context}{arguments}['props']{props_tail}, ctx?: {pick}<{context}{arguments}, 'attrs' | 'emit' | 'slots'>, expose?: {context}{arguments}['expose']) => import('vue').VNode & {{ __ctx?: {context}{arguments} }};"
        )
    }
}
