use super::NativeSelectedSfcIssueKind as Kind;
use vize_l0::Span;
use vize_l1::{SurfaceChild, markup::NativeInterpolationFailure};
use vize_l2::lang::js::{NativeInterpolationInput, NativeTemplateIssue, NativeTemplateOwner};

pub(super) struct Failure<'a> {
    pub span: Option<Span>,
    pub kind: Kind,
    pub interpolation: Option<NativeInterpolationFailure<'a>>,
}
impl From<NativeTemplateIssue> for Failure<'_> {
    fn from(issue: NativeTemplateIssue) -> Self {
        Self {
            span: Some(issue.span),
            kind: Kind::Template(issue),
            interpolation: None,
        }
    }
}

pub(super) fn construct<'a>(owner: &mut NativeTemplateOwner<'a>) -> Result<(), Failure<'a>> {
    let mut walk = owner.begin()?;
    let selected = walk.selected();
    for child in selected.children() {
        match child.surface() {
            SurfaceChild::Text(token) => {
                let span = selected.component().block().span_of(token.text);
                let text = selected
                    .prepare_condensed_root_text(child)
                    .map_err(|error| Failure {
                        span,
                        kind: Kind::RootText(error),
                        interpolation: None,
                    })?;
                walk.root_text(&text)?;
            }
            SurfaceChild::Interpolation(interpolation) => {
                let span = selected
                    .component()
                    .block()
                    .span_of(interpolation.content.text);
                let operand = selected
                    .observe_interpolation_expression(child.reborrow())
                    .map_err(|failure| Failure {
                        span,
                        kind: Kind::Interpolation(failure.kind()),
                        interpolation: Some(failure),
                    })?;
                walk.root_interpolation(child, NativeInterpolationInput::from_operand(operand))?;
            }
            _ => {
                walk.child(child)?;
            }
        }
    }
    walk.complete()?;
    Ok(())
}
