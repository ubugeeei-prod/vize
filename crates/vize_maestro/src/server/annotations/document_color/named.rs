//! CSS named colours from CSS Color Module Level 4, section 6.1.
//!
//! Source: <https://www.w3.org/TR/css-color-4/#named-colors>. The 148 opaque
//! entries are kept in the specification's ASCII order; `transparent` is the
//! transparent black defined by section 6.3.

/// Resolve an ASCII case-insensitive CSS colour keyword to RGBA bytes.
#[cfg(test)]
pub(super) fn rgba(name: &str) -> Option<[u8; 4]> {
    rgba_bytes(name.as_bytes())
}

pub(super) fn rgba_bytes(name: &[u8]) -> Option<[u8; 4]> {
    let index = CSS_NAMED_COLORS
        .binary_search_by(|(candidate, _)| {
            candidate
                .bytes()
                .cmp(name.iter().map(|byte| byte.to_ascii_lowercase()))
        })
        .ok()?;
    Some(CSS_NAMED_COLORS[index].1.to_be_bytes())
}

/// Packed as `0xRRGGBBAA` so every table row stays visually auditable against
/// the hexadecimal value in the specification.
const CSS_NAMED_COLORS: [(&str, u32); 149] = [
    ("aliceblue", 0xf0f8ffff),
    ("antiquewhite", 0xfaebd7ff),
    ("aqua", 0x00ffffff),
    ("aquamarine", 0x7fffd4ff),
    ("azure", 0xf0ffffff),
    ("beige", 0xf5f5dcff),
    ("bisque", 0xffe4c4ff),
    ("black", 0x000000ff),
    ("blanchedalmond", 0xffebcdff),
    ("blue", 0x0000ffff),
    ("blueviolet", 0x8a2be2ff),
    ("brown", 0xa52a2aff),
    ("burlywood", 0xdeb887ff),
    ("cadetblue", 0x5f9ea0ff),
    ("chartreuse", 0x7fff00ff),
    ("chocolate", 0xd2691eff),
    ("coral", 0xff7f50ff),
    ("cornflowerblue", 0x6495edff),
    ("cornsilk", 0xfff8dcff),
    ("crimson", 0xdc143cff),
    ("cyan", 0x00ffffff),
    ("darkblue", 0x00008bff),
    ("darkcyan", 0x008b8bff),
    ("darkgoldenrod", 0xb8860bff),
    ("darkgray", 0xa9a9a9ff),
    ("darkgreen", 0x006400ff),
    ("darkgrey", 0xa9a9a9ff),
    ("darkkhaki", 0xbdb76bff),
    ("darkmagenta", 0x8b008bff),
    ("darkolivegreen", 0x556b2fff),
    ("darkorange", 0xff8c00ff),
    ("darkorchid", 0x9932ccff),
    ("darkred", 0x8b0000ff),
    ("darksalmon", 0xe9967aff),
    ("darkseagreen", 0x8fbc8fff),
    ("darkslateblue", 0x483d8bff),
    ("darkslategray", 0x2f4f4fff),
    ("darkslategrey", 0x2f4f4fff),
    ("darkturquoise", 0x00ced1ff),
    ("darkviolet", 0x9400d3ff),
    ("deeppink", 0xff1493ff),
    ("deepskyblue", 0x00bfffff),
    ("dimgray", 0x696969ff),
    ("dimgrey", 0x696969ff),
    ("dodgerblue", 0x1e90ffff),
    ("firebrick", 0xb22222ff),
    ("floralwhite", 0xfffaf0ff),
    ("forestgreen", 0x228b22ff),
    ("fuchsia", 0xff00ffff),
    ("gainsboro", 0xdcdcdcff),
    ("ghostwhite", 0xf8f8ffff),
    ("gold", 0xffd700ff),
    ("goldenrod", 0xdaa520ff),
    ("gray", 0x808080ff),
    ("green", 0x008000ff),
    ("greenyellow", 0xadff2fff),
    ("grey", 0x808080ff),
    ("honeydew", 0xf0fff0ff),
    ("hotpink", 0xff69b4ff),
    ("indianred", 0xcd5c5cff),
    ("indigo", 0x4b0082ff),
    ("ivory", 0xfffff0ff),
    ("khaki", 0xf0e68cff),
    ("lavender", 0xe6e6faff),
    ("lavenderblush", 0xfff0f5ff),
    ("lawngreen", 0x7cfc00ff),
    ("lemonchiffon", 0xfffacdff),
    ("lightblue", 0xadd8e6ff),
    ("lightcoral", 0xf08080ff),
    ("lightcyan", 0xe0ffffff),
    ("lightgoldenrodyellow", 0xfafad2ff),
    ("lightgray", 0xd3d3d3ff),
    ("lightgreen", 0x90ee90ff),
    ("lightgrey", 0xd3d3d3ff),
    ("lightpink", 0xffb6c1ff),
    ("lightsalmon", 0xffa07aff),
    ("lightseagreen", 0x20b2aaff),
    ("lightskyblue", 0x87cefaff),
    ("lightslategray", 0x778899ff),
    ("lightslategrey", 0x778899ff),
    ("lightsteelblue", 0xb0c4deff),
    ("lightyellow", 0xffffe0ff),
    ("lime", 0x00ff00ff),
    ("limegreen", 0x32cd32ff),
    ("linen", 0xfaf0e6ff),
    ("magenta", 0xff00ffff),
    ("maroon", 0x800000ff),
    ("mediumaquamarine", 0x66cdaaff),
    ("mediumblue", 0x0000cdff),
    ("mediumorchid", 0xba55d3ff),
    ("mediumpurple", 0x9370dbff),
    ("mediumseagreen", 0x3cb371ff),
    ("mediumslateblue", 0x7b68eeff),
    ("mediumspringgreen", 0x00fa9aff),
    ("mediumturquoise", 0x48d1ccff),
    ("mediumvioletred", 0xc71585ff),
    ("midnightblue", 0x191970ff),
    ("mintcream", 0xf5fffaff),
    ("mistyrose", 0xffe4e1ff),
    ("moccasin", 0xffe4b5ff),
    ("navajowhite", 0xffdeadff),
    ("navy", 0x000080ff),
    ("oldlace", 0xfdf5e6ff),
    ("olive", 0x808000ff),
    ("olivedrab", 0x6b8e23ff),
    ("orange", 0xffa500ff),
    ("orangered", 0xff4500ff),
    ("orchid", 0xda70d6ff),
    ("palegoldenrod", 0xeee8aaff),
    ("palegreen", 0x98fb98ff),
    ("paleturquoise", 0xafeeeeff),
    ("palevioletred", 0xdb7093ff),
    ("papayawhip", 0xffefd5ff),
    ("peachpuff", 0xffdab9ff),
    ("peru", 0xcd853fff),
    ("pink", 0xffc0cbff),
    ("plum", 0xdda0ddff),
    ("powderblue", 0xb0e0e6ff),
    ("purple", 0x800080ff),
    ("rebeccapurple", 0x663399ff),
    ("red", 0xff0000ff),
    ("rosybrown", 0xbc8f8fff),
    ("royalblue", 0x4169e1ff),
    ("saddlebrown", 0x8b4513ff),
    ("salmon", 0xfa8072ff),
    ("sandybrown", 0xf4a460ff),
    ("seagreen", 0x2e8b57ff),
    ("seashell", 0xfff5eeff),
    ("sienna", 0xa0522dff),
    ("silver", 0xc0c0c0ff),
    ("skyblue", 0x87ceebff),
    ("slateblue", 0x6a5acdff),
    ("slategray", 0x708090ff),
    ("slategrey", 0x708090ff),
    ("snow", 0xfffafaff),
    ("springgreen", 0x00ff7fff),
    ("steelblue", 0x4682b4ff),
    ("tan", 0xd2b48cff),
    ("teal", 0x008080ff),
    ("thistle", 0xd8bfd8ff),
    ("tomato", 0xff6347ff),
    ("transparent", 0x00000000),
    ("turquoise", 0x40e0d0ff),
    ("violet", 0xee82eeff),
    ("wheat", 0xf5deb3ff),
    ("white", 0xffffffff),
    ("whitesmoke", 0xf5f5f5ff),
    ("yellow", 0xffff00ff),
    ("yellowgreen", 0x9acd32ff),
];

#[cfg(test)]
mod tests {
    use std::fmt::Write;

    use sha2::{Digest, Sha256};

    use super::{CSS_NAMED_COLORS, rgba};

    #[test]
    fn table_is_complete_sorted_and_ascii_case_insensitive() {
        assert_eq!(CSS_NAMED_COLORS.len(), 149);
        assert!(
            CSS_NAMED_COLORS
                .windows(2)
                .all(|pair| pair[0].0 < pair[1].0)
        );
        for &(name, expected) in &CSS_NAMED_COLORS {
            assert_eq!(rgba(name), Some(expected.to_be_bytes()), "{name}");
            assert_eq!(
                rgba(&name.to_ascii_uppercase()),
                Some(expected.to_be_bytes()),
                "{name}"
            );
        }
        assert_eq!(rgba("transparent"), Some([0, 0, 0, 0]));
    }

    #[test]
    fn every_gray_grey_alias_has_the_same_value() {
        for (gray, grey) in [
            ("gray", "grey"),
            ("darkgray", "darkgrey"),
            ("darkslategray", "darkslategrey"),
            ("dimgray", "dimgrey"),
            ("lightgray", "lightgrey"),
            ("lightslategray", "lightslategrey"),
            ("slategray", "slategrey"),
        ] {
            assert_eq!(rgba(gray), rgba(grey), "{gray}/{grey}");
        }
    }

    #[test]
    fn table_matches_the_pinned_css_color_4_oracle() {
        let mut serialization = String::new();
        for (name, packed) in CSS_NAMED_COLORS {
            writeln!(serialization, "{name}=#{packed:08x}").unwrap();
        }

        // SHA-256 of the 149 sorted W3C rows serialized as
        // `name=#rrggbbaa\n`; transparent is the section 6.3 transparent black.
        // Expected: 8c7caee2456a1cab1e378b1009475d38bd04be7b95726da9f976f140d3fb3b93.
        let digest: [u8; 32] = Sha256::digest(serialization).into();
        assert_eq!(
            digest,
            [
                0x8c, 0x7c, 0xae, 0xe2, 0x45, 0x6a, 0x1c, 0xab, 0x1e, 0x37, 0x8b, 0x10, 0x09, 0x47,
                0x5d, 0x38, 0xbd, 0x04, 0xbe, 0x7b, 0x95, 0x72, 0x6d, 0xa9, 0xf9, 0x76, 0xf1, 0x40,
                0xd3, 0xfb, 0x3b, 0x93,
            ]
        );
        for (name, expected) in [
            ("aliceblue", [0xf0, 0xf8, 0xff, 0xff]),
            ("aqua", [0x00, 0xff, 0xff, 0xff]),
            ("darkslategray", [0x2f, 0x4f, 0x4f, 0xff]),
            ("green", [0x00, 0x80, 0x00, 0xff]),
            ("rebeccapurple", [0x66, 0x33, 0x99, 0xff]),
            ("transparent", [0x00, 0x00, 0x00, 0x00]),
            ("yellowgreen", [0x9a, 0xcd, 0x32, 0xff]),
        ] {
            assert_eq!(rgba(name), Some(expected), "{name}");
        }
    }
}
