//! `hsl()` / `hsla()` parsing and HSL-to-sRGB conversion.

use super::ColorLiteral;

pub(super) fn literal(
    content: &str,
    start: usize,
    identifier_end: usize,
    limit: usize,
) -> Option<ColorLiteral> {
    let arguments_start = identifier_end + 1;
    let end = super::lex::function_end(content.as_bytes(), identifier_end, limit)?;
    let arguments = normalize_comments(&content[arguments_start..end - 1])?;
    let (hue, saturation, lightness, alpha) = components(&arguments)?;
    let [red, green, blue] = to_rgb(hue, saturation, lightness);
    Some(ColorLiteral {
        start,
        end,
        red,
        green,
        blue,
        alpha,
    })
}

fn components(arguments: &str) -> Option<(f32, f32, f32, f32)> {
    if arguments.contains(',') {
        if arguments.contains('/') {
            return None;
        }
        let mut parts = arguments.split(',').map(super::rgb::trim_css_whitespace);
        let hue = hue(parts.next()?)?;
        let saturation = percentage(parts.next()?)?;
        let lightness = percentage(parts.next()?)?;
        let alpha = parts
            .next()
            .map_or(Some(1.0), |part| super::rgb::channel(part, 1.0))?;
        if parts.next().is_some() {
            return None;
        }
        return Some((hue, saturation, lightness, alpha));
    }

    let (channels, alpha) = match arguments.split_once('/') {
        Some((channels, alpha)) if !alpha.contains('/') => (channels, Some(alpha.trim())),
        Some(_) => return None,
        None => (arguments, None),
    };
    let mut parts = channels
        .split(is_css_whitespace)
        .filter(|part| !part.is_empty());
    let hue = modern_hue(parts.next()?)?;
    let saturation = modern_saturation(parts.next()?)?;
    let lightness = modern_lightness(parts.next()?)?;
    if parts.next().is_some() {
        return None;
    }
    let alpha = alpha.map_or(Some(1.0), modern_alpha)?;
    Some((hue, saturation, lightness, alpha))
}

fn hue(part: &str) -> Option<f32> {
    let part = super::rgb::trim_css_whitespace(part);
    let (number, factor) = [
        ("turn", 360.0),
        ("grad", 0.9),
        ("rad", 180.0 / std::f32::consts::PI),
        ("deg", 1.0),
    ]
    .into_iter()
    .find_map(|(unit, factor)| {
        part.get(part.len().checked_sub(unit.len())?..)
            .is_some_and(|suffix| suffix.eq_ignore_ascii_case(unit))
            .then(|| (&part[..part.len() - unit.len()], factor))
    })
    .unwrap_or((part, 1.0));
    let degrees = super::rgb::parse_css_number(number)? * factor;
    degrees.is_finite().then(|| degrees.rem_euclid(360.0))
}

fn modern_hue(part: &str) -> Option<f32> {
    if super::rgb::trim_css_whitespace(part).eq_ignore_ascii_case("none") {
        return Some(0.0);
    }
    hue(part)
}

fn percentage(part: &str) -> Option<f32> {
    let value =
        super::rgb::parse_css_number(super::rgb::trim_css_whitespace(part).strip_suffix('%')?)?
            / 100.0;
    value.is_finite().then(|| value.clamp(0.0, 1.0))
}

fn modern_component(part: &str) -> Option<f32> {
    let part = super::rgb::trim_css_whitespace(part);
    if part.eq_ignore_ascii_case("none") {
        return Some(0.0);
    }
    let number = part.strip_suffix('%').unwrap_or(part);
    let value = super::rgb::parse_css_number(number)? / 100.0;
    value.is_finite().then_some(value)
}

fn modern_saturation(part: &str) -> Option<f32> {
    modern_component(part).map(|value| value.max(0.0))
}

fn modern_lightness(part: &str) -> Option<f32> {
    modern_component(part)
}

fn modern_alpha(part: &str) -> Option<f32> {
    if super::rgb::trim_css_whitespace(part).eq_ignore_ascii_case("none") {
        return Some(0.0);
    }
    super::rgb::channel(part, 1.0)
}

fn is_css_whitespace(character: char) -> bool {
    matches!(character, ' ' | '\t' | '\n' | '\r' | '\u{000c}')
}

fn normalize_comments(arguments: &str) -> Option<String> {
    let mut normalized = String::with_capacity(arguments.len());
    let mut cursor = 0;
    while cursor < arguments.len() {
        if arguments[cursor..].starts_with("/*") {
            let close = arguments[cursor + 2..].find("*/")? + cursor + 2;
            normalized.push(' ');
            cursor = close + 2;
            continue;
        }
        let character = arguments[cursor..].chars().next()?;
        normalized.push(character);
        cursor += character.len_utf8();
    }
    Some(normalized)
}

fn to_rgb(hue: f32, saturation: f32, lightness: f32) -> [f32; 3] {
    let hue = f64::from(hue);
    let saturation = f64::from(saturation);
    let lightness = f64::from(lightness);
    let chroma = (1.0 - (2.0 * lightness - 1.0).abs()) * saturation;
    let sector = hue / 60.0;
    let secondary = chroma * (1.0 - (sector.rem_euclid(2.0) - 1.0).abs());
    let [red, green, blue] = match sector as u8 {
        0 => [chroma, secondary, 0.0],
        1 => [secondary, chroma, 0.0],
        2 => [0.0, chroma, secondary],
        3 => [0.0, secondary, chroma],
        4 => [secondary, 0.0, chroma],
        _ => [chroma, 0.0, secondary],
    };
    let match_value = lightness - chroma / 2.0;
    [red + match_value, green + match_value, blue + match_value]
        .map(|channel| channel.clamp(0.0, 1.0) as f32)
}

#[cfg(test)]
mod tests {
    use super::{components, normalize_comments, to_rgb};

    #[test]
    fn accepts_modern_legacy_and_hue_units() {
        assert_eq!(components("0 100% 50%"), Some((0.0, 1.0, 0.5, 1.0)));
        assert_eq!(components("0 100 50"), Some((0.0, 1.0, 0.5, 1.0)));
        assert_eq!(components("0 1 0.5"), Some((0.0, 0.01, 0.005, 1.0)));
        assert_eq!(components("30 300% 75%"), Some((30.0, 3.0, 0.75, 1.0)));
        assert_eq!(components("30 -100% -25%"), Some((30.0, 0.0, -0.25, 1.0)));
        assert_eq!(
            components("none none none / none"),
            Some((0.0, 0.0, 0.0, 0.0))
        );
        assert_eq!(
            normalize_comments("0/**/100%/**/50%")
                .as_deref()
                .and_then(components),
            Some((0.0, 1.0, 0.5, 1.0))
        );
        assert_eq!(
            components("120, 100%, 25%, 50%"),
            Some((120.0, 1.0, 0.25, 0.5))
        );
        assert_eq!(
            components("0.5turn 100% 50% / .25"),
            Some((180.0, 1.0, 0.5, 0.25))
        );
        assert_eq!(components("200grad 100% 50%"), Some((180.0, 1.0, 0.5, 1.0)));
        assert_eq!(
            components("3.1415927rad 100% 50%"),
            Some((180.0, 1.0, 0.5, 1.0))
        );
    }

    #[test]
    fn rejects_mixed_or_malformed_component_syntax() {
        assert_eq!(components("0, 100%, 50% / .5"), None);
        assert_eq!(components("0, 100, 50"), None);
        assert_eq!(components("none, 100%, 50%"), None);
        assert_eq!(components("0\u{000b}100%\u{000b}50%"), None);
        assert_eq!(components("0,\u{00a0}100%,\u{00a0}50%"), None);
        assert_eq!(components("0 100% 50% .5"), None);
        assert_eq!(components("0 100% 50% // .5"), None);
        assert_eq!(components("0 100% 50% extra"), None);
        assert_eq!(components("NaN 100% 50%"), None);
    }

    #[test]
    fn converts_each_hue_sector_and_achromatic_values() {
        assert_eq!(to_rgb(0.0, 1.0, 0.5), [1.0, 0.0, 0.0]);
        assert_eq!(to_rgb(120.0, 1.0, 0.5), [0.0, 1.0, 0.0]);
        assert_eq!(to_rgb(240.0, 1.0, 0.5), [0.0, 0.0, 1.0]);
        assert_eq!(to_rgb(30.0, 0.0, 0.25), [0.25, 0.25, 0.25]);
        assert_eq!(to_rgb(30.0, 3.0, 0.75), [1.0, 0.75, 0.0]);
        assert!(
            components("30 3e38 3e38")
                .map(|(hue, saturation, lightness, _)| to_rgb(hue, saturation, lightness))
                .is_some_and(|channels| channels
                    .into_iter()
                    .all(|channel| channel.is_finite() && (0.0..=1.0).contains(&channel)))
        );
    }
}
