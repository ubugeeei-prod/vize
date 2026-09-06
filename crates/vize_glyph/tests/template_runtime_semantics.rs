use vize_glyph::{EndOfLine, FormatOptions, format_template};

fn format(source: &str) -> String {
    format_template(source, &FormatOptions::default())
        .unwrap()
        .to_string()
}

fn assert_fixed_point(source: &str, expected: &str) {
    let first = format(source);
    assert_eq!(first.as_str(), expected);
    assert_eq!(
        format(&first),
        first,
        "fmt(fmt(source)) must equal fmt(source)"
    );
}

#[test]
fn text_and_interpolation_boundaries_do_not_gain_runtime_whitespace() {
    assert_fixed_point("<p>Hello</p>", "<p>Hello</p>");
    assert_fixed_point(
        "<p>Hello {{name}} world</p>",
        "<p>Hello {{ name }} world</p>",
    );
    assert_fixed_point(
        "<p> Hello {{name}} world </p>",
        "<p> Hello {{ name }} world </p>",
    );
    assert_fixed_point(
        "<div><span>Hello</span><strong>{{name}}</strong></div>",
        "<div><span>Hello</span><strong>{{ name }}</strong></div>",
    );

    assert_ne!(format("<p>Hello</p>"), format("<p> Hello </p>"));
    assert_ne!(
        format("<p>Hello {{name}}</p>"),
        format("<p>Hello{{name}}</p>")
    );
}

#[test]
fn dynamic_attribute_groups_remain_as_authored() {
    let source = concat!(
        r#"<Widget :z="first()" :a="second()" "#,
        r#":getter="observed" :assigned="value = next" :updated="count++" "#,
        r#"v-model="model" @z="onZ()" @a="onA()" />"#,
    );
    let formatted = format(source);
    assert_eq!(format(&formatted), formatted);
    let names = [
        "v-model=",
        ":z=",
        ":a=",
        ":getter=",
        ":assigned=",
        ":updated=",
        "@z=",
        "@a=",
    ];
    let positions = names.map(|name| formatted.find(name).unwrap());
    assert!(positions.windows(2).all(|pair| pair[0] < pair[1]));
}

#[test]
fn literal_attributes_sort_only_inside_safe_unique_runs() {
    assert_fixed_point(
        r#"<Widget :value="read()" class="box" id="root" :next="write()" />"#,
        r#"<Widget id="root" class="box" :value="read()" :next="write()" />"#,
    );
    assert_fixed_point(
        r#"<Widget class="first" class="second" :value="read()" />"#,
        r#"<Widget class="first" class="second" :value="read()" />"#,
    );
    assert_fixed_point(
        r#"<Widget title="{{ first() }}" id="root" />"#,
        r#"<Widget title="{{ first() }}" id="root" />"#,
    );
}

#[test]
fn dynamic_argument_bindings_keep_literal_collision_order() {
    assert_fixed_point(
        r#"<Widget :[name]="dynamic" id="literal" class="box" />"#,
        r#"<Widget :[name]="dynamic" id="literal" class="box" />"#,
    );
    assert_fixed_point(
        r#"<Widget v-bind:[name]="dynamic" id="literal" class="box" />"#,
        r#"<Widget :[name]="dynamic" id="literal" class="box" />"#,
    );
}

#[test]
fn comments_and_inline_sibling_whitespace_keep_their_boundaries() {
    assert_fixed_point(
        "<p>Hello<!-- keep -->{{name}}</p>",
        "<p>Hello<!-- keep -->{{ name }}</p>",
    );
    let adjacent = format("<div><i>A</i><i>B</i></div>");
    let separated = format("<div><i>A</i> <i>B</i></div>");
    assert_eq!(adjacent, "<div><i>A</i><i>B</i></div>");
    assert_eq!(separated, "<div><i>A</i> <i>B</i></div>");
    assert_ne!(adjacent, separated);
    assert_eq!(format(&separated), separated);
    for horizontal in ["\t", "   "] {
        assert_fixed_point(
            &format!("<div><i>A</i>{horizontal}<i>B</i></div>"),
            "<div><i>A</i> <i>B</i></div>",
        );
    }
    assert_fixed_point(
        "<p><span>A</span> <!-- keep --> <span>B</span></p>",
        "<p><span>A</span> <!-- keep --> <span>B</span></p>",
    );
    assert_fixed_point(
        "<div><i>A</i>\r<i>B</i></div>",
        "<div><i>A</i>\n  <i>B</i></div>",
    );
    // Vue's default whitespace condensation removes whitespace-only sibling
    // gaps containing either CR or LF. Rewriting the lone CR to the configured
    // layout newline therefore preserves runtime text instead of injecting it.
}

#[test]
fn configured_crlf_and_tabs_do_not_leak_into_adjacent_text() {
    let options = FormatOptions {
        end_of_line: EndOfLine::Crlf,
        use_tabs: true,
        ..FormatOptions::default()
    };
    let source = "<div>\r\n\t<span>Hello</span>\r\n</div>";
    let first = format_template(source, &options).unwrap();
    let second = format_template(&first, &options).unwrap();
    assert_eq!(first, second);
    assert_eq!(
        first.as_str(),
        "<div>\r\n\t<span>Hello</span>\r\n</div>",
        "configured CRLF and tabs must drive layout only, never adjacent text"
    );
}

#[test]
fn wrapped_interpolations_stay_adjacent_to_their_element_boundary() {
    let options = FormatOptions {
        print_width: 30,
        ..FormatOptions::default()
    };
    let source = "<p>{{ veryLongFunctionName(firstArgument, secondArgument) }}</p>";
    let first = format_template(source, &options).unwrap();
    let second = format_template(&first, &options).unwrap();
    assert_eq!(first, second);
    assert!(first.starts_with("<p>{{\n"), "{first}");
    assert!(first.ends_with("}}</p>"), "{first}");

    let compact = "<p>A{{ veryLongFunctionName(firstArgument, secondArgument) }}B</p>";
    let first = format_template(compact, &options).unwrap();
    let second = format_template(&first, &options).unwrap();
    assert_eq!(first, second);
    assert!(first.starts_with("<p>A{{\n"), "{first}");
    assert!(first.ends_with("}}B</p>"), "{first}");

    let spaced = "<p>A {{ veryLongFunctionName(firstArgument, secondArgument) }} B</p>";
    let first = format_template(spaced, &options).unwrap();
    let second = format_template(&first, &options).unwrap();
    assert_eq!(first, second);
    assert!(first.starts_with("<p>A {{\n"), "{first}");
    assert!(first.ends_with("}} B</p>"), "{first}");

    let mixed = concat!(
        "<a>{{ result.fields ? result.fields[0]?.tableAlias ?? ",
        "result.fields[0]?.table : fallback }} ({{ result.rows.length }})</a>",
    );
    let first = format_template(mixed, &options).unwrap();
    let second = format_template(&first, &options).unwrap();
    assert_eq!(first, second);
    assert!(first.starts_with("<a>{{\n"), "{first}");
    assert!(
        first.ends_with("}} ({{ result.rows.length }})</a>"),
        "{first}"
    );

    let reversed = concat!(
        "<a>{{ result.rows.length }} ({{ result.fields ? ",
        "result.fields[0]?.tableAlias ?? result.fields[0]?.table : fallback }})</a>",
    );
    let first = format_template(reversed, &options).unwrap();
    let second = format_template(&first, &options).unwrap();
    assert_eq!(first, second);
    assert!(
        first.starts_with("<a>{{ result.rows.length }} ({{\n"),
        "{first}"
    );
    assert!(first.ends_with("}})</a>"), "{first}");
}
