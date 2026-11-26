//! A blank line inside a multi-line attribute value is written empty, like
//! every other blank line in the file, instead of carrying the attribute's
//! indentation as trailing whitespace.
use vize_glyph::{EndOfLine, FormatOptions, format_sfc};

fn check_options() -> FormatOptions {
    FormatOptions {
        skip_script_stabilization: true,
        ..FormatOptions::default()
    }
}

fn assert_fixed_point(input: &str, expected: &str, options: &FormatOptions) {
    let mut current: vize_l0::String = input.into();
    for pass in 0..3 {
        let result = format_sfc(&current, options).expect("format");
        assert_eq!(result.code.as_str(), expected, "pass {pass}");
        assert_eq!(result.changed, current.as_str() != expected, "pass {pass}");
        current = result.code;
    }
}

fn assert_no_trailing_whitespace(output: &str) {
    for (index, line) in output.split('\n').enumerate() {
        let line = line.trim_end_matches('\r');
        assert_eq!(
            line.trim_end_matches([' ', '\t']),
            line,
            "line {} ends with whitespace: {output:?}",
            index + 1
        );
    }
}

#[test]
fn sfc_handler_blank_line_is_written_empty() {
    let input = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-attribute-value-blank-lines/App.vue.txt"
    );
    let expected = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-attribute-value-blank-lines/reference.expected.txt"
    );
    assert_fixed_point(input, expected, &FormatOptions::default());
    assert_no_trailing_whitespace(expected);
}

#[test]
fn check_mode_accepts_the_empty_blank_line() {
    let expected = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-attribute-value-blank-lines/reference.expected.txt"
    );
    let input = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-attribute-value-blank-lines/App.vue.txt"
    );
    assert!(!format_sfc(expected, &check_options()).unwrap().changed);
    assert!(format_sfc(input, &check_options()).unwrap().changed);
}

#[test]
fn every_embedded_expression_attribute_writes_blank_lines_empty() {
    let input = "<template>\n  <div\n    v-bind=\"{\n      a: 1,\n\n      b: 2,\n    }\"\n    :class=\"{\n      a: x,\n\n      b: y,\n    }\"\n    v-on=\"{\n      click: () => {\n        a(1);\n\n        a(2);\n      },\n    }\"\n    @input=\"\n      a(1);\n\n      a(2);\n    \"\n  />\n</template>\n";
    let first = format_sfc(input, &FormatOptions::default())
        .expect("format")
        .code;
    assert!(
        first.contains("\n\n"),
        "the blank lines are kept: {first:?}"
    );
    assert_no_trailing_whitespace(&first);
    assert_fixed_point(&first, &first, &FormatOptions::default());
    assert!(!format_sfc(&first, &check_options()).unwrap().changed);
}

#[test]
fn a_value_moved_onto_its_own_lines_writes_blank_lines_empty() {
    // A multi-line call moves below the attribute name, where its lines take
    // their final indentation from the attribute writer.
    let input = "<template>\n  <div :x=\"run(() => {\n    a(1);\n\n    a(2);\n  })\" @y=\"run(() => {\n    a(1);\n\n    a(2);\n  })\" />\n</template>\n";
    let expected = "<template>\n  <div\n    :x=\"\n      run(() => {\n        a(1);\n\n        a(2);\n      })\n    \"\n    @y=\"\n      run(() => {\n        a(1);\n\n        a(2);\n      })\n    \"\n  />\n</template>\n";
    assert_fixed_point(input, expected, &FormatOptions::default());
    assert!(!format_sfc(expected, &check_options()).unwrap().changed);
    let canonical = FormatOptions {
        canonical_directive_layout: true,
        ..FormatOptions::default()
    };
    assert_fixed_point(expected, expected, &canonical);
}

#[test]
fn blank_lines_stay_empty_with_tabs_and_crlf() {
    let tabs = FormatOptions {
        use_tabs: true,
        ..FormatOptions::default()
    };
    assert_fixed_point(
        "<template>\n\t<button\n\t\t@click=\"() => {\n\t\t\ta(1);\n\n\t\t\ta(2);\n\t\t}\"\n\t/>\n</template>\n",
        "<template>\n\t<button\n\t\t@click=\"() => {\n\t\t\ta(1);\n\n\t\t\ta(2);\n\t\t}\"\n\t/>\n</template>\n",
        &tabs,
    );
    let crlf = FormatOptions {
        end_of_line: EndOfLine::Crlf,
        ..FormatOptions::default()
    };
    assert_fixed_point(
        "<template>\r\n  <button\r\n    @click=\"() => {\r\n      a(1);\r\n\r\n      a(2);\r\n    }\"\r\n  />\r\n</template>\r\n",
        "<template>\r\n  <button\r\n    @click=\"() => {\r\n      a(1);\r\n\r\n      a(2);\r\n    }\"\r\n  />\r\n</template>\r\n",
        &crlf,
    );
}

#[test]
fn blank_lines_inside_a_template_literal_value_keep_their_bytes() {
    // A template literal's bytes are runtime value: a whitespace-only line
    // inside it is not layout and must survive untouched.
    let input = "<template>\n  <button\n    @click=\"() => {\n      a(`x\n  \ny`);\n\n      a(2);\n    }\"\n  />\n</template>\n";
    let first = format_sfc(input, &FormatOptions::default())
        .expect("format")
        .code;
    assert!(first.contains("a(`x\n  \ny`);"), "{first:?}");
    assert_fixed_point(&first, &first, &FormatOptions::default());
}
