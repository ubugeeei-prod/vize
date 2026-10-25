//! Declaration ownership for the existing concrete-colour scanner.

use super::lex::{identifier_end, identifier_eq, is_declaration_name, skip_trivia};

#[derive(Clone, Copy, PartialEq, Eq)]
pub(in crate::server::annotations::document_color) enum CssMode {
    Stylesheet,
    DeclarationList,
    Preprocessor,
    IndentedDeclarations,
    StylusDeclarations,
}

impl CssMode {
    pub(super) fn accepts_root_declarations(self) -> bool {
        self != Self::Stylesheet
    }

    pub(super) fn accepts_variable_names(self) -> bool {
        matches!(
            self,
            Self::Preprocessor | Self::IndentedDeclarations | Self::StylusDeclarations
        )
    }

    pub(super) fn has_line_comments(self) -> bool {
        self.accepts_variable_names()
    }

    pub(super) fn indented(self) -> bool {
        matches!(self, Self::IndentedDeclarations | Self::StylusDeclarations)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Header {
    Other,
    Supports,
    Container,
    Parameters,
}

#[derive(Clone, Copy)]
enum Parenthesis {
    Declaration,
    Variable,
}

#[derive(Default)]
pub(super) struct Conditions {
    header: Option<(usize, Header, usize)>,
    start: Option<(usize, usize, Parenthesis)>,
    value_depth: Option<usize>,
    rejected: bool,
}

impl Conditions {
    pub(super) fn reset(&mut self) {
        *self = Self::default();
    }

    pub(super) fn in_value(&self) -> bool {
        self.value_depth.is_some()
    }

    pub(super) fn open(
        &mut self,
        content: &str,
        statement: usize,
        at: usize,
        depth: usize,
        in_value: bool,
        mode: CssMode,
    ) {
        if in_value || self.in_value() {
            return;
        }
        let (header, keyword_end) = match self.header {
            Some((start, header, end)) if start == statement => (header, end),
            _ => {
                let header = header(content.as_bytes(), statement, at, mode);
                self.header = Some((statement, header.0, header.1));
                header
            }
        };
        let bytes = content.as_bytes();
        let mut name = at;
        while name > statement
            && bytes
                .get(name - 1)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            name -= 1;
        }
        let function = bytes.get(name..at).unwrap_or_default();
        #[cfg(test)]
        super::record_declaration_name_work(at - name);
        let keyword = at == keyword_end;
        let kind = match header {
            Header::Supports if function.is_empty() || keyword => Parenthesis::Declaration,
            Header::Container if function.eq_ignore_ascii_case(b"style") => {
                Parenthesis::Declaration
            }
            Header::Parameters => Parenthesis::Variable,
            _ => return,
        };
        self.start = Some((at + 1, depth + 1, kind));
        self.rejected = false;
    }

    pub(super) fn colon(&mut self, bytes: &[u8], at: usize, depth: usize) -> bool {
        let Some((start, owner_depth, kind)) = self.start else {
            return false;
        };
        if owner_depth != depth || self.in_value() || self.rejected {
            return false;
        }
        let variable = matches!(kind, Parenthesis::Variable);
        let head = skip_trivia(bytes, start, at);
        if variable && !bytes.get(head).is_some_and(|b| matches!(b, b'$' | b'@')) {
            self.rejected = true;
            return false;
        }
        if !is_declaration_name(bytes, start, at, variable) {
            self.rejected = true;
            return false;
        }
        self.value_depth = Some(depth);
        true
    }

    pub(super) fn comma(&mut self, at: usize, depth: usize) {
        if let Some((_, owner_depth, Parenthesis::Variable)) = self.start
            && owner_depth == depth
        {
            self.start = Some((at + 1, depth, Parenthesis::Variable));
            self.value_depth = None;
            self.rejected = false;
        }
    }

    pub(super) fn close(&mut self, depth: usize) {
        if self.value_depth.is_some_and(|owner| owner > depth) {
            self.value_depth = None;
        }
        if self.start.is_some_and(|(_, owner, _)| owner > depth) {
            self.start = None;
        }
    }
}

fn header(bytes: &[u8], start: usize, end: usize, mode: CssMode) -> (Header, usize) {
    #[cfg(test)]
    super::record_declaration_name_work(end.saturating_sub(start));
    let at = skip_trivia(bytes, start, end);
    if bytes.get(at) == Some(&b'@') {
        let name = at + 1;
        let stop = identifier_end(bytes, name, end);
        if identifier_eq(bytes, name, stop, b"supports") {
            return (Header::Supports, stop);
        }
        if identifier_eq(bytes, name, stop, b"container") {
            return (Header::Container, stop);
        }
        if mode.accepts_variable_names()
            && [b"mixin".as_slice(), b"function", b"include"]
                .iter()
                .any(|expected| identifier_eq(bytes, name, stop, expected))
        {
            return (Header::Parameters, stop);
        }
    } else if mode.accepts_variable_names() && bytes.get(at) == Some(&b'.') {
        return (Header::Parameters, at);
    }
    (Header::Other, at)
}

/// Stylus permits `theme = red` and indented `color red`; selectors do not grant values.
pub(super) fn stylus_value(content: &str, line: usize, at: usize, end: usize) -> Option<usize> {
    #[cfg(test)]
    super::record_declaration_name_work(at.saturating_sub(line));
    let bytes = content.as_bytes();
    if skip_trivia(bytes, line, at) != at {
        return None;
    }
    let name = at + usize::from(bytes.get(at) == Some(&b'$'));
    let stop = identifier_end(bytes, name, end);
    if stop == name {
        return None;
    }
    let next = skip_trivia(bytes, stop, end);
    if bytes.get(next) == Some(&b'=') {
        return Some(next + 1);
    }
    if !bytes
        .get(line)
        .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
    {
        return None;
    }
    if !bytes.get(stop).is_some_and(|b| matches!(b, b' ' | b'\t')) {
        return None;
    }
    if bytes.get(next) == Some(&b':') || !crate::ide::style::known_property(content.get(at..stop)?)
    {
        return None;
    }
    Some(stop)
}
