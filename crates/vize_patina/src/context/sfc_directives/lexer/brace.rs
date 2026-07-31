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
    ExpressionBody,
    Statement,
    TypeExpression,
}

#[derive(Default)]
struct ReturnTypeState {
    brace_depth: u32,
    can_end: bool,
}

struct ClassHeaderState {
    body: BraceContext,
    paren_depth: usize,
    angle_depth: u32,
}

#[derive(Default)]
pub(super) struct DelimiterState {
    parens: Vec<ParenContext>,
    braces: Vec<BraceContext>,
    next_paren_is_control: bool,
    next_paren_is_function: Option<BraceContext>,
    function_type_depth: u32,
    next_brace: Option<BraceContext>,
    pending_function_body: Option<BraceContext>,
    return_type: Option<ReturnTypeState>,
    class_headers: Vec<ClassHeaderState>,
    expression_required: bool,
    brace_expression_required: bool,
    possible_label: bool,
}

impl DelimiterState {
    pub(super) fn before_non_identifier(&mut self, byte: u8, starts_comment: bool) {
        if byte.is_ascii_whitespace() || starts_comment {
            return;
        }
        if byte != b':' {
            self.possible_label = false;
        }
        if byte != b'(' {
            self.next_paren_is_control = false;
        }
        if byte == b':' && self.pending_function_body.is_some() && self.return_type.is_none() {
            self.return_type = Some(ReturnTypeState::default());
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
        if let Some(class_header) = self
            .class_headers
            .last_mut()
            .filter(|class_header| class_header.paren_depth == self.parens.len())
        {
            match byte {
                b'<' => class_header.angle_depth += 1,
                b'>' if class_header.angle_depth > 0 => class_header.angle_depth -= 1,
                _ => {}
            }
        }
        if byte != b'{' {
            self.next_brace = None;
        }
    }

    pub(super) fn observe_identifier(&mut self, identifier: &[u8], after_dot: bool) {
        let at_statement_start = !self.expression_required && !self.brace_expression_required;
        self.possible_label =
            !after_dot && at_statement_start && identifier_can_be_label(identifier);
        if let Some(return_type) = self.return_type.as_mut() {
            return_type.can_end = true;
        }
        let keeps_control = self.next_paren_is_control && identifier == b"await" && !after_dot;
        self.next_paren_is_control =
            keeps_control || (!after_dot && identifier_opens_control_paren(identifier));
        if after_dot {
            if self.function_type_depth == 0 {
                self.next_paren_is_function = None;
            }
        } else if identifier == b"function" {
            self.next_paren_is_function = Some(if self.expression_required {
                BraceContext::ExpressionBody
            } else {
                BraceContext::Statement
            });
            self.function_type_depth = 0;
        } else if identifier == b"class" {
            let body = if self.expression_required {
                BraceContext::ExpressionBody
            } else {
                BraceContext::Statement
            };
            self.class_headers.push(ClassHeaderState {
                body,
                paren_depth: self.parens.len(),
                angle_depth: 0,
            });
        }
        self.next_brace = (!after_dot
            && matches!(identifier, b"catch" | b"do" | b"else" | b"finally" | b"try"))
        .then_some(BraceContext::Statement);
        if self.expression_required
            || !matches!(identifier, b"async" | b"declare" | b"default" | b"export")
        {
            self.expression_required = true;
        }
        if self.brace_expression_required || !matches!(identifier, b"declare" | b"export") {
            self.brace_expression_required = true;
        }
    }

    pub(super) fn open_paren(&mut self) {
        let context = if std::mem::take(&mut self.next_paren_is_control) {
            ParenContext::Control
        } else if self.function_type_depth == 0 && self.next_paren_is_function.is_some() {
            let body = self.next_paren_is_function.take().unwrap();
            self.function_type_depth = 0;
            ParenContext::Function(body)
        } else {
            ParenContext::Expression
        };
        if let Some(return_type) = self.return_type.as_mut() {
            return_type.can_end = false;
        }
        self.next_brace = None;
        self.expression_required = true;
        self.brace_expression_required = true;
        self.parens.push(context);
    }

    pub(super) fn close_paren(&mut self) -> bool {
        let context = self.parens.pop();
        self.next_brace = None;
        match context {
            Some(ParenContext::Control) => self.next_brace = Some(BraceContext::Statement),
            Some(ParenContext::Function(body)) => self.pending_function_body = Some(body),
            Some(ParenContext::Expression) => {
                if let Some(return_type) = self.return_type.as_mut() {
                    return_type.can_end = true;
                }
            }
            None => {}
        }
        self.expression_required = true;
        self.brace_expression_required = true;
        matches!(context, Some(ParenContext::Control))
    }

    pub(super) fn open_brace(&mut self, expression_complete: bool) {
        let function_body_starts = self.return_type.as_ref().is_none_or(|return_type| {
            return_type.brace_depth == 0 && (return_type.can_end || expression_complete)
        });
        let context = if self.pending_function_body.is_some() && function_body_starts {
            self.return_type = None;
            self.pending_function_body.take().unwrap()
        } else if self.pending_function_body.is_some() {
            self.return_type.as_mut().unwrap().brace_depth += 1;
            BraceContext::TypeExpression
        } else if self.class_headers.last().is_some_and(|class_header| {
            class_header.angle_depth == 0 && class_header.paren_depth == self.parens.len()
        }) {
            self.class_headers.pop().unwrap().body
        } else if let Some(context) = self.next_brace.take() {
            context
        } else if self.brace_expression_required {
            BraceContext::Expression
        } else {
            BraceContext::Statement
        };
        self.expression_required = matches!(context, BraceContext::Expression);
        self.brace_expression_required = self.expression_required;
        self.braces.push(context);
    }

    pub(super) fn close_brace(&mut self) -> bool {
        let context = self.braces.pop();
        let closes_statement = matches!(context, Some(BraceContext::Statement));
        if matches!(context, Some(BraceContext::TypeExpression))
            && let Some(return_type) = self.return_type.as_mut()
        {
            return_type.brace_depth = return_type.brace_depth.saturating_sub(1);
            return_type.can_end = true;
        }
        self.expression_required = !closes_statement;
        self.brace_expression_required = self.expression_required;
        closes_statement
    }

    pub(super) fn observe_arrow(&mut self) {
        if let Some(return_type) = self.return_type.as_mut() {
            return_type.can_end = false;
        } else {
            self.next_brace = Some(BraceContext::ExpressionBody);
        }
        self.expression_required = true;
        self.brace_expression_required = true;
    }

    pub(super) fn observe_operator(&mut self, operator: u8) {
        let is_label = operator == b':' && std::mem::take(&mut self.possible_label);
        if let Some(return_type) = self.return_type.as_mut() {
            return_type.can_end = operator == b'>';
        }
        if operator == b';'
            && self
                .return_type
                .as_ref()
                .is_none_or(|return_type| return_type.brace_depth == 0)
        {
            self.pending_function_body = None;
            self.return_type = None;
            if self.class_headers.last().is_some_and(|class_header| {
                class_header.angle_depth == 0 && class_header.paren_depth == self.parens.len()
            }) {
                self.class_headers.pop();
            }
        }
        self.expression_required = !is_label && operator != b';';
        self.brace_expression_required = self.expression_required;
    }

    pub(super) fn finish_line(&mut self, can_start_expression: bool) {
        if !can_start_expression {
            self.expression_required = false;
            self.brace_expression_required = false;
        }
    }
}

fn identifier_can_be_label(identifier: &[u8]) -> bool {
    !matches!(
        identifier,
        b"await"
            | b"break"
            | b"case"
            | b"catch"
            | b"class"
            | b"const"
            | b"continue"
            | b"debugger"
            | b"declare"
            | b"default"
            | b"do"
            | b"else"
            | b"export"
            | b"extends"
            | b"finally"
            | b"for"
            | b"function"
            | b"if"
            | b"import"
            | b"let"
            | b"new"
            | b"return"
            | b"switch"
            | b"throw"
            | b"try"
            | b"var"
            | b"while"
            | b"with"
            | b"yield"
    )
}
