//! Remove only the formatter's unary wrapper, using the existing parsed AST.
use oxc_ast::ast::{Expression, Program, Statement};

pub(crate) struct FormattedExpression {
    pub code: vize_l0::String,
    pub retained_bare_sequence: bool,
}

pub(super) struct Argument<'a> {
    pub text: &'a str,
    pub retained_bare_sequence: bool,
}

#[inline]
pub(super) fn unwrap_argument<'a>(
    printed: &'a str,
    program: &Program<'_>,
    original: &'a str,
) -> Option<Argument<'a>> {
    let [Statement::ExpressionStatement(statement)] = program.body.as_slice() else {
        return None;
    };
    let Expression::UnaryExpression(unary) = &statement.expression else {
        return None;
    };
    if let Expression::SequenceExpression(sequence) = &unary.argument {
        return sequence_argument(printed, original, sequence.span.start, program);
    }
    // Calls/members need no normal unary wrapper. Edge parentheses belong to
    // the authored receiver/call or comment-preserving layout and must remain.
    if matches!(
        &unary.argument,
        Expression::CallExpression(_)
            | Expression::StaticMemberExpression(_)
            | Expression::ComputedMemberExpression(_)
            | Expression::PrivateFieldExpression(_)
            | Expression::ChainExpression(_)
            | Expression::TSNonNullExpression(_)
            | Expression::TSInstantiationExpression(_)
    ) {
        return Some(Argument {
            text: printed,
            retained_bare_sequence: false,
        });
    }
    Some(Argument {
        text: printed
            .strip_prefix('(')
            .and_then(|rest| rest.strip_suffix(')'))
            .unwrap_or(printed),
        retained_bare_sequence: false,
    })
}

#[cold]
#[inline(never)]
fn sequence_argument<'a>(
    printed: &'a str,
    original: &'a str,
    argument_start: u32,
    program: &Program<'_>,
) -> Option<Argument<'a>> {
    // The retained Sequence span excludes its enclosing parentheses. Its
    // prefix contains only outer grouping, layout and parser-owned comments.
    let wrapper_len = "void (".len();
    let prefix = original.get(..(argument_start as usize).checked_sub(wrapper_len)?)?;
    let grouped = prefix.char_indices().any(|(offset, token)| {
        let absolute = (offset + wrapper_len) as u32;
        token == '('
            && !program
                .comments
                .iter()
                .any(|comment| comment.span.start <= absolute && absolute < comment.span.end)
    });
    // Vue emits an unparenthesized interpolation as a consumer argument list.
    // Preserve its original nested argument groups, not just the JS value of
    // the entire Sequence, while an authored enclosing group stays one value.
    Some(Argument {
        text: if grouped { printed } else { original },
        retained_bare_sequence: !grouped,
    })
}
