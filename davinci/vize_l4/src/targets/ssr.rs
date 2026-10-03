//! Native server rendering from sole-owner L3 SSR decisions.
//!
//! L3 decides HTML eligibility, root fallthrough, fragments and void content
//! in the canonical walk. L4 appends JavaScript and records original spans.
//! An unsupported whole view returns no writer or partial render function.

use vize_l0::{Span, id::NodeId};
use vize_l3::decision::ssr::{NativeSsrFileAnalysis, SsrFacts, SsrPart, SsrUnsupported};
use vize_l3::decision::{NativeAnalysis, policy::TargetPolicy};

use crate::runtime::{Runtime, vocabulary};
use crate::write::{LinkSink, Writer};

mod write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SsrError {
    pub node: Option<NodeId>,
    pub span: Span,
    pub kind: SsrErrorKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SsrErrorKind {
    WrongPolicy,
    MissingAnalysis,
    MissingRuntimeHelper,
    Unsupported(SsrUnsupported),
    OutputTooLarge,
}

/// Emit a complete prepared SSR function from its exact sealed native owner.
/// No caller source, region, decision table or expression facts are paired here.
pub fn emit<L: LinkSink>(analysis: &NativeAnalysis<'_, '_>) -> Result<Writer<L>, SsrError> {
    if analysis.policy() != TargetPolicy::Ssr {
        return Err(SsrError {
            node: None,
            span: Span::new(0, 0),
            kind: SsrErrorKind::WrongPolicy,
        });
    }
    encode(analysis.artifact().source(), analysis.ssr())
}

/// Consume only a complete SSR file view and retain its diagnostic owner.
/// Original native SFC custody and runtime expression exposure remain separate.
pub fn emit_file<L: LinkSink>(
    analysis: &NativeSsrFileAnalysis<'_, '_>,
) -> Result<Writer<L>, SsrError> {
    encode(analysis.artifact().source(), analysis.ssr())
}

fn encode<L: LinkSink>(
    source: &str,
    facts: Option<&SsrFacts<'_, '_>>,
) -> Result<Writer<L>, SsrError> {
    let error = |kind| SsrError {
        node: None,
        span: Span::new(0, source.len() as u32),
        kind,
    };
    let facts = facts.ok_or_else(|| error(SsrErrorKind::MissingAnalysis))?;
    if let Some(rejected) = facts.unsupported().first() {
        return Err(SsrError {
            node: Some(rejected.node),
            span: rejected.span,
            kind: SsrErrorKind::Unsupported(rejected.reason),
        });
    }
    let runtime = vocabulary(Runtime::VueServerRenderer);
    let attrs = runtime
        .helper("ssrRenderAttrs")
        .ok_or_else(|| error(SsrErrorKind::MissingRuntimeHelper))?;
    let merge = runtime
        .helper("mergeProps")
        .ok_or_else(|| error(SsrErrorKind::MissingRuntimeHelper))?;
    let mut writer = Writer::<L>::with_capacity(source.len());
    writer.push("function ssrRender(_ctx, _push, _parent, _attrs) {");
    writer.indent();
    writer.newline();
    if !facts.parts().is_empty() {
        writer.push("_push(`");
        if facts.fragment() {
            writer.push("<!--[-->");
        }
        for part in facts.parts() {
            match *part {
                SsrPart::Open { node, element, .. } => {
                    writer.push("<");
                    write::template(&mut writer, element.tag, element.span, false);
                    if facts.inherit_attrs() == Some(node) {
                        writer.use_helper(attrs);
                        writer.push("${_ssrRenderAttrs(");
                        if element.attributes.is_empty() {
                            writer.push("_attrs");
                        } else {
                            writer.use_helper(merge);
                            writer.push("_mergeProps({ ");
                            for (index, attribute) in element.attributes.iter().enumerate() {
                                if index > 0 {
                                    writer.push(", ");
                                }
                                write::property(&mut writer, attribute.name, attribute.span);
                                writer.push(": ");
                                write::quoted(
                                    &mut writer,
                                    attribute.value.unwrap_or_default(),
                                    attribute.span,
                                );
                            }
                            writer.push(" }, _attrs)");
                        }
                        if element.tag.contains('-') {
                            writer.push(", ");
                            write::quoted(&mut writer, element.tag, element.span);
                        }
                        writer.push(")}");
                    } else {
                        for attribute in &element.attributes {
                            writer.push(" ");
                            write::template(&mut writer, attribute.name, attribute.span, false);
                            if let Some(value) = attribute.value {
                                writer.push("=\"");
                                write::template(&mut writer, value, attribute.span, true);
                                writer.push("\"");
                            }
                        }
                    }
                    writer.push(">");
                }
                SsrPart::Close { element, .. } => {
                    writer.push("</");
                    write::template(&mut writer, element.tag, element.span, false);
                    writer.push(">");
                }
                SsrPart::Text { text, .. } => {
                    write::template(&mut writer, text.content, text.span, true)
                }
                SsrPart::Comment { comment, .. } => {
                    writer.push("<!--");
                    write::template(&mut writer, comment.content, comment.span, false);
                    writer.push("-->");
                }
            }
        }
        if facts.fragment() {
            writer.push("<!--]-->");
        }
        writer.push("`)");
    }
    writer.deindent();
    writer.newline();
    writer.push("}");
    if u32::try_from(writer.len()).is_err() {
        return Err(error(SsrErrorKind::OutputTooLarge));
    }
    Ok(writer)
}
