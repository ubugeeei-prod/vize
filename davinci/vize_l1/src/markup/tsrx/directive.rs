//! Statement containers and the `@if`/`@for`/`@switch`/`@try` directives.
//! Setup statements, heads and tests are parsed by the host parser; only the
//! directive keywords and branch structure are read here.

use oxc_ast::ast::{Expression, Statement};
use oxc_parser::{Kind, MarkupSite};
use vize_l0::{Box, Span, Vec};

use super::diagnostic::{self as error, BodyKind, Exit};
use super::grammar::{Scan, opens, skip_trivia, span_of, word, word_is};
use super::tree::{Block, Catch, For, If, IfBranch, Node, Switch, SwitchCase, Try};

impl<'a> Scan<'_, '_, 'a> {
    /// The construct whose `@` is at `at`.
    pub(super) fn at_node(&mut self, at: u32) -> Option<Node<'a>> {
        if self.byte(at + 1) == Some(b'{') {
            return self.code_block(at, Some(BodyKind::CodeBlock));
        }
        match word(self.bytes(), at + 1) {
            Some(b"if") => self.if_node(at),
            Some(b"for") => self.for_node(at),
            Some(b"switch") => self.switch_node(at),
            Some(b"try") => self.try_node(at),
            _ => self.fail(error::unexpected(Span::new(at, at + 1))),
        }
    }

    /// `@{ … }` at `at`; `body` is `None` for a function body, where exits are legal.
    pub(super) fn code_block(&mut self, at: u32, body: Option<BodyKind>) -> Option<Node<'a>> {
        let mut block = self.block(at + 1, body)?;
        block.span = Span::new(at, block.span.end);
        Some(Node::CodeBlock(Box::new_in(block, &self.alloc)))
    }

    /// The braces at `open`: setup statements, then at most one rendered node.
    pub(super) fn block(&mut self, open: u32, body: Option<BodyKind>) -> Option<Block<'a>> {
        self.seat(open + 1)?;
        let mut statements = Vec::new_in(&self.alloc);
        let mut render: Option<Node<'a>> = None;
        let close = loop {
            let (kind, span) = self.token();
            match kind {
                Kind::RCurly => break span.end,
                Kind::Eof => return self.fail(error::expected("}", span)),
                Kind::Semicolon => self.cursor.bump(),
                Kind::LAngle | Kind::At
                    if kind == Kind::LAngle
                        || opens(self.bytes(), span.start, MarkupSite::Expression) =>
                {
                    let node = if kind == Kind::LAngle {
                        self.markup(span.start)?
                    } else {
                        self.at_node(span.start)?
                    };
                    if render.is_some() {
                        return self.fail(error::single_output(node.span()));
                    }
                    self.seat(node.span().end)?;
                    render = Some(node);
                }
                _ => {
                    let statement = self.cursor.statement();
                    if self.cursor.failed() {
                        return None;
                    }
                    if render.is_some() {
                        return self.fail(error::statement_after_output(span_of(&statement)));
                    }
                    statements.push(statement);
                }
            }
        };
        if let Some(body) = body
            && let Some((exit, at)) = first_exit(&statements, false, false)
        {
            return self.fail(error::invalid_exit(body, exit, at));
        }
        Some(Block {
            span: Span::new(open, close),
            statements,
            render,
        })
    }

    /// `( Expression )` at the current host token; leaves the token after `)`.
    fn parenthesized(&mut self) -> Option<Expression<'a>> {
        self.expect(Kind::LParen)?;
        self.cursor.bump();
        let expression = self.cursor.sequence();
        if self.cursor.failed() {
            return None;
        }
        self.expect(Kind::RParen)?;
        self.cursor.bump();
        Some(expression)
    }

    /// The `{` of a directive body at the current host token.
    fn body_open(&mut self) -> Option<u32> {
        let (kind, span) = self.token();
        if kind == Kind::LCurly {
            Some(span.start)
        } else {
            self.fail(error::directive_body_expected(span))
        }
    }

    /// The `{` written at `offset` after a keyword such as `@else` or `@empty`.
    fn raw_body_open(&mut self, offset: u32) -> Option<u32> {
        let open = skip_trivia(self.bytes(), offset);
        if self.byte(open) == Some(b'{') {
            Some(open)
        } else {
            self.fail(error::directive_body_expected(Span::new(open, open + 1)))
        }
    }

    fn if_node(&mut self, at: u32) -> Option<Node<'a>> {
        let mut branches = Vec::new_in(&self.alloc);
        let mut branch_start = at;
        let mut head = at + 3;
        loop {
            self.seat(head)?;
            let test = self.parenthesized()?;
            let open = self.body_open()?;
            let body = self.block(open, Some(BodyKind::If))?;
            let end = body.span.end;
            branches.push(IfBranch {
                span: Span::new(branch_start, end),
                test: Some(test),
                body,
            });
            let next = skip_trivia(self.bytes(), end);
            if !word_is(self.bytes(), next, b"@else") {
                self.bare_branch(next, &[b"else"], "if")?;
                break;
            }
            branch_start = next;
            let after = skip_trivia(self.bytes(), next + 5);
            if word_is(self.bytes(), after, b"if") {
                head = after + 2;
                continue;
            }
            let open = self.raw_body_open(after)?;
            let body = self.block(open, Some(BodyKind::If))?;
            let span = Span::new(branch_start, body.span.end);
            branches.push(IfBranch {
                span,
                test: None,
                body,
            });
            break;
        }
        let end = branches.last().map_or(at, |branch| branch.span.end);
        let node = If {
            span: Span::new(at, end),
            branches,
        };
        Some(Node::If(Box::new_in(node, &self.alloc)))
    }

    fn for_node(&mut self, at: u32) -> Option<Node<'a>> {
        self.seat(at + 4)?;
        let r#await = self.token().0 == Kind::Await;
        if r#await {
            self.cursor.bump();
        }
        self.expect(Kind::LParen)?;
        let head = self.token().1;
        let Some((left, right)) = self.cursor.for_of_head() else {
            if self.cursor.failed() {
                return None;
            }
            return self.fail(error::for_of_only(head));
        };
        let mut index = None;
        let mut key = None;
        if self.token().0 == Kind::Semicolon {
            self.cursor.bump();
            if self.at_contextual("index") {
                self.cursor.bump();
                index = Some(self.cursor.binding_identifier());
                if self.token().0 == Kind::Semicolon {
                    self.cursor.bump();
                }
            }
            if self.at_contextual("key") {
                self.cursor.bump();
                key = Some(self.cursor.expression());
            }
            if self.at_contextual("index") {
                let span = self.token().1;
                return self.fail(error::for_clause(span));
            }
        }
        if self.cursor.failed() {
            return None;
        }
        self.expect(Kind::RParen)?;
        self.cursor.bump();
        let open = self.body_open()?;
        let body = self.block(open, Some(BodyKind::For))?;
        let mut end = body.span.end;
        let next = skip_trivia(self.bytes(), end);
        self.bare_branch(next, &[b"empty"], "for")?;
        let empty = if word_is(self.bytes(), next, b"@empty") {
            let open = self.raw_body_open(next + 6)?;
            let empty = self.block(open, Some(BodyKind::For))?;
            end = empty.span.end;
            Some(empty)
        } else {
            None
        };
        let node = For {
            span: Span::new(at, end),
            r#await,
            left,
            right,
            index,
            key,
            body,
            empty,
        };
        Some(Node::For(Box::new_in(node, &self.alloc)))
    }

    /// A next branch written without its `@` (`else`, `empty`, `pending`, `catch`).
    fn bare_branch(&mut self, offset: u32, branches: &[&[u8]], directive: &str) -> Option<()> {
        let Some(branch) = branches
            .iter()
            .find(|branch| word_is(self.bytes(), offset, branch))
        else {
            return Some(());
        };
        let branch = core::str::from_utf8(branch).unwrap_or_default();
        let at = Span::new(offset, offset + branch.len() as u32);
        self.fail(error::directive_branch_expected(branch, directive, at))
    }

    fn at_contextual(&self, keyword: &str) -> bool {
        let (kind, span) = self.token();
        kind == Kind::Ident && self.text(span) == keyword
    }

    fn switch_node(&mut self, at: u32) -> Option<Node<'a>> {
        self.seat(at + 7)?;
        let discriminant = self.parenthesized()?;
        self.expect(Kind::LCurly)?;
        self.cursor.bump();
        let mut cases = Vec::new_in(&self.alloc);
        let end = loop {
            let (kind, span) = self.token();
            if kind == Kind::RCurly {
                break span.end;
            }
            let start = span.start;
            let test = if kind == Kind::At && word_is(self.bytes(), start + 1, b"case") {
                self.seat(start + 5)?;
                let test = self.cursor.expression();
                if self.cursor.failed() {
                    return None;
                }
                Some(test)
            } else if kind == Kind::At && word_is(self.bytes(), start + 1, b"default") {
                self.seat(start + 8)?;
                None
            } else {
                return self.fail(error::unexpected(span));
            };
            self.expect(Kind::Colon)?;
            self.cursor.bump();
            let open = self.expect(Kind::LCurly)?.start;
            let body = self.block(open, Some(BodyKind::SwitchCase))?;
            let case_end = body.span.end;
            cases.push(SwitchCase {
                span: Span::new(start, case_end),
                test,
                body,
            });
            self.seat(case_end)?;
        };
        let node = Switch {
            span: Span::new(at, end),
            discriminant,
            cases,
        };
        Some(Node::Switch(Box::new_in(node, &self.alloc)))
    }

    fn try_node(&mut self, at: u32) -> Option<Node<'a>> {
        self.seat(at + 4)?;
        let open = self.body_open()?;
        let body = self.block(open, Some(BodyKind::Try))?;
        let mut end = body.span.end;
        let mut next = skip_trivia(self.bytes(), end);
        let mut pending = None;
        if word_is(self.bytes(), next, b"@pending") {
            let open = self.raw_body_open(next + 8)?;
            let block = self.block(open, Some(BodyKind::Try))?;
            end = block.span.end;
            next = skip_trivia(self.bytes(), end);
            pending = Some(block);
        }
        let mut catch = None;
        if word_is(self.bytes(), next, b"@catch") {
            let start = next;
            self.seat(next + 6)?;
            let params = if self.token().0 == Kind::LParen {
                let params = self.cursor.formal_parameters();
                if self.cursor.failed() {
                    return None;
                }
                Some(params)
            } else {
                None
            };
            let open = self.body_open()?;
            let block = self.block(open, Some(BodyKind::Try))?;
            end = block.span.end;
            next = skip_trivia(self.bytes(), end);
            catch = Some(Catch {
                span: Span::new(start, end),
                params,
                body: block,
            });
        }
        self.bare_branch(next, &[b"pending", b"catch"], "try")?;
        if word_is(self.bytes(), next, b"@finally") {
            return self.fail(error::try_finally(Span::new(next, next + 8)));
        }
        if pending.is_none() && catch.is_none() {
            return self.fail(error::try_handler_missing(Span::new(at, end)));
        }
        let node = Try {
            span: Span::new(at, end),
            body,
            pending,
            catch,
        };
        Some(Node::Try(Box::new_in(node, &self.alloc)))
    }
}

/// The first `return`, `break` or `continue` that would leave a template body.
/// Functions and classes are boundaries; loops own their `break`/`continue`,
/// switches own their `break`, and labeled jumps target their own labels.
fn first_exit(
    statements: &[Statement<'_>],
    in_loop: bool,
    in_switch: bool,
) -> Option<(Exit, Span)> {
    statements
        .iter()
        .find_map(|statement| exit_in(statement, in_loop, in_switch))
}

fn exit_in(statement: &Statement<'_>, in_loop: bool, in_switch: bool) -> Option<(Exit, Span)> {
    match statement {
        Statement::ReturnStatement(_) => Some((Exit::Return, span_of(statement))),
        Statement::BreakStatement(jump) if jump.label.is_none() && !in_loop && !in_switch => {
            Some((Exit::Break, span_of(statement)))
        }
        Statement::ContinueStatement(jump) if jump.label.is_none() && !in_loop => {
            Some((Exit::Continue, span_of(statement)))
        }
        Statement::BlockStatement(block) => first_exit(&block.body, in_loop, in_switch),
        Statement::IfStatement(branch) => {
            exit_in(&branch.consequent, in_loop, in_switch).or_else(|| {
                branch
                    .alternate
                    .as_ref()
                    .and_then(|alternate| exit_in(alternate, in_loop, in_switch))
            })
        }
        Statement::TryStatement(attempt) => first_exit(&attempt.block.body, in_loop, in_switch)
            .or_else(|| {
                attempt
                    .handler
                    .as_ref()
                    .and_then(|handler| first_exit(&handler.body.body, in_loop, in_switch))
            })
            .or_else(|| {
                attempt
                    .finalizer
                    .as_ref()
                    .and_then(|finalizer| first_exit(&finalizer.body, in_loop, in_switch))
            }),
        Statement::LabeledStatement(labeled) => exit_in(&labeled.body, in_loop, in_switch),
        Statement::ForStatement(looped) => exit_in(&looped.body, true, in_switch),
        Statement::ForInStatement(looped) => exit_in(&looped.body, true, in_switch),
        Statement::ForOfStatement(looped) => exit_in(&looped.body, true, in_switch),
        Statement::WhileStatement(looped) => exit_in(&looped.body, true, in_switch),
        Statement::DoWhileStatement(looped) => exit_in(&looped.body, true, in_switch),
        Statement::SwitchStatement(switch) => switch
            .cases
            .iter()
            .find_map(|case| first_exit(&case.consequent, in_loop, true)),
        _ => None,
    }
}
