//! P2-11 patch-flag witness: the S2 DOM lane must compute the
//! same per-node patch flags as the shipped lane, including dynamic
//! prop arrays plus slot and fragment stability markers.

#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_types,
    clippy::disallowed_methods
)]

mod support;

use vize_carton::Allocator;
use vize_s1_to_s2::emit_dom_source;

struct Case {
    name: &'static str,
    src: &'static str,
    sites: &'static [&'static str],
}

const CASES: &[Case] = &[
    Case {
        name: "text_child",
        src: "<div>{{ msg }}</div>",
        sites: &["1 /* TEXT */"],
    },
    Case {
        name: "class",
        src: r#"<div :class="cls"></div>"#,
        sites: &["2 /* CLASS */"],
    },
    Case {
        name: "style",
        src: r#"<div :style="style"></div>"#,
        sites: &["4 /* STYLE */"],
    },
    Case {
        name: "prop",
        src: r#"<div :id="id"></div>"#,
        sites: &["8 /* PROPS */, [\"id\"]"],
    },
    Case {
        name: "full_props",
        src: r#"<div :[key]="value"></div>"#,
        sites: &["16 /* FULL_PROPS */"],
    },
    Case {
        name: "full_props_need_hydration",
        src: r#"<div :[key].prop="value"></div>"#,
        sites: &["48 /* FULL_PROPS, NEED_HYDRATION */"],
    },
    Case {
        name: "prop_modifier_need_hydration",
        src: r#"<div :value.prop="value"></div>"#,
        sites: &["40 /* PROPS, NEED_HYDRATION */, [\".value\"]"],
    },
    Case {
        name: "click_event_props",
        src: r#"<div @click="handler"></div>"#,
        sites: &["8 /* PROPS */, [\"onClick\"]"],
    },
    Case {
        name: "hydrating_key_event",
        src: r#"<div @keyup.enter="handler"></div>"#,
        sites: &["40 /* PROPS, NEED_HYDRATION */, [\"onKeyup\"]"],
    },
    Case {
        name: "hydrating_key_event_plain",
        src: r#"<div @keyup="handler"></div>"#,
        sites: &["40 /* PROPS, NEED_HYDRATION */, [\"onKeyup\"]"],
    },
    Case {
        name: "need_patch",
        src: r#"<div ref="el"></div>"#,
        sites: &["512 /* NEED_PATCH */"],
    },
    Case {
        name: "directive_need_patch",
        src: r#"<div v-example></div>"#,
        sites: &["512 /* NEED_PATCH */"],
    },
    Case {
        name: "directive_text_need_patch",
        src: r#"<div v-example>{{ msg }}</div>"#,
        sites: &["1 /* TEXT */", "513 /* TEXT, NEED_PATCH */"],
    },
    Case {
        name: "dynamic_slots",
        src: r#"<Foo><template #header v-if="ok">x</template></Foo>"#,
        sites: &["2 /* DYNAMIC */", "1024 /* DYNAMIC_SLOTS */"],
    },
    Case {
        name: "dynamic_slots_builtin",
        src: "<KeepAlive><Foo /></KeepAlive>",
        sites: &["1024 /* DYNAMIC_SLOTS */"],
    },
    Case {
        name: "dynamic_slot_name",
        src: r#"<Foo><template #[name]>x</template></Foo>"#,
        sites: &["2 /* DYNAMIC */", "1024 /* DYNAMIC_SLOTS */"],
    },
    Case {
        name: "text_and_class",
        src: r#"<div :class="cls">{{ msg }}</div>"#,
        sites: &["3 /* TEXT, CLASS */"],
    },
    Case {
        name: "stable_fragment",
        src: "<div></div><span></span>",
        sites: &["64 /* STABLE_FRAGMENT */"],
    },
    Case {
        name: "keyed_fragment",
        src: r#"<div v-for="item in list" :key="item.id">{{ item.label }}</div>"#,
        sites: &["1 /* TEXT */", "128 /* KEYED_FRAGMENT */"],
    },
    Case {
        name: "unkeyed_fragment",
        src: r#"<div v-for="item in list">{{ item.label }}</div>"#,
        sites: &["1 /* TEXT */", "256 /* UNKEYED_FRAGMENT */"],
    },
];

#[test]
fn s2_patch_flags_match_the_shipped_dom_lane_per_node() {
    let battery: Vec<_> = CASES.iter().map(|case| (case.name, case.src)).collect();
    support::assert_s2_matches_shipped(&battery);

    let mut mismatches = Vec::new();
    for case in CASES {
        let expected: Vec<_> = case.sites.iter().map(|site| site.to_string()).collect();
        let old = support::shipped(case.src);
        let allocator = Allocator::new();
        let new = emit_dom_source(&allocator, case.src)
            .unwrap_or_else(|error| panic!("{}: S2 emit refused: {error:?}", case.name))
            .assembled();
        let old_sites = patch_sites(&old);
        let new_sites = patch_sites(&new);

        if old_sites != expected || new_sites != expected {
            mismatches.push(format!(
                "{}: expected={expected:?} old={old_sites:?} new={new_sites:?}",
                case.name
            ));
        }
    }
    assert!(
        mismatches.is_empty(),
        "patch flag mismatches:\n{}",
        mismatches.join("\n")
    );
}

fn patch_sites(source: &str) -> Vec<String> {
    let bytes = source.as_bytes();
    let mut sites = Vec::new();
    let mut cursor = 0usize;
    while let Some(comment_rel) = source[cursor..].find(" /* ") {
        let comment_start = cursor + comment_rel;
        let Some(comment_end_rel) = source[comment_start..].find(" */") else {
            break;
        };
        let comment_end = comment_start + comment_end_rel + " */".len();
        let Some(number_start) = flag_number_start(bytes, comment_start) else {
            cursor = comment_end;
            continue;
        };
        let mut site_end = comment_end;
        if let Some(array_end) = dynamic_props_array_end(source, comment_end) {
            site_end = array_end;
        }
        sites.push(source[number_start..site_end].trim().to_string());
        cursor = site_end;
    }
    sites
}

fn flag_number_start(bytes: &[u8], comment_start: usize) -> Option<usize> {
    let mut index = comment_start;
    while index > 0 && bytes[index - 1].is_ascii_whitespace() {
        index -= 1;
    }
    if index == 0 || !bytes[index - 1].is_ascii_digit() {
        return None;
    }
    while index > 0 && bytes[index - 1].is_ascii_digit() {
        index -= 1;
    }
    if index > 0 && bytes[index - 1] == b'-' {
        index -= 1;
    }
    Some(index)
}

fn dynamic_props_array_end(source: &str, comment_end: usize) -> Option<usize> {
    let tail = &source[comment_end..];
    let trimmed = tail.trim_start();
    if !trimmed.starts_with(", [") {
        return None;
    }
    let offset = tail.len() - trimmed.len();
    let array_start = comment_end + offset + ", ".len();
    let array_tail = &source[array_start..];
    Some(array_start + array_tail.find(']')? + 1)
}
