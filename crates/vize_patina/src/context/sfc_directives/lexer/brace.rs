//! Lightweight delimiter roles used to distinguish regex from division.

use super::token::identifier_opens_control_paren;

#[derive(Clone, Copy, PartialEq, Eq)]
enum ParenContext {
    Expression,
    Control,
    Function(BraceContext),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BraceContext {
    Expression,
    FunctionExpression,
    Statement,
}

#[derive(Default)]
pub(super) struct DelimiterState {
    parens: Vec<ParenContext>,
    braces: Vec<BraceContext>,
    next_paren_is_control: bool,
    next_paren_is_function: Option<BraceContext>,
    function_type_depth: u32,
    next_brace: Option<BraceContext>,
    function_expression_required: bool,
}

impl DelimiterState {
    pub(super) fn before_non_identifier(&mut self, byte: u8, starts_comment: bool) {
        if byte.is_ascii_whitespace() || starts_comment {
            return;
        }
        if byte != b'(' {
            self.next_paren_is_control = false;
        }
        if self.next_paren_is_function.is_some() {
            match byte {
                b'(' | b'*' => {}
                b'<' => self.function_type_depth += 1,
                b'>' if self.function_type_depth > 0 => self.function_type_depth -= 1,
                _ if self.function_type_depth > 0 => {}
                _ => self.next_paren_is_function = None,
            }
        }
        if byte != b'{' {
            self.next_brace = None;
        }
    }

    pub(super) fn observe_identifier(&mut self, identifier: &[u8], after_dot: bool) {
        let keeps_control = self.next_paren_is_control && identifier == b"await" && !after_dot;
        self.next_paren_is_control =
            keeps_control || (!after_dot && identifier_opens_control_paren(identifier));
        if after_dot {
            self.next_paren_is_function = None;
            self.function_type_depth = 0;
        } else if identifier == b"function" {
            self.next_paren_is_function = Some(if self.function_expression_required {
                BraceContext::FunctionExpression
            } else {
                BraceContext::Statement
            });
            self.function_type_depth = 0;
        }
        self.next_brace = (!after_dot
            && matches!(identifier, b"catch" | b"do" | b"else" | b"finally" | b"try"))
        .then_some(BraceContext::Statement);
        if self.function_expression_required
            || !matches!(identifier, b"async" | b"declare" | b"default" | b"export")
        {
            self.function_expression_required = true;
        }
    }

    pub(super) fn open_paren(&mut self) {
        let context = if std::mem::take(&mut self.next_paren_is_control) {
            ParenContext::Control
        } else if let Some(body) = self.next_paren_is_function.take() {
            self.function_type_depth = 0;
            ParenContext::Function(body)
        } else {
            ParenContext::Expression
        };
        self.next_brace = None;
        self.function_expression_required = true;
        self.parens.push(context);
    }

    pub(super) fn close_paren(&mut self) -> bool {
        let context = self.parens.pop();
        self.next_brace = match context {
            Some(ParenContext::Control) => Some(BraceContext::Statement),
            Some(ParenContext::Function(body)) => Some(body),
            _ => None,
        };
        self.function_expression_required = true;
        matches!(context, Some(ParenContext::Control))
    }

    pub(super) fn open_brace(&mut self) {
        let context = self.next_brace.take().unwrap_or(BraceContext::Expression);
        self.function_expression_required = matches!(context, BraceContext::Expression);
        self.braces.push(context);
    }

    pub(super) fn close_brace(&mut self) -> bool {
        let closes_statement = matches!(self.braces.pop(), Some(BraceContext::Statement));
        self.function_expression_required = !closes_statement;
        closes_statement
    }

    pub(super) fn observe_arrow(&mut self) {
        self.next_brace = Some(BraceContext::FunctionExpression);
        self.function_expression_required = true;
    }

    pub(super) fn observe_operator(&mut self, operator: u8) {
        self.function_expression_required = operator != b';';
    }

    pub(super) fn finish_line(&mut self, can_start_expression: bool) {
        if !can_start_expression {
            self.function_expression_required = false;
        }
    }
}
