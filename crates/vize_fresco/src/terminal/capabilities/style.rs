use super::{ColorSupport, TerminalCapabilities};
use crate::terminal::{Color, Style};

impl TerminalCapabilities {
    /// Clamp a style to the resolved color depth while preserving attributes.
    ///
    /// RGB colors are mapped to the nearest xterm palette entry for ANSI 256
    /// output and to the nearest canonical ANSI color for ANSI 16 output.
    pub fn adapt_style(self, style: Style) -> Style {
        let depth = self.color.value();
        Style {
            fg: adapt_color(style.fg, depth),
            bg: adapt_color(style.bg, depth),
            ..style
        }
    }

    /// Select a Unicode symbol or its required ASCII fallback.
    pub fn select_symbol<'a>(self, unicode: &'a str, ascii: &'a str) -> &'a str {
        if self.unicode.value() { unicode } else { ascii }
    }
}

fn adapt_color(color: Option<Color>, support: ColorSupport) -> Option<Color> {
    let color = color?;
    match support {
        ColorSupport::Monochrome => None,
        ColorSupport::TrueColor => Some(color),
        ColorSupport::Ansi256 => Some(match color {
            Color::Rgb(red, green, blue) => Color::Indexed(rgb_to_ansi256(red, green, blue)),
            other => other,
        }),
        ColorSupport::Ansi16 => Some(to_ansi16(color)),
    }
}

fn to_ansi16(color: Color) -> Color {
    match color {
        Color::Reset
        | Color::Black
        | Color::Red
        | Color::Green
        | Color::Yellow
        | Color::Blue
        | Color::Magenta
        | Color::Cyan
        | Color::White
        | Color::Gray
        | Color::LightRed
        | Color::LightGreen
        | Color::LightYellow
        | Color::LightBlue
        | Color::LightMagenta
        | Color::LightCyan
        | Color::LightWhite => color,
        Color::Indexed(index) if index < 16 => ANSI_COLORS[usize::from(index)].0,
        Color::Indexed(index) => {
            let (red, green, blue) = ansi256_to_rgb(index);
            nearest_ansi16(red, green, blue)
        }
        Color::Rgb(red, green, blue) => nearest_ansi16(red, green, blue),
    }
}

fn rgb_to_ansi256(red: u8, green: u8, blue: u8) -> u8 {
    let red_index = cube_index(red);
    let green_index = cube_index(green);
    let blue_index = cube_index(blue);
    let cube = 16 + 36 * red_index + 6 * green_index + blue_index;
    let cube_rgb = (
        cube_level(red_index),
        cube_level(green_index),
        cube_level(blue_index),
    );
    let gray_index = ((u16::from(red) + u16::from(green) + u16::from(blue)) / 3)
        .saturating_sub(8)
        .div_ceil(10)
        .min(23) as u8;
    let gray_level = 8 + gray_index * 10;
    let gray = 232 + gray_index;
    if distance((red, green, blue), (gray_level, gray_level, gray_level))
        < distance((red, green, blue), cube_rgb)
    {
        gray
    } else {
        cube
    }
}

fn cube_index(channel: u8) -> u8 {
    match channel {
        0..=47 => 0,
        48..=114 => 1,
        value => ((u16::from(value) - 35) / 40).min(5) as u8,
    }
}

fn cube_level(index: u8) -> u8 {
    if index == 0 { 0 } else { 55 + index * 40 }
}

fn ansi256_to_rgb(index: u8) -> (u8, u8, u8) {
    match index {
        0..=15 => ANSI_COLORS[usize::from(index)].1,
        16..=231 => {
            let offset = index - 16;
            (
                cube_level(offset / 36),
                cube_level((offset % 36) / 6),
                cube_level(offset % 6),
            )
        }
        _ => {
            let level = 8 + (index - 232) * 10;
            (level, level, level)
        }
    }
}

fn nearest_ansi16(red: u8, green: u8, blue: u8) -> Color {
    ANSI_COLORS
        .iter()
        .min_by_key(|(_, candidate)| distance((red, green, blue), *candidate))
        .map_or(Color::Reset, |(color, _)| *color)
}

fn distance(left: (u8, u8, u8), right: (u8, u8, u8)) -> u32 {
    let red = i32::from(left.0) - i32::from(right.0);
    let green = i32::from(left.1) - i32::from(right.1);
    let blue = i32::from(left.2) - i32::from(right.2);
    (red * red + green * green + blue * blue) as u32
}

const ANSI_COLORS: [(Color, (u8, u8, u8)); 16] = [
    (Color::Black, (0, 0, 0)),
    (Color::Red, (128, 0, 0)),
    (Color::Green, (0, 128, 0)),
    (Color::Yellow, (128, 128, 0)),
    (Color::Blue, (0, 0, 128)),
    (Color::Magenta, (128, 0, 128)),
    (Color::Cyan, (0, 128, 128)),
    (Color::White, (192, 192, 192)),
    (Color::Gray, (128, 128, 128)),
    (Color::LightRed, (255, 0, 0)),
    (Color::LightGreen, (0, 255, 0)),
    (Color::LightYellow, (255, 255, 0)),
    (Color::LightBlue, (0, 0, 255)),
    (Color::LightMagenta, (255, 0, 255)),
    (Color::LightCyan, (0, 255, 255)),
    (Color::LightWhite, (255, 255, 255)),
];
