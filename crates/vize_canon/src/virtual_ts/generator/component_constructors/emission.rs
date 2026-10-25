//! Borrowed type references and the two public generic component signatures.
//!
//! These views format directly into the enclosing emission buffer. They do
//! not own intermediate TypeScript or interpret authored type expressions.

use core::fmt;

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
            ..
        } = self;
        write!(
            formatter,
            "type __VizeGenericContext{parameters} = {{\n  props: {props_type}{arguments}"
        )?;
        if self.emit_props {
            write!(formatter, " & __EmitProps<{emits}>")?;
        }
        if self.jsx_slots {
            write!(formatter, " & __VizeJsxSlotProps<{slots}>")?;
        }
        formatter.write_str(" & import('vue').VNodeProps & import('vue').AllowedComponentProps & import('vue').ComponentCustomProps;\n  attrs: any;\n  emit: ")?;
        if self.emit_props {
            write!(formatter, "__VizePublicEmit<{emits}>")?;
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
                " & {{ $el: Awaited<ReturnType<typeof __setup{arguments}>>['__vize_root_el'] }}"
            )?;
        }
        if self.template_refs {
            write!(
                formatter,
                " & {{ $refs: Awaited<ReturnType<typeof __setup{arguments}>>['__vize_refs'] }}"
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
}

impl fmt::Display for GenericSignature<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (name, props_tail) = match self.role {
            SignatureRole::Component => ("__VizeGenericComponent", ""),
            SignatureRole::TemplateRefResolver => {
                ("__VizeGenericRefResolver", " & Record<string, unknown>")
            }
        };
        let Self {
            parameters,
            arguments,
            ..
        } = self;
        write!(
            formatter,
            "type {name} = {parameters}(props: __VizeGenericContext{arguments}['props']{props_tail}, ctx?: Pick<__VizeGenericContext{arguments}, 'attrs' | 'emit' | 'slots'>, expose?: __VizeGenericContext{arguments}['expose']) => import('vue').VNode & {{ __ctx?: __VizeGenericContext{arguments} }};\n"
        )
    }
}
