//! A function argument list authored one argument per line keeps the line
//! breaks at both of its parentheses, instead of pulling the first argument
//! onto the opening line and the closing parenthesis onto the last argument.
use vize_glyph::{EndOfLine, FormatOptions, format_sfc, format_style};

fn assert_style_fixed_point(input: &str, expected: &str, options: &FormatOptions) {
    let first = format_style(input, options).expect("first pass");
    assert_eq!(first.as_str(), expected, "first pass");
    let second = format_style(&first, options).expect("second pass");
    assert_eq!(second.as_str(), expected, "second pass");
}

#[test]
fn sfc_style_keeps_multiline_function_arguments() {
    let input = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-multiline-style-function-arguments/App.vue.txt"
    );
    let expected = include_str!(
        "../../../tests/_fixtures/differential/formatter/sfc-multiline-style-function-arguments/reference.expected.txt"
    );
    let options = FormatOptions::default();
    let mut current: vize_l0::String = input.into();
    for pass in 0..3 {
        let result = format_sfc(&current, &options).expect("format");
        assert_eq!(result.code.as_str(), expected, "pass {pass}");
        assert_eq!(result.changed, current.as_str() != expected, "pass {pass}");
        current = result.code;
    }
}

#[test]
fn closing_parenthesis_lines_up_with_the_declaration_in_every_indent_style() {
    let input = ".a {\n  background: linear-gradient(\n    to right,\n    red,\n    blue\n  );\n  mask: linear-gradient(\n    to right,\n    red .5%,\n    blue\n  );\n}\n";
    let tabs = FormatOptions {
        use_tabs: true,
        ..FormatOptions::default()
    };
    assert_style_fixed_point(
        input,
        ".a {\n\tbackground: linear-gradient(\n\t\tto right,\n\t\tred,\n\t\tblue\n\t);\n\tmask: linear-gradient(\n\t\tto right,\n\t\tred .5%,\n\t\tblue\n\t);\n}\n",
        &tabs,
    );
    let four = FormatOptions {
        tab_width: 4,
        ..FormatOptions::default()
    };
    assert_style_fixed_point(
        input,
        ".a {\n    background: linear-gradient(\n        to right,\n        red,\n        blue\n    );\n    mask: linear-gradient(\n        to right,\n        red .5%,\n        blue\n    );\n}\n",
        &four,
    );
}

#[test]
fn crlf_argument_lists_keep_their_terminators() {
    let input = ".a {\r\n  background: linear-gradient(\r\n    to right,\r\n    red,\r\n    blue\r\n  );\r\n}\r\n";
    assert_style_fixed_point(
        input,
        ".a {\r\n  background: linear-gradient(\r\n    to right,\r\n    red,\r\n    blue\r\n  );\r\n}",
        &FormatOptions {
            end_of_line: EndOfLine::Auto,
            ..FormatOptions::default()
        },
    );
}

#[test]
fn lists_the_printer_joins_stay_joined() {
    let input = ".a {\n  width: calc(\n    100% - 1px\n  );\n}\n";
    assert_style_fixed_point(
        input,
        ".a {\n  width: calc(100% - 1px);\n}\n",
        &FormatOptions::default(),
    );
}
