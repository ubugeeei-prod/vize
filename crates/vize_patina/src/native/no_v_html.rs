//! Opaque full directives and static binding sinks from one genuine header.

use vize_l0::{
    SmallVec, Span,
    diag::{Advisory, Diagnostic, DiagnosticPart, MessageLookup, PartKind, Stage},
};
use vize_l1::markup::NativeElement;

use super::{NO_V_HTML_RULE, NativeLintFinding, NativeLintRefusal, header, header_rules};

/// Header syntax and unproven inherited verbatim context remain distinct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NativeVHtmlLintError {
    Header(NativeLintRefusal),
    InheritedLiteralContext { span: Span },
}

impl From<NativeLintRefusal> for NativeVHtmlLintError {
    fn from(error: NativeLintRefusal) -> Self {
        Self::Header(error)
    }
}

pub(super) fn check(
    element: &NativeElement<'_, '_>,
    messages: &impl MessageLookup,
) -> Result<Vec<NativeLintFinding>, NativeVHtmlLintError> {
    let mut matches: SmallVec<[Span; 4]> = SmallVec::new();
    let mut own_pre = false;
    let (receipt, opening) =
        header_rules::inspect_attributes(element, |_, binding| match binding {
            header::Binding::Other {
                full_directive_name: Some("html"),
                range,
            } => matches.push(range),
            header::Binding::Bind {
                name: "innerHTML" | "outerHTML",
                argument_range,
                ..
            } => matches.push(argument_range),
            header::Binding::Static { name: "v-pre", .. } => own_pre = true,
            _ => {}
        })?;
    if receipt.header_is_literal() && !own_pre {
        return Err(NativeVHtmlLintError::InheritedLiteralContext { span: opening });
    }
    Ok(matches
        .into_iter()
        .map(|range| NativeLintFinding {
            rule_name: NO_V_HTML_RULE,
            diagnostic: Diagnostic::new(
                Advisory::Warning,
                Stage::Surface,
                range,
                messages.lookup("vue/no-v-html.message").as_ref(),
            )
            .with_part(DiagnosticPart::new(
                PartKind::Help,
                range,
                messages.lookup("vue/no-v-html.help").as_ref(),
            )),
        })
        .collect())
}
