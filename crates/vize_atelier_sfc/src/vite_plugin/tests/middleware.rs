use super::super::{normalize_css_module_filename, normalize_dev_middleware_url};

#[test]
fn normalizes_virtual_css_module_filename() {
    assert_eq!(
        normalize_css_module_filename("/project/\0/src/Card.vue?vue&type=style&module.scss")
            .as_str(),
        "/src/Card.vue"
    );
    assert_eq!(
        normalize_css_module_filename("/src/Card.vue?vue&type=style&lang.css").as_str(),
        "/src/Card.vue"
    );
}

#[test]
fn normalizes_dev_middleware_x00_urls() {
    let rewrite =
        normalize_dev_middleware_url("/@id/__x00__/Users/app/src/icon.svg?import").unwrap();

    assert_eq!(
        rewrite.cleaned_url.as_str(),
        "/@fs/Users/app/src/icon.svg?import"
    );
    assert_eq!(rewrite.fs_path.as_str(), "/Users/app/src/icon.svg");
    assert!(normalize_dev_middleware_url("/@id/__x00__/Users/app/src/App.vue.ts").is_none());
}
