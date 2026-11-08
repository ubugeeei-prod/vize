//! Style conversion for the render NAPI boundary.

use super::types::StyleNapi;
use crate::terminal::{Color, Style};

pub(super) fn convert_style(style: StyleNapi) -> Style {
    let mut result = Style::default();

    if let Some(ref fg) = style.fg {
        result.fg = parse_color(fg);
    }
    if let Some(ref bg) = style.bg {
        result.bg = parse_color(bg);
    }
    result.bold = style.bold.unwrap_or(false);
    result.dim = style.dim.unwrap_or(false);
    result.italic = style.italic.unwrap_or(false);
    result.underline = style.underline.unwrap_or(false);
    result.reverse = style.inverse.unwrap_or(false);
    result.blink = style.blink.unwrap_or(false);
    result.hidden = style.hidden.unwrap_or(false);
    result.strikethrough = style.strikethrough.unwrap_or(false);

    result
}

pub(super) fn parse_color(value: &str) -> Option<Color> {
    if let Some(color) = Color::from_hex(value) {
        return Some(color);
    }

    if value.eq_ignore_ascii_case("black") {
        return Some(Color::Black);
    }
    if value.eq_ignore_ascii_case("red") {
        return Some(Color::Red);
    }
    if value.eq_ignore_ascii_case("green") {
        return Some(Color::Green);
    }
    if value.eq_ignore_ascii_case("yellow") {
        return Some(Color::Yellow);
    }
    if value.eq_ignore_ascii_case("blue") {
        return Some(Color::Blue);
    }
    if value.eq_ignore_ascii_case("magenta") {
        return Some(Color::Magenta);
    }
    if value.eq_ignore_ascii_case("cyan") {
        return Some(Color::Cyan);
    }
    if value.eq_ignore_ascii_case("white") {
        return Some(Color::White);
    }
    if value.eq_ignore_ascii_case("gray") || value.eq_ignore_ascii_case("grey") {
        return Some(Color::Gray);
    }
    if value.eq_ignore_ascii_case("lightred") {
        return Some(Color::LightRed);
    }
    if value.eq_ignore_ascii_case("lightgreen") {
        return Some(Color::LightGreen);
    }
    if value.eq_ignore_ascii_case("lightyellow") {
        return Some(Color::LightYellow);
    }
    if value.eq_ignore_ascii_case("lightblue") {
        return Some(Color::LightBlue);
    }
    if value.eq_ignore_ascii_case("lightmagenta") {
        return Some(Color::LightMagenta);
    }
    if value.eq_ignore_ascii_case("lightcyan") {
        return Some(Color::LightCyan);
    }
    if value.eq_ignore_ascii_case("lightwhite") {
        return Some(Color::LightWhite);
    }
    if value.eq_ignore_ascii_case("reset") {
        return Some(Color::Reset);
    }
    None
}
