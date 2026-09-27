//! Public formatter bytes for branches previously covered only by helpers.
//! Provenance refers to the scoped glyph patch and its original regression tests.
#![expect(clippy::unwrap_used, reason = "fixture helpers fail by panicking")]

use vize_glyph::{FormatOptions, format_sfc, format_template};
use vize_l0::String;

fn snapshot_bytes(name: &str, output: &str) {
    insta::with_settings!({prepend_module_to_snapshot => false}, {
        insta::assert_binary_snapshot!(name, output.as_bytes().to_vec());
    });
}

fn template(name: &str, source: &str, options: &FormatOptions) -> String {
    let first = format_template(source, options).unwrap();
    snapshot_bytes(name, &first);
    assert_eq!(format_template(&first, options).unwrap(), first);
    first
}

fn sfc(name: &str, source: &str, options: &FormatOptions) -> String {
    let first = format_sfc(source, options).unwrap().code;
    snapshot_bytes(name, &first);
    assert_eq!(format_sfc(&first, options).unwrap().code, first);
    first
}

#[test]
fn legacy_bound_slots_keep_their_public_ordering_boundaries() {
    // aff98628b (#5808), 1ab272292 (#4978): bound legacy slot names were
    // classified by attribute_priority but absent from its tests. #default
    // is a different slot directive and cannot serve as their witness.
    let options = FormatOptions {
        print_width: 200,
        ..FormatOptions::default()
    };
    for (name, source, expected) in [
        (
            "history_template_static_slot.txt",
            r#"<Comp class="box" slot="header"></Comp>"#,
            r#"<Comp slot="header" class="box"></Comp>"#,
        ),
        (
            "history_template_static_slot_scope.txt",
            r#"<Comp class="box" slot-scope="scope"></Comp>"#,
            r#"<Comp slot-scope="scope" class="box"></Comp>"#,
        ),
        (
            "history_template_bound_slot.txt",
            r#"<Comp class="box" :slot="slotName" id="after"></Comp>"#,
            r#"<Comp class="box" :slot="slotName" id="after"></Comp>"#,
        ),
        (
            "history_template_longhand_bound_slot.txt",
            r#"<Comp class="box" v-bind:slot="slotName" id="after"></Comp>"#,
            r#"<Comp class="box" :slot="slotName" id="after"></Comp>"#,
        ),
        (
            "history_template_bound_slot_scope.txt",
            r#"<Comp class="box" :slot-scope="scope" id="after"></Comp>"#,
            r#"<Comp class="box" :slot-scope="scope" id="after"></Comp>"#,
        ),
        (
            "history_template_longhand_bound_slot_scope.txt",
            r#"<Comp class="box" v-bind:slot-scope="scope" id="after"></Comp>"#,
            r#"<Comp class="box" :slot-scope="scope" id="after"></Comp>"#,
        ),
    ] {
        assert_eq!(template(name, source, &options).as_str(), expected);
    }
}

#[test]
fn templated_non_script_roots_keep_the_authored_opening_tag() {
    // b1ef6b734 (#5746): script, template, style and custom writers share
    // opening_tag::content_after_opening_tag. The old public fixture covered
    // only script setup; these exercise the other writer branches and %>>.
    let options = FormatOptions::default();
    for (name, source, opening_tag, body) in [
        (
            "history_sfc_templated_template_root.txt",
            "<template<%= useHtml ? ' lang=\"html\"' : '' %>>\n<div>{{message}}</div>\n</template>\n",
            "<template<%= useHtml ? ' lang=\"html\"' : '' %>>",
            "<div>{{ message }}</div>",
        ),
        (
            "history_sfc_templated_style_root.txt",
            "<style<%= scoped ? ' scoped' : '' %>>\n.box{color:red}\n</style>\n",
            "<style<%= scoped ? ' scoped' : '' %>>",
            "color: red;",
        ),
        (
            "history_sfc_templated_custom_root.txt",
            "<i18n<%= global ? ' global' : '' %>>\n{\"hello\":\"Hello\"}\n</i18n>\n",
            "<i18n<%= global ? ' global' : '' %>>",
            "{\"hello\":\"Hello\"}",
        ),
    ] {
        let output = sfc(name, source, &options);
        assert!(output.starts_with(opening_tag));
        assert!(output.contains(body));
        assert!(!output.contains("%>>\n>"));
    }
}

#[test]
fn incomplete_raw_closing_tags_keep_the_tail_without_fabricating_a_close() {
    // 85a067737 (#3268): copy_whitespace_significant_element's no-> branch
    // was untested. Public template formatting trims final CR/LF globally;
    // it must preserve the incomplete tag and every other authored tail byte.
    let options = FormatOptions::default();
    for (name, source) in [
        (
            "history_template_incomplete_pre_close.txt",
            "<pre>body</pre\n",
        ),
        (
            "history_template_incomplete_textarea_close.txt",
            "<textarea>body</textarea\n  ",
        ),
        (
            "history_template_incomplete_v_pre_close.txt",
            "<div v-pre>body</div\n  ",
        ),
    ] {
        let output = template(name, source, &options);
        assert_eq!(output.as_str(), source.trim_end_matches(['\r', '\n']));
        assert!(!output.ends_with('>'));
    }
}

#[test]
fn unavoidable_directive_quotes_have_complete_public_output() {
    // 26e85e98d: directive_attribute_escapes_unavoidable_inner_double_quotes
    // originally asserted only two contains checks, despite being catalogued
    // as complete output. Keep those constraints and pin every output byte.
    let output = template(
        "history_template_unavoidable_inner_quotes.txt",
        r#"<div :title="prefix + &quot;a'b&quot;"></div>"#,
        &FormatOptions::default(),
    );
    assert!(output.contains(":title=\""));
    assert!(output.contains("&quot;a'b&quot;"));
}
