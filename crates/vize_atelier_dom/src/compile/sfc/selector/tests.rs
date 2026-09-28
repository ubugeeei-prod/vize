use super::l2_sfc_fast_path_supported_source;

#[test]
fn html_void_element_does_not_keep_parent_open_after_close() {
    for source in [
        r#"<a><img src="x"></a><a>next</a>"#,
        r#"<a><IMG src="x"></a><a>next</a>"#,
    ] {
        assert!(
            l2_sfc_fast_path_supported_source(source),
            "{source} should keep the direct L2 SFC fast path"
        );
    }
}

#[test]
fn stray_end_tag_requires_parser_diagnostics() {
    assert!(!l2_sfc_fast_path_supported_source("<div></div></div>"));
    assert!(l2_sfc_fast_path_supported_source("<div></div>"));
}

#[test]
fn duplicate_attributes_require_parser_warnings() {
    for source in [
        r#"<h4 :class="premium" class="" class="shop_title">Shop</h4>"#,
        r#"<div CLASS="first" class="second" />"#,
    ] {
        assert!(!l2_sfc_fast_path_supported_source(source), "{source}");
    }
    for source in [
        r#"<div class="first" title="class='second'">Shop</div>"#,
        r#"<div :class="first" class="second">Shop</div>"#,
    ] {
        assert!(l2_sfc_fast_path_supported_source(source), "{source}");
    }
}

#[test]
fn expression_entities_require_the_shared_parser() {
    for source in [
        r#"<span>{{ '&lt;' }}</span>"#,
        r#"<span :title="'&lt;'">x</span>"#,
        r#"<span v-bind:title="'&amp;'">x</span>"#,
    ] {
        assert!(!l2_sfc_fast_path_supported_source(source), "{source}");
    }
    assert!(l2_sfc_fast_path_supported_source("<span>&amp;</span>"));
}
