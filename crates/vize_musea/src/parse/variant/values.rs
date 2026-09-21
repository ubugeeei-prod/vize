//! Decoders for the payload-carrying `<variant>` attribute values (`args`,
//! `viewport`). They read one attribute value S1 already delimited; nothing
//! here scans the file.

use crate::types::ViewportConfig;
use vize_s0::{Allocator, FxHashMap};

/// Parse args JSON string into a map with arena-allocated keys.
/// HTML entities are decoded before parsing.
pub(super) fn parse_args_json<'a>(
    allocator: &'a Allocator,
    s: &str,
) -> Result<FxHashMap<&'a str, serde_json::Value>, serde_json::Error> {
    let json_str: std::borrow::Cow<'_, str> = if s.contains('&') {
        // Decode common HTML entities - allocates only when needed
        std::borrow::Cow::Owned(
            s.replace("&quot;", "\"")
                .replace("&apos;", "'")
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&amp;", "&"),
        )
    } else {
        std::borrow::Cow::Borrowed(s)
    };

    #[allow(clippy::disallowed_types)]
    let map: FxHashMap<std::string::String, serde_json::Value> = serde_json::from_str(&json_str)?;

    Ok(map
        .into_iter()
        .map(|(k, v)| {
            let key: &'a str = allocator.alloc_str(&k);
            (key, v)
        })
        .collect())
}

/// Parse a `viewport` value: JSON (`{"width":375,"height":667}`) or the
/// `WxH` / `WxH@scale` shorthand.
pub(super) fn parse_viewport(viewport_str: &str) -> Option<ViewportConfig> {
    if viewport_str.starts_with('{') {
        let json_str = if viewport_str.contains('&') {
            std::borrow::Cow::Owned(viewport_str.replace("&quot;", "\"").replace("&apos;", "'"))
        } else {
            std::borrow::Cow::Borrowed(viewport_str)
        };

        if let Ok(config) = serde_json::from_str::<ViewportConfig>(&json_str) {
            return Some(config);
        }
    }

    let (width, rest) = viewport_str.split_once('x')?;
    let width: u32 = width.parse().ok()?;
    let (height, device_scale_factor) = match rest.split_once('@') {
        Some((height, scale)) => (height, Some(scale.parse::<f32>().ok()?)),
        None => (rest, None),
    };
    Some(ViewportConfig {
        width,
        height: height.parse().ok()?,
        device_scale_factor,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_viewport;
    use crate::types::ViewportConfig;

    #[test]
    fn viewport_forms() {
        let cases = [
            ("375x667", Some((375, 667, None))),
            ("375x667@2", Some((375, 667, Some(2.0)))),
            (r#"{"width":320,"height":480}"#, Some((320, 480, None))),
            (
                "{&quot;width&quot;:320,&quot;height&quot;:480,&quot;deviceScaleFactor&quot;:3}",
                Some((320, 480, Some(3.0))),
            ),
            ("{\"width\":1}x2", None),
            ("wide", None),
            ("375x", None),
            ("375x667@", None),
            ("375x667@2@3", None),
        ];
        for (value, expected) in cases {
            let actual = parse_viewport(value).map(
                |ViewportConfig {
                     width,
                     height,
                     device_scale_factor,
                 }| (width, height, device_scale_factor),
            );
            assert_eq!(actual, expected, "{value}");
        }
    }
}
