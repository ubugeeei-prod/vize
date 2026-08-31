//! The P0-2 fixture ladder.
//!
//! Six committed `.vue` fixtures spanning a size ladder (small / medium /
//! large) and three stress axes (nesting depth, attribute width,
//! interpolation count). Provenance for every file - including the exact
//! corpus commit for the real-project extracts - lives in
//! `fixtures/PROVENANCE.md`. Benches embed the sources with `include_str!`,
//! so a fixture edit is a bench-identity change and needs new baselines;
//! the exact-length tests below make such an edit loud.

/// One ladder fixture.
#[derive(Debug, Clone, Copy)]
pub struct Fixture {
    /// Ladder name, used in bench ids (`armature_parse_stress-deep`).
    pub name: &'static str,
    /// Workspace-relative path, recorded as the report `fixture` field.
    pub relative_path: &'static str,
    /// Full `.vue` source.
    pub source: &'static str,
}

/// The committed ladder, ordered small to large, then stress fixtures.
pub const LADDER: [Fixture; 6] = [
    Fixture {
        name: "small",
        relative_path: "tools/benchmarks/crates/davinci_harness/fixtures/small.vue",
        source: include_str!("../fixtures/small.vue"),
    },
    Fixture {
        name: "medium",
        relative_path: "tools/benchmarks/crates/davinci_harness/fixtures/medium.vue",
        source: include_str!("../fixtures/medium.vue"),
    },
    Fixture {
        name: "large",
        relative_path: "tools/benchmarks/crates/davinci_harness/fixtures/large.vue",
        source: include_str!("../fixtures/large.vue"),
    },
    Fixture {
        name: "stress-deep",
        relative_path: "tools/benchmarks/crates/davinci_harness/fixtures/stress-deep.vue",
        source: include_str!("../fixtures/stress-deep.vue"),
    },
    Fixture {
        name: "stress-wide",
        relative_path: "tools/benchmarks/crates/davinci_harness/fixtures/stress-wide.vue",
        source: include_str!("../fixtures/stress-wide.vue"),
    },
    Fixture {
        name: "stress-interp",
        relative_path: "tools/benchmarks/crates/davinci_harness/fixtures/stress-interp.vue",
        source: include_str!("../fixtures/stress-interp.vue"),
    },
];

/// Extract the inner source of the root `<template>` block.
///
/// Root SFC blocks open and close at column zero in every ladder fixture,
/// while nested `<template #slot>` usages are indented, so line-anchored
/// scanning is exact here: the block opens at the first line starting with
/// `<template` and closes at the next line starting with `</template>`. The
/// returned slice matches SFC block-content semantics - it starts right after
/// the opening tag's `>` (so with the leading newline) and ends after the
/// newline preceding the closing tag.
///
/// This is a fixture-loader convenience, not a general SFC parser; benches
/// that need real block splitting use `vize_atelier_sfc::parse_sfc`.
pub fn template_block(source: &str) -> Option<&str> {
    let open_line_start = if source.starts_with("<template") {
        0
    } else {
        source.find("\n<template").map(|index| index + 1)?
    };
    let open_end = source[open_line_start..]
        .find('>')
        .map(|index| open_line_start + index + 1)?;
    let close_newline = source[open_end..]
        .find("\n</template>")
        .map(|index| open_end + index)?;
    Some(&source[open_end..close_newline + 1])
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Fixture identity pin: source and extracted-template lengths are exact.
    /// A fixture edit must change these numbers - that is the point; update
    /// them together with new bench baselines.
    #[test]
    fn ladder_identity_is_pinned() {
        let expected: [(&str, usize, usize); 6] = [
            ("small", 710, 191),
            ("medium", 4259, 2238),
            ("large", 30828, 10271),
            ("stress-deep", 10378, 10356),
            ("stress-wide", 5702, 5680),
            ("stress-interp", 8140, 8118),
        ];
        assert_eq!(LADDER.len(), expected.len());
        for (fixture, (name, source_len, template_len)) in LADDER.iter().zip(expected) {
            assert_eq!(fixture.name, name);
            assert_eq!(fixture.source.len(), source_len);
            let template = template_block(fixture.source)
                .expect("every ladder fixture has a root template block");
            assert_eq!(template.len(), template_len);
        }
    }

    #[test]
    fn template_block_slices_exactly() {
        let block = template_block(LADDER[0].source).expect("small.vue has a template block");
        let expected = concat!(
            "\n",
            "  <button class=\"counter\" type=\"button\" @click=\"increment\">\n",
            "    <span class=\"counter-label\">{{ label }}</span>\n",
            "    <span class=\"counter-value\">{{ count }} ({{ doubled }})</span>\n",
            "  </button>\n",
        );
        assert_eq!(block, expected);
    }

    #[test]
    fn template_block_rejects_sources_without_root_template() {
        assert_eq!(template_block("<script>const x = 1;</script>\n"), None);
        assert_eq!(template_block(""), None);
    }
}
