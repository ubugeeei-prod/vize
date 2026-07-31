//! Document colours (`textDocument/documentColor`, `textDocument/colorPresentation`).
//!
//! The editor paints a swatch next to every colour literal and opens a picker
//! when the swatch is clicked. `@vue/language-server` 3.3.8 advertises
//! `colorProvider: true`; Maestro advertised nothing and answered
//! `-32601 Method not found`, so a `.vue` file was the one place in a project
//! where the picker did not appear (#3456).
//!
//! # Where colours are looked for
//!
//! Only where CSS is authored: every `<style>` block's content, and the value
//! of a static `style="…"` attribute in the template. A bound `:style="…"` is a
//! JavaScript expression, not CSS, and is left alone.
//!
//! Ranges are authored `.vue` coordinates, never virtual TypeScript.
#![allow(clippy::disallowed_macros, clippy::disallowed_types)]

mod named;
mod scan;

use tower_lsp::lsp_types::{Color, ColorInformation, ColorPresentation, Position, Range};

use crate::ide::offset_to_position;

pub(super) struct DocumentColorService;

impl DocumentColorService {
    /// Every colour literal in the document's CSS, in document order.
    pub(super) fn colors(content: &str, filename: &str) -> Vec<ColorInformation> {
        let mut colors = Vec::new();
        for region in css_regions(content, filename) {
            for literal in scan::colors_in(content, region.range, region.mode) {
                colors.push(ColorInformation {
                    range: Range {
                        start: position_at(content, literal.start),
                        end: position_at(content, literal.end),
                    },
                    color: Color {
                        red: literal.red,
                        green: literal.green,
                        blue: literal.blue,
                        alpha: literal.alpha,
                    },
                });
            }
        }
        colors
    }

    /// The notations the picker offers for a chosen colour.
    ///
    /// The label doubles as the inserted text (the LSP default when no
    /// `textEdit` is supplied), so each one must be valid CSS on its own. Hex
    /// comes first because it is what the picker shows as the primary value.
    pub(super) fn presentations(color: Color) -> Vec<ColorPresentation> {
        let red = to_byte(color.red);
        let green = to_byte(color.green);
        let blue = to_byte(color.blue);
        let opaque = to_byte(color.alpha) == 255;

        let hex = if opaque {
            format!("#{red:02x}{green:02x}{blue:02x}")
        } else {
            format!(
                "#{red:02x}{green:02x}{blue:02x}{:02x}",
                to_byte(color.alpha)
            )
        };
        let functional = if opaque {
            format!("rgb({red}, {green}, {blue})")
        } else {
            format!("rgba({red}, {green}, {blue}, {})", trim_alpha(color.alpha))
        };
        let [hue, saturation, lightness] = rgb_to_hsl(color.red, color.green, color.blue);
        let hsl = if opaque {
            format!(
                "hsl({} {}% {}%)",
                trim_decimal(hue),
                trim_decimal(saturation * 100.0),
                trim_decimal(lightness * 100.0)
            )
        } else {
            format!(
                "hsl({} {}% {}% / {})",
                trim_decimal(hue),
                trim_decimal(saturation * 100.0),
                trim_decimal(lightness * 100.0),
                trim_alpha(color.alpha)
            )
        };

        [hex, functional, hsl]
            .into_iter()
            .map(|label| ColorPresentation {
                label,
                text_edit: None,
                additional_text_edits: None,
            })
            .collect()
    }
}

#[derive(Clone, Copy)]
struct CssRegion {
    range: (usize, usize),
    mode: scan::CssMode,
}

/// Byte regions of the document that hold CSS.
fn css_regions(content: &str, filename: &str) -> Vec<CssRegion> {
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: filename.into(),
        ..Default::default()
    };
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
        return Vec::new();
    };

    let mut regions: Vec<CssRegion> = descriptor
        .styles
        .iter()
        .map(|block| CssRegion {
            range: (block.loc.start, block.loc.end),
            mode: match block.lang.as_deref() {
                Some("sass") => scan::CssMode::IndentedSass,
                Some("css") | None => scan::CssMode::Stylesheet,
                Some(_) => scan::CssMode::Preprocessor,
            },
        })
        .collect();

    if let Some(template) = descriptor.template.as_ref() {
        regions.extend(
            static_style_attributes(content, (template.loc.start, template.loc.end))
                .into_iter()
                .map(|range| CssRegion {
                    range,
                    mode: scan::CssMode::DeclarationList,
                }),
        );
    }

    regions.sort_unstable_by_key(|region| region.range);
    regions
}

/// Value spans of every static `style="…"` attribute in `region`.
///
/// `:style` and `v-bind:style` are skipped: their value is a JavaScript
/// expression, so a colour inside one is a string literal in code rather than
/// CSS the editor may rewrite in place.
fn static_style_attributes(content: &str, region: (usize, usize)) -> Vec<(usize, usize)> {
    let (region_start, region_end) = region;
    let bytes = content.as_bytes();
    let mut spans = Vec::new();
    let mut cursor = region_start;

    while let Some(relative) = content[cursor..region_end].find("style") {
        let name_start = cursor + relative;
        cursor = name_start + "style".len();

        // A preceding identifier byte means this is `background-style` or the
        // tail of `:style`; either way the attribute name is not `style`.
        if bytes
            .get(name_start.wrapping_sub(1))
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':'))
        {
            continue;
        }
        if bytes.get(cursor) != Some(&b'=') {
            continue;
        }
        let quote = match bytes.get(cursor + 1) {
            Some(&byte @ (b'"' | b'\'')) => byte,
            _ => continue,
        };
        let value_start = cursor + 2;
        let Some(relative_end) = content[value_start..region_end].find(quote as char) else {
            break;
        };
        let value_end = value_start + relative_end;
        spans.push((value_start, value_end));
        cursor = value_end + 1;
    }

    spans
}

fn position_at(content: &str, offset: usize) -> Position {
    let (line, character) = offset_to_position(content, offset);
    Position { line, character }
}

fn to_byte(channel: f32) -> u32 {
    (channel.clamp(0.0, 1.0) * 255.0).round() as u32
}

fn rgb_to_hsl(red: f32, green: f32, blue: f32) -> [f32; 3] {
    let red = f64::from(red.clamp(0.0, 1.0));
    let green = f64::from(green.clamp(0.0, 1.0));
    let blue = f64::from(blue.clamp(0.0, 1.0));
    let maximum = red.max(green).max(blue);
    let minimum = red.min(green).min(blue);
    let delta = maximum - minimum;
    let lightness = (maximum + minimum) / 2.0;
    if delta == 0.0 {
        return [0.0, 0.0, lightness as f32];
    }

    let saturation = delta / (1.0 - (2.0 * lightness - 1.0).abs());
    let sector = if maximum == red {
        (green - blue) / delta
    } else if maximum == green {
        (blue - red) / delta + 2.0
    } else {
        (red - green) / delta + 4.0
    };
    [
        (sector * 60.0).rem_euclid(360.0) as f32,
        saturation as f32,
        lightness as f32,
    ]
}

fn trim_decimal(value: f32) -> String {
    let text = format!("{:.2}", value);
    text.trim_end_matches('0').trim_end_matches('.').to_string()
}

/// Alpha as CSS writes it: `1`, `0.5`, `0.35` — never `0.500000`.
fn trim_alpha(alpha: f32) -> String {
    trim_decimal(alpha.clamp(0.0, 1.0))
}

#[cfg(test)]
#[path = "document_color/hsl_presentation_tests.rs"]
mod hsl_presentation_tests;

#[cfg(test)]
mod tests;
