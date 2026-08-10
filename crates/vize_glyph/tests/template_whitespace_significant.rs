use vize_glyph::{FormatOptions, format_sfc, format_template};

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

    // PascalCase resolves as a Vue component, not as the native textarea.
    // It therefore follows ordinary component formatting instead of the raw
    // HTML path.
    let mixed_case = "<Textarea>\n    </Textarea>";
    assert_eq!(
        format_template(mixed_case, &options).unwrap().as_str(),
        "<Textarea></Textarea>"
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
fn pascal_case_raw_names_remain_components_in_template_and_sfc_paths() {
    let options = FormatOptions::default();
    for tag in ["Pre", "Textarea", "Listing"] {
        let source = format!(r#"<{tag}><span class="value" id="child">value</span></{tag}>"#);
        let expected = format!(r#"<{tag}><span id="child" class="value">value</span></{tag}>"#);
        assert_eq!(
            format_template(&source, &options).unwrap(),
            expected,
            "{tag}"
        );

        let sfc = format!(
            "<template>\n  <{tag}>\n<span class=\"value\" id=\"child\">value</span>\n</{tag}>\n</template>\n"
        );
        let expected_sfc = format!(
            "<template>\n  <{tag}>\n    <span id=\"child\" class=\"value\">value</span>\n  </{tag}>\n</template>\n"
        );
        let formatted = format_sfc(&sfc, &options).unwrap();
        assert_eq!(formatted.code, expected_sfc, "{tag}");
        assert_eq!(
            format_sfc(&formatted.code, &options).unwrap().code,
            expected_sfc
        );
    }
}

#[test]
fn listing_content_is_preserved_byte_for_byte() {
    let options = FormatOptions::default();
    let source = "<listing>\r\n\t a\r\n b</listing>";
    let result = format_template(source, &options).unwrap();
    assert_eq!(result.as_str(), source);
    assert_eq!(format_template(result.as_str(), &options).unwrap(), result);

    let sfc = "<template>\n  <listing>\r\n\t a\r\n b</listing>\n</template>\n";
    let formatted = format_sfc(sfc, &options).unwrap();
    assert_eq!(formatted.code.as_str(), sfc);
    assert_eq!(
        format_sfc(&formatted.code, &options).unwrap().code,
        formatted.code
    );
}

#[test]
fn self_closing_raw_tags_do_not_capture_following_sfc_lines() {
    let options = FormatOptions::default();
    for tag in ["pre", "textarea", "listing"] {
        let source =
            format!("<template>\n  <{tag} />\n  <div>\n    value\n  </div>\n</template>\n");
        let formatted = format_sfc(&source, &options).unwrap();
        assert_eq!(formatted.code, source, "{tag}");
        assert_eq!(format_sfc(&formatted.code, &options).unwrap().code, source);
    }
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
