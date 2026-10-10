//! Completed authored attribute heads, including opaque v-pre spellings.

use super::super::{CurrentDirective, Parser};
use vize_l1::markup::directive::frozen_attribute_name;
use vize_relief::{
    DirectiveNode, ElementNode, PropNode, SourceLocation,
    errors::{CompilerError, ErrorCode},
};

impl<'a> Parser<'a> {
    pub(in crate::parser) fn report_head_custody_error(
        &mut self,
        loc: &SourceLocation,
        extra: bool,
    ) {
        let message = if extra {
            "Completed directive head custody has extra entries while freezing v-pre."
        } else {
            "Completed directive head custody is missing while freezing v-pre."
        };
        self.errors.push(CompilerError::with_message(
            ErrorCode::MissingDirectiveName,
            message,
            Some(loc.clone()),
        ));
    }

    /// Join the completed head to its original borrowed semantic name. The
    /// lexer already supplied both boundaries; this never scans an argument.
    pub(in crate::parser) fn completed_directive_head(
        &self,
        name: &str,
        authored: Option<&'a str>,
        expected_start: usize,
        end_bound: usize,
    ) -> Option<(&'a str, &'a str, usize)> {
        let authored = authored?;
        let start = (authored.as_ptr() as usize).checked_sub(self.source.as_ptr() as usize)?;
        let end = start.checked_add(authored.len())?;
        let owned = self.source.get(start..end)?;
        if start != expected_start || end > end_bound || !std::ptr::eq(owned, authored) {
            return None;
        }
        let prefix_end = match authored.as_bytes().first()? {
            b':' | b'.' if name == "bind" => 1,
            b'@' if name == "on" => 1,
            b'#' if name == "slot" => 1,
            b'v' if authored.as_bytes().get(1) == Some(&b'-') => {
                let name_start = (name.as_ptr() as usize).checked_sub(authored.as_ptr() as usize)?;
                let name_end = name_start.checked_add(name.len())?;
                if name.is_empty()
                    || name_start != 2
                    || !std::ptr::eq(authored.get(name_start..name_end)?, name)
                {
                    return None;
                }
                name_end
            }
            _ => return None,
        };
        Some((authored.get(..prefix_end)?, authored, end))
    }

    pub(in crate::parser) fn completed_node_head(
        &self,
        dir: &DirectiveNode<'a>,
    ) -> Option<(&'a str, &'a str, usize)> {
        self.completed_directive_head(
            dir.name,
            dir.raw_name,
            dir.loc.span.start as usize,
            dir.loc.span.end as usize,
        )
    }

    pub(in crate::parser) fn retain_completed_directive_head(
        &mut self,
        dir: &CurrentDirective<'a>,
        loc: &SourceLocation,
    ) -> Option<&'a str> {
        let authored = self.source.get(dir.name_start..dir.name_end);
        if let Some((prefix, authored, _)) =
            self.completed_directive_head(dir.name, authored, dir.name_start, dir.name_end)
            && std::ptr::eq(prefix, dir.raw_name)
        {
            return Some(authored);
        }
        self.current_element = None;
        self.report_head_custody_error(loc, false);
        None
    }

    pub(in crate::parser) fn completed_raw_name(
        &mut self,
        dir: &CurrentDirective<'a>,
        loc: &SourceLocation,
    ) -> Option<&'a str> {
        if self.frozen_elements.is_some() {
            self.retain_completed_directive_head(dir, loc)
        } else {
            Some(dir.raw_name)
        }
    }

    pub(in crate::parser) fn frozen_directive_name(
        &mut self,
        dir: &DirectiveNode<'a>,
        before_pre: bool,
    ) -> Option<(&'a str, SourceLocation)> {
        if self.frozen_elements.is_none() {
            return Some((self.compatible_directive_name(dir), dir.loc.clone()));
        }
        let Some((_, authored, end)) = self.completed_node_head(dir) else {
            self.report_head_custody_error(&dir.loc, false);
            return None;
        };
        let name = if before_pre {
            authored
        } else {
            frozen_attribute_name(self.allocator, authored)
        };
        Some((name, self.create_loc(dir.loc.span.start as usize, end)))
    }

    /// Restore the original public prefix before a normal opening escapes.
    /// Own v-pre retains its complete heads until the existing conversion.
    pub(in crate::parser) fn finalize_directive_heads(
        &mut self,
        element: &mut ElementNode<'a>,
        ordinal: Option<usize>,
    ) -> bool {
        if self.frozen_elements.is_none() {
            return true;
        }
        if ordinal.is_some() {
            for prop in &element.props {
                if let PropNode::Directive(dir) = prop
                    && self.completed_node_head(dir).is_none()
                {
                    self.report_head_custody_error(&dir.loc, false);
                    return false;
                }
            }
            return true;
        }
        for prop in &mut element.props {
            if let PropNode::Directive(dir) = prop {
                if dir.name != "pre"
                    && let Some((prefix, _, _)) = self.completed_node_head(dir)
                {
                    dir.raw_name = Some(prefix);
                    continue;
                }
                self.report_head_custody_error(&dir.loc, false);
                return false;
            }
        }
        true
    }

    /// The original public parser freezes prefix plus its already parsed argument.
    pub(in crate::parser) fn compatible_directive_name(
        &mut self,
        dir: &DirectiveNode<'a>,
    ) -> &'a str {
        let prefix = dir.raw_name.unwrap_or(dir.name);
        let argument = dir.arg.as_ref().map(|arg| match arg {
            vize_relief::ExpressionNode::Simple(simple) => simple.content,
            vize_relief::ExpressionNode::Compound(compound) => compound.loc.span.slice(self.source),
        });
        match argument {
            Some(argument) => {
                let mut name = vize_l0::String::with_capacity(prefix.len() + argument.len());
                name.push_str(prefix);
                name.push_str(argument);
                self.interner.intern(&name)
            }
            None => prefix,
        }
    }

    /// Process the end of a full attribute or directive head.
    pub(in crate::parser) fn on_attrib_name_end_impl(&mut self, end: usize) {
        if self.frozen_elements.is_some()
            && self.in_v_pre
            && let Some(attr) = self.current_attr.as_ref()
        {
            let authored = self.get_source_retained(attr.name_start, end);
            let name = frozen_attribute_name(self.allocator, authored);
            if let Some(attr) = self.current_attr.as_mut() {
                attr.name = name;
            }
        }
        if let Some(ref mut attr) = self.current_attr {
            attr.name_end = end;
        }

        if let Some(ref mut dir) = self.current_dir {
            dir.name_end = end;
        }
    }
}

#[cfg(test)]
mod tests;
