use super::{RegionKind, comment_at, visit_regions};

#[test]
fn comments_preserve_exact_original_regions_around_code_and_strings() {
    let source = "close() // close 😀\r\n + count /* 日本語 count */ + '/* data */'";
    let mut regions = Vec::new();
    visit_regions(source, |range, kind| {
        if !range.is_empty() {
            regions.push((source.get(range).unwrap(), kind));
        }
    });
    assert_eq!(
        regions,
        [
            ("close() ", RegionKind::Code),
            ("// close 😀", RegionKind::Comment),
            ("\r\n + count ", RegionKind::Code),
            ("/* 日本語 count */", RegionKind::Comment),
            (" + ", RegionKind::Code),
            ("'/* data */'", RegionKind::String),
        ]
    );
    for marker in ["// close", "日本語 count"] {
        assert!(comment_at(source, source.find(marker).unwrap() + 3));
    }
    assert!(!comment_at(source, source.rfind("data").unwrap()));
}

#[test]
fn nested_template_substitutions_keep_original_code_and_quasi_ownership() {
    let source = "`😀${open ? `nested ${count /* note */}` : { key: '// data' }}tail`";
    let mut regions = Vec::new();
    visit_regions(source, |range, kind| {
        if !range.is_empty() {
            regions.push((source.get(range).unwrap(), kind));
        }
    });
    assert_eq!(
        regions,
        [
            ("`😀", RegionKind::String),
            ("open ? ", RegionKind::Code),
            ("`nested ", RegionKind::String),
            ("count ", RegionKind::Code),
            ("/* note */", RegionKind::Comment),
            ("}`", RegionKind::String),
            (" : { key: ", RegionKind::Code),
            ("'// data'", RegionKind::String),
            (" }", RegionKind::Code),
            ("}tail`", RegionKind::String),
        ]
    );
}

#[test]
fn regular_expression_data_and_postfix_division_do_not_hide_real_comments() {
    for source in [
        "/[/*'`]/.test(close)",
        "value = /https?:\\/\\//.test(close)",
        "value /= 2",
    ] {
        assert!(!comment_at(source, source.find('/').unwrap()));
    }
    let source = "count++ / /* close */ divisor";
    assert!(comment_at(source, source.find("close").unwrap()));
    assert!(!comment_at(source, source.find("divisor").unwrap()));
}

#[test]
fn escaped_unicode_and_unfinished_regions_remain_bounded_original_bytes() {
    for source in [
        "'\\😀'",
        "`raw \\` // data`",
        "/* unterminated 😀",
        "// line\u{2028}count",
    ] {
        visit_regions(source, |range, _| {
            assert!(source.get(range).is_some());
        });
    }
    assert!(comment_at("/* unterminated 😀", 5));
    assert!(!comment_at("// line\u{2028}count", "// line\u{2028}".len()));
}

#[test]
fn html_encoded_quotes_shield_data_without_changing_authored_code_token_regions() {
    for source in [
        "&quot;/* data */&quot; + count /* note */",
        "&#34;// data&#x22; + count /* note */",
        "&apos;/* data */&apos; + count /* note */",
    ] {
        assert!(!super::html_comment_at(
            source,
            source.find("data").unwrap()
        ));
        assert!(super::html_comment_at(source, source.find("note").unwrap()));
    }
    let source = "'&amp;quot;/* data */' + count /* note */";
    assert!(!super::html_comment_at(
        source,
        source.find("data").unwrap()
    ));
    assert!(super::html_comment_at(source, source.find("note").unwrap()));
}
