//! Find CSS colour literals in a byte range of authored text.
//!
//! # Coverage
//!
//! CSS named colours, hex forms, and legacy/modern `rgb()` / `rgba()`.
//!
//! `hsl()` is not recognised yet: see #3502.
//!
//! CSS comments are skipped so the picker never rewrites inactive text.

mod lex;

use self::lex::{
    decode_identifier, identifier_end, is_declaration_name, is_identifier_boundary,
    is_identifier_byte, pair_at, skip_comment, skip_function, skip_string, skipped_function_end,
};
use super::named;

#[cfg(test)]
std::thread_local! {
    static SCAN_STEPS: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static RGB_PROBES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    static DECLARATION_NAME_WORK: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

#[cfg(test)]
fn record_declaration_name_work(bytes: usize) {
    DECLARATION_NAME_WORK.set(DECLARATION_NAME_WORK.get() + bytes);
}

/// A colour literal: `(start, end, red, green, blue, alpha)` with the channels
/// already normalised to 0.0..=1.0, which is what LSP's `Color` carries.
pub(crate) struct ColorLiteral {
    pub(crate) start: usize,
    pub(crate) end: usize,
    pub(crate) red: f32,
    pub(crate) green: f32,
    pub(crate) blue: f32,
    pub(crate) alpha: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum CssMode {
    Stylesheet,
    DeclarationList,
    Preprocessor,
    IndentedSass,
}

impl CssMode {
    fn accepts_root_declarations(self) -> bool {
        self != Self::Stylesheet
    }

    fn accepts_variable_names(self) -> bool {
        matches!(self, Self::Preprocessor | Self::IndentedSass)
    }

    fn has_line_comments(self) -> bool {
        matches!(self, Self::Preprocessor | Self::IndentedSass)
    }
}

pub(crate) fn colors_in(content: &str, region: (usize, usize), mode: CssMode) -> Vec<ColorLiteral> {
    let (region_start, region_end) = region;
    let bytes = content.as_bytes();
    let mut found = Vec::new();
    let mut cursor = region_start;
    let mut block_depth = 0usize;
    let mut parenthesis_depth = 0usize;
    let mut in_value = false;
    let mut statement_start = region_start;
    let mut tentative_value = None;

    while cursor < region_end {
        #[cfg(test)]
        SCAN_STEPS.set(SCAN_STEPS.get() + 1);
        if pair_at(bytes, cursor, b"/*") {
            cursor = skip_comment(bytes, cursor, region_end);
            continue;
        }
        if mode.has_line_comments() && pair_at(bytes, cursor, b"//") {
            cursor = bytes[cursor..region_end]
                .iter()
                .position(|byte| matches!(byte, b'\r' | b'\n'))
                .map_or(region_end, |offset| cursor + offset);
            continue;
        }
        if matches!(bytes[cursor], b'\'' | b'"') {
            cursor = skip_string(bytes, cursor, region_end);
            continue;
        }
        if mode == CssMode::IndentedSass
            && parenthesis_depth == 0
            && matches!(bytes[cursor], b'\r' | b'\n')
        {
            tentative_value = None;
            in_value = false;
            cursor += 1;
            statement_start = cursor;
            continue;
        }

        match bytes[cursor] {
            b'{' => {
                if let Some(checkpoint) = tentative_value.take() {
                    found.truncate(checkpoint);
                }
                block_depth += 1;
                parenthesis_depth = 0;
                in_value = false;
                cursor += 1;
                statement_start = cursor;
                continue;
            }
            b'}' => {
                tentative_value = None;
                block_depth = block_depth.saturating_sub(1);
                parenthesis_depth = 0;
                in_value = false;
                cursor += 1;
                statement_start = cursor;
                continue;
            }
            b':' if !in_value
                && parenthesis_depth == 0
                && (mode.accepts_root_declarations() || block_depth > 0)
                && is_declaration_name(
                    bytes,
                    statement_start,
                    cursor,
                    mode.accepts_variable_names(),
                ) =>
            {
                if mode != CssMode::DeclarationList {
                    tentative_value.get_or_insert(found.len());
                }
                in_value = true;
                cursor += 1;
                continue;
            }
            b';' if parenthesis_depth == 0 => {
                tentative_value = None;
                in_value = false;
                cursor += 1;
                statement_start = cursor;
                continue;
            }
            b'(' => parenthesis_depth += 1,
            b')' => parenthesis_depth = parenthesis_depth.saturating_sub(1),
            _ => {}
        }

        let mut identifier_token_end = None;
        let literal = if bytes[cursor] == b'#' {
            hex_literal(content, cursor, region_end)
        } else if (bytes[cursor].is_ascii_alphabetic() || bytes[cursor] == b'\\')
            && is_identifier_boundary(bytes, cursor, region_start)
        {
            let end = identifier_end(bytes, cursor, region_end);
            identifier_token_end = Some(end);
            if let Some(skipped_end) = skipped_function_end(content, cursor, region_end) {
                cursor = skipped_end;
                continue;
            }
            if bytes.get(end) == Some(&b'(') {
                if !is_rgb_function_name(content, cursor, end) {
                    None
                } else if let Some(literal) = rgb_literal(content, cursor, end, region_end) {
                    Some(literal)
                } else {
                    cursor = skip_function(bytes, end, region_end);
                    continue;
                }
            } else if in_value {
                named_color_literal(content, cursor, end)
            } else {
                None
            }
        } else {
            None
        };

        match literal {
            Some(literal) => {
                cursor = literal.end;
                found.push(literal);
            }
            None => cursor = identifier_token_end.unwrap_or(cursor + 1),
        }
    }

    found
}

/// Parse one complete identifier before deciding whether it is a colour.
fn named_color_literal(content: &str, start: usize, end: usize) -> Option<ColorLiteral> {
    let mut decoded = [0u8; 32];
    let decoded_len = decode_identifier(content.as_bytes(), start, end, &mut decoded)?;
    let [red, green, blue, alpha] = named::rgba_bytes(&decoded[..decoded_len])?;
    Some(ColorLiteral {
        start,
        end,
        red: red as f32 / 255.0,
        green: green as f32 / 255.0,
        blue: blue as f32 / 255.0,
        alpha: alpha as f32 / 255.0,
    })
}

fn is_rgb_function_name(content: &str, start: usize, end: usize) -> bool {
    let mut decoded = [0u8; 4];
    let Some(len) = decode_identifier(content.as_bytes(), start, end, &mut decoded) else {
        return false;
    };
    decoded[..len].eq_ignore_ascii_case(b"rgb") || decoded[..len].eq_ignore_ascii_case(b"rgba")
}

/// `#` followed by exactly 3, 4, 6 or 8 hex digits and nothing that could
/// continue the token. `#abcdefg` is not a colour, and neither is `#abcde`.
fn hex_literal(content: &str, start: usize, limit: usize) -> Option<ColorLiteral> {
    let bytes = content.as_bytes();
    let mut end = start + 1;
    while end < limit && bytes[end].is_ascii_hexdigit() {
        end += 1;
    }
    let digits = &content[start + 1..end];
    if !matches!(digits.len(), 3 | 4 | 6 | 8) || is_identifier_byte(bytes.get(end)) {
        return None;
    }

    // The 3- and 4-digit forms double each digit: `#f0a` is `#ff00aa`.
    let expand = digits.len() <= 4;
    let mut channels = [1.0f32; 4];
    for (index, channel) in channels.iter_mut().enumerate() {
        let raw = if expand {
            match digits.as_bytes().get(index) {
                Some(byte) => {
                    let value = hex_value(*byte)?;
                    value * 16 + value
                }
                None => break,
            }
        } else {
            // Running out of digits means the form carries no alpha, which is
            // the 6-digit case: stop, leaving alpha at its opaque default.
            let (Some(high), Some(low)) = (
                digits.as_bytes().get(index * 2),
                digits.as_bytes().get(index * 2 + 1),
            ) else {
                break;
            };
            hex_value(*high)? * 16 + hex_value(*low)?
        };
        *channel = raw as f32 / 255.0;
    }

    Some(ColorLiteral {
        start,
        end,
        red: channels[0],
        green: channels[1],
        blue: channels[2],
        alpha: channels[3],
    })
}

fn hex_value(byte: u8) -> Option<u32> {
    (byte as char).to_digit(16)
}

/// `rgb(...)` / `rgba(...)`, comma- or space-separated, with an optional alpha
/// after `,` or `/`. Components may be numbers or percentages.
fn rgb_literal(
    content: &str,
    start: usize,
    identifier_end: usize,
    limit: usize,
) -> Option<ColorLiteral> {
    #[cfg(test)]
    RGB_PROBES.set(RGB_PROBES.get() + 1);
    let arguments_start = identifier_end + 1;
    let close = content[arguments_start..limit].find(')')? + arguments_start;
    let end = close + 1;

    let mut components = content[arguments_start..close]
        .split(|ch: char| ch == ',' || ch == '/' || ch.is_ascii_whitespace())
        .filter(|part| !part.is_empty());

    let red = channel(components.next()?, 255.0)?;
    let green = channel(components.next()?, 255.0)?;
    let blue = channel(components.next()?, 255.0)?;
    let alpha = match components.next() {
        Some(part) => channel(part, 1.0)?,
        None => 1.0,
    };
    if components.next().is_some() {
        return None;
    }

    Some(ColorLiteral {
        start,
        end,
        red,
        green,
        blue,
        alpha,
    })
}

#[cfg(test)]
pub(super) fn colors_in_with_metrics(
    content: &str,
    region: (usize, usize),
    declaration_list: bool,
) -> (Vec<ColorLiteral>, usize, usize) {
    SCAN_STEPS.set(0);
    RGB_PROBES.set(0);
    DECLARATION_NAME_WORK.set(0);
    let mode = if declaration_list {
        CssMode::DeclarationList
    } else {
        CssMode::Stylesheet
    };
    let colors = colors_in(content, region, mode);
    (
        colors,
        SCAN_STEPS.get() + DECLARATION_NAME_WORK.get(),
        RGB_PROBES.get(),
    )
}

/// One component, normalised to 0.0..=1.0. `full` is the value that maps to 1.0
/// for the plain-number form: 255 for a colour channel, 1 for alpha.
fn channel(part: &str, full: f32) -> Option<f32> {
    let part = part.trim();
    let value = match part.strip_suffix('%') {
        Some(percent) => percent.trim().parse::<f32>().ok()? / 100.0,
        None => part.parse::<f32>().ok()? / full,
    };
    value.is_finite().then(|| value.clamp(0.0, 1.0))
}
