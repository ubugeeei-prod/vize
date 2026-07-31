//! Find CSS colour literals in a byte range of authored text.
//!
//! # Coverage
//!
//! `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, and the `rgb()` / `rgba()`
//! functional notation in both the legacy comma form (`rgb(255, 0, 0)`) and the
//! modern space form (`rgb(255 0 0 / 50%)`).
//!
//! Named colours (`red`) and `hsl()` are **not** recognised: see #3502. A
//! missing swatch is invisible; a wrong one is not, so nothing is guessed.
//!
//! CSS comments are skipped, because a swatch rendered inside `/* ... */` would
//! offer to rewrite text the stylesheet never reads.

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

pub(crate) fn colors_in(content: &str, region: (usize, usize)) -> Vec<ColorLiteral> {
    let (region_start, region_end) = region;
    let bytes = content.as_bytes();
    let mut found = Vec::new();
    let mut cursor = region_start;

    while cursor < region_end {
        if content[cursor..region_end].starts_with("/*") {
            cursor = content[cursor..region_end]
                .find("*/")
                .map_or(region_end, |relative| cursor + relative + 2);
            continue;
        }

        // `rgb(` only starts a colour at a token boundary: the `r` in
        // `border-radius` is not the start of a function call.
        let literal = if bytes[cursor] == b'#' {
            hex_literal(content, cursor, region_end)
        } else if (bytes[cursor] | 0x20) == b'r'
            && !is_ident_byte(bytes.get(cursor.wrapping_sub(1)))
        {
            rgb_literal(content, cursor, region_end)
        } else {
            None
        };

        match literal {
            Some(literal) => {
                cursor = literal.end;
                found.push(literal);
            }
            None => cursor += 1,
        }
    }

    found
}

#[inline]
fn is_ident_byte(byte: Option<&u8>) -> bool {
    byte.is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
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
    if !matches!(digits.len(), 3 | 4 | 6 | 8) || is_ident_byte(bytes.get(end)) {
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
fn rgb_literal(content: &str, start: usize, limit: usize) -> Option<ColorLiteral> {
    let head = &content[start..limit];
    let open = head.find('(')?;
    let name = &head[..open];
    if !name.eq_ignore_ascii_case("rgb") && !name.eq_ignore_ascii_case("rgba") {
        return None;
    }
    let close = head[open..].find(')')? + open;
    let end = start + close + 1;

    let mut components = head[open + 1..close]
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
