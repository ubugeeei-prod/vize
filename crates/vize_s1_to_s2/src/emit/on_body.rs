//! Block-body event handlers (`$event => { ... }`).

use oxc_ast::ast::Expression;
use vize_s2::expr::JsExpr;

use super::EmitCx;

pub(super) fn emit(cx: &mut EmitCx<'_>, source: &str, padding: Option<(&str, &str)>) {
    cx.buf.push("$event => {");
    if let Some((leading, _)) = padding {
        cx.buf.push(leading);
    }
    cx.buf.push(source);
    let trailing_has_newline = padding
        .map(|(_, trailing)| trailing.bytes().any(|byte| matches!(byte, b'\n' | b'\r')))
        .unwrap_or(false);
    if ends_in_line_comment(source) && !trailing_has_newline {
        cx.buf.newline();
    }
    if let Some((_, trailing)) = padding {
        cx.buf.push(trailing);
    }
    cx.buf.push("}");
}

pub(super) fn preserves_raw_function_handler(js: &JsExpr<'_>) -> bool {
    let is_block_function = match js.ast {
        Expression::ArrowFunctionExpression(arrow) => !arrow.expression,
        Expression::FunctionExpression(function) => function.body.is_some(),
        _ => false,
    };
    is_block_function && !ends_in_line_comment(js.source)
}

pub(super) fn ends_in_line_comment(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut index = 0usize;
    let mut state = ScanState::Code;
    let mut escaped = false;
    while index < bytes.len() {
        let byte = bytes[index];
        match state {
            ScanState::Code => match byte {
                b'\'' => enter_string(&mut state, &mut escaped, ScanState::Single),
                b'"' => enter_string(&mut state, &mut escaped, ScanState::Double),
                b'`' => enter_string(&mut state, &mut escaped, ScanState::Template),
                b'/' if bytes.get(index + 1) == Some(&b'/') => {
                    state = ScanState::Line;
                    index += 1;
                }
                b'/' if bytes.get(index + 1) == Some(&b'*') => {
                    state = ScanState::Block;
                    index += 1;
                }
                _ => {}
            },
            ScanState::Single if escaped => escaped = false,
            ScanState::Single if byte == b'\\' => escaped = true,
            ScanState::Single if byte == b'\'' => state = ScanState::Code,
            ScanState::Single => {}
            ScanState::Double if escaped => escaped = false,
            ScanState::Double if byte == b'\\' => escaped = true,
            ScanState::Double if byte == b'"' => state = ScanState::Code,
            ScanState::Double => {}
            ScanState::Template if escaped => escaped = false,
            ScanState::Template if byte == b'\\' => escaped = true,
            ScanState::Template if byte == b'`' => state = ScanState::Code,
            ScanState::Template => {}
            ScanState::Block if byte == b'*' && bytes.get(index + 1) == Some(&b'/') => {
                state = ScanState::Code;
                index += 1;
            }
            ScanState::Block => {}
            ScanState::Line if matches!(byte, b'\n' | b'\r') => state = ScanState::Code,
            ScanState::Line => {}
        }
        index += 1;
    }
    matches!(state, ScanState::Line)
}

fn enter_string(state: &mut ScanState, escaped: &mut bool, next: ScanState) {
    *state = next;
    *escaped = false;
}

#[derive(Clone, Copy)]
enum ScanState {
    Code,
    Single,
    Double,
    Template,
    Block,
    Line,
}
