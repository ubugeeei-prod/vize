//! Static authored style attributes over the genuine selected header provider.

use vize_l0::{
    SmallVec, Span,
    diag::{Advisory, Diagnostic, DiagnosticPart, MessageLookup, PartKind, Stage},
};
use vize_l1::markup::NativeElement;

use super::{NO_INLINE_STYLE_RULE, NativeLintFinding, NativeLintRefusal, header, header_rules};

pub(super) fn check(
    element: &NativeElement<'_, '_>,
    messages: &impl MessageLookup,
) -> Result<Vec<NativeLintFinding>, NativeLintRefusal> {
    let mut matches: SmallVec<[Span; 4]> = SmallVec::new();
    header_rules::inspect(element, |binding| {
        if let header::Binding::Static {
            name: "style",
            range,
            ..
        } = binding
        {
            matches.push(range);
        }
    })?;
    Ok(matches
        .into_iter()
        .map(|range| NativeLintFinding {
            rule_name: NO_INLINE_STYLE_RULE,
            diagnostic: Diagnostic::new(
                Advisory::Warning,
                Stage::Surface,
                range,
                messages.lookup("vue/no-inline-style.message").as_ref(),
            )
            .with_part(DiagnosticPart::new(
                PartKind::Help,
                range,
                messages.lookup("vue/no-inline-style.help").as_ref(),
            )),
        })
        .collect())
}
