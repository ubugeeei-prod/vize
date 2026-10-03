use super::support::{LOCALES, expected, parity, span, warning};
use vize_l0::cstr;

fn policy(tag: &str, attrs: &[(&str, &str)]) {
    for (attr, suggestion) in attrs {
        let header = cstr!("{attr}='opaque'");
        let source = cstr!("<!--🦀--><template>日本語<{tag} {header} /></template>");
        for locale in LOCALES {
            assert_eq!(
                parity(&source, locale),
                expected(vec![warning(
                    locale,
                    span(&source, &header),
                    tag,
                    attr,
                    suggestion
                )])
            );
        }
    }
}

#[test]
fn universal_align_and_bgcolor_use_exact_original_suggestions() {
    policy(
        "div",
        &[
            ("align", "CSS `text-align` or `margin: auto`"),
            ("bgcolor", "CSS `background-color`"),
        ],
    );
}

#[test]
fn border_warns_except_on_the_exact_table_tag() {
    policy("div", &[("border", "CSS `border`")]);
    for locale in LOCALES {
        assert_eq!(
            parity("<template><table border='1'></table></template>", locale),
            expected(vec![])
        );
    }
}

#[test]
fn body_background_text_and_three_link_names_retain_every_branch() {
    policy(
        "body",
        &[
            ("background", "CSS `background-image`"),
            ("text", "CSS `color`"),
            ("link", "CSS `:link`, `:visited`"),
            ("vlink", "CSS `:link`, `:visited`"),
            ("alink", "CSS `:link`, `:visited`"),
        ],
    );
}

#[test]
fn own_table_padding_and_spacing_are_admitted_without_descendant_credit() {
    policy(
        "table",
        &[
            ("cellpadding", "CSS `padding` on cells"),
            ("cellspacing", "CSS `border-spacing`"),
        ],
    );
}

#[test]
fn standalone_td_and_th_retain_each_exact_cell_policy() {
    for tag in ["td", "th"] {
        policy(
            tag,
            &[
                ("width", "CSS `width`/`height`"),
                ("height", "CSS `width`/`height`"),
                ("valign", "CSS `vertical-align`"),
                ("nowrap", "CSS `white-space: nowrap`"),
            ],
        );
    }
}

#[test]
fn img_spacing_and_br_clear_retain_complete_attribute_ranges() {
    policy(
        "img",
        &[("hspace", "CSS `margin`"), ("vspace", "CSS `margin`")],
    );
    policy("br", &[("clear", "CSS `clear`")]);
}

#[test]
fn hr_four_names_use_the_hr_suggestion_after_the_cell_guards() {
    policy(
        "hr",
        &[
            ("noshade", "CSS styling"),
            ("size", "CSS styling"),
            ("width", "CSS styling"),
            ("color", "CSS styling"),
        ],
    );
}

#[test]
fn li_ul_type_and_pre_width_retain_original_tag_branches() {
    policy("li", &[("type", "CSS `list-style-type`")]);
    policy("ul", &[("type", "CSS `list-style-type`")]);
    policy("pre", &[("width", "CSS `width`")]);
}

#[test]
fn tag_specific_names_are_clean_on_other_exact_tags() {
    let source = "<template><div background text link vlink alink cellpadding cellspacing width height valign nowrap hspace vspace clear noshade size color type /></template>";
    for locale in LOCALES {
        assert_eq!(parity(source, locale), expected(vec![]));
    }
}

#[test]
fn namespace_spellings_do_not_receive_local_name_exceptions() {
    policy("svg:table", &[("border", "CSS `border`")]);
    for source in [
        "<template><svg:table cellpadding cellspacing /></template>",
        "<template><svg:body background text link /></template>",
        "<template><svg:td width height /></template>",
    ] {
        for locale in LOCALES {
            assert_eq!(parity(source, locale), expected(vec![]));
        }
    }
}
