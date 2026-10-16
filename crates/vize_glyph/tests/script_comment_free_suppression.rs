use vize_glyph::{FormatOptions, format_script};

#[test]
fn comment_free_suppression_fast_path_preserves_complete_public_outputs() {
    // Whole references are authored independently of the optimized predicates.
    // Marker-looking string contents must remain normal code; real markers
    // retain the exact suppressed source and formatting resumes afterwards.
    let cases = [
        ("const x={a:1,b:2}", "const x = { a: 1, b: 2 };\n"),
        (
            "const value='// prettier-ignore';const other=`/* oxfmt-ignore */`",
            "const value = \"// prettier-ignore\";\nconst other = `/* oxfmt-ignore */`;\n",
        ),
        (
            "// prettier-ignore\nconst x={a:1,b:2}\nconst y=2",
            "// prettier-ignore\nconst x={a:1,b:2}\nconst y = 2;\n",
        ),
        (
            "/* oxfmt-ignore */\nconst x={a:1,b:2}\nconst y=2",
            "/* oxfmt-ignore */\nconst x={a:1,b:2}\nconst y = 2;\n",
        ),
        (
            "const x=1 // ordinary\nconst y=2",
            "const x = 1; // ordinary\nconst y = 2;\n",
        ),
        (
            "const x={a:1,b:2} // prettier-ignore\nconst y=2",
            "const x={a:1,b:2} // prettier-ignore\nconst y = 2;\n",
        ),
    ];
    let options = FormatOptions::default();
    for (source, expected) in cases {
        let mut current = source.to_owned();
        for pass in 1..=3 {
            let output = format_script(&current, &options).unwrap();
            assert_eq!(output.as_str(), expected, "pass {pass}: {source}");
            current = output.to_string();
        }
    }
}
