use super::vapor_surface::{MARKUP_ADMISSION, first_cause, message_shape};

#[test]
fn plain_templates_have_no_lowering_cause() {
    assert_eq!(first_cause("<div><p>{{ a }}</p></div>"), MARKUP_ADMISSION);
}

#[test]
fn dropped_content_names_its_provenance_rule() {
    assert_eq!(
        first_cause("<p v-if=\"a\">a</p><!-- gap --><p v-else>b</p>"),
        "provenance:drop.branch-gap"
    );
    assert_eq!(
        first_cause("<div></span></div>"),
        "provenance:drop.unexpected"
    );
}

#[test]
fn diagnostic_buckets_elide_quoted_names() {
    assert_eq!(
        message_shape("unknown directive `v-foo` on 'div' near \"x\""),
        "unknown directive `_` on '_' near \"_\""
    );
}
