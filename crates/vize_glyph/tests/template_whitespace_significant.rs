use vize_glyph::{FormatOptions, format_template};

#[test]
fn textarea_whitespace_only_content_preserved() {
    // Whitespace-only `<textarea>` content is the element's initial value
    // and must survive formatting; the immediate-empty-close collapse used
    // to drop it, changing the rendered DOM. (#3250)
    let options = FormatOptions::default();

    let result = format_template("<textarea>\n    </textarea>", &options).unwrap();
    assert_eq!(result.as_str(), "<textarea>\n    </textarea>");
    assert_eq!(
        format_template(&result, &options).unwrap(),
        result,
        "textarea preservation must be idempotent"
    );

    // Content around the whitespace is preserved verbatim too.
    let with_value = "<textarea>\n  hello\n</textarea>";
    assert_eq!(
        format_template(with_value, &options).unwrap().as_str(),
        with_value
    );

    // A `<pre>` whose only content is whitespace is preserved for the same
    // reason (whitespace is significant inside `pre`).
    let pre = "<pre>\n    </pre>";
    assert_eq!(format_template(pre, &options).unwrap().as_str(), pre);

    // Tag-name matching is case-insensitive: mixed-case forms are just as
    // whitespace-significant and must not be collapsed. (#3250)
    let mixed_case = "<TeXtArEa>\n    </TeXtArEa>";
    assert_eq!(
        format_template(mixed_case, &options).unwrap().as_str(),
        mixed_case
    );

    // Any element carrying `v-pre` is whitespace-significant too, so a
    // whitespace-only body must survive verbatim and stay idempotent even
    // as attributes are normalized/sorted. (#3250)
    let v_pre = "<div v-pre>\n    </div>";
    let v_pre_result = format_template(v_pre, &options).unwrap();
    assert_eq!(v_pre_result.as_str(), v_pre);
    assert_eq!(
        format_template(&v_pre_result, &options).unwrap(),
        v_pre_result,
        "v-pre preservation must be idempotent"
    );
}

#[test]
fn non_significant_empty_element_still_collapses() {
    // Regression guard: the whitespace-significant carve-out must not stop
    // ordinary empty elements from collapsing to a single line. (#3250)
    let options = FormatOptions::default();
    assert_eq!(
        format_template("<div>\n    </div>", &options)
            .unwrap()
            .as_str(),
        "<div></div>"
    );
    assert_eq!(
        format_template("<span>\n</span>", &options)
            .unwrap()
            .as_str(),
        "<span></span>"
    );
}
