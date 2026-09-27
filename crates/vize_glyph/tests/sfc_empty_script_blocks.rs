#![expect(
    clippy::expect_used,
    clippy::string_slice,
    reason = "tests assert by panicking"
)]
use vize_glyph::{FormatOptions, FormatResult, format_script, format_sfc};

fn check_options() -> FormatOptions {
    FormatOptions {
        skip_script_stabilization: true,
        ..FormatOptions::default()
    }
}

fn canonical(source: &str) -> FormatResult {
    let write = FormatOptions::default();
    let check = check_options();
    let predicted = format_sfc(source, &check).expect("check-mode format");
    let first = format_sfc(source, &write).expect("first write");
    let second = format_sfc(first.code.as_str(), &write).expect("second write");

    assert_eq!(
        predicted.changed, first.changed,
        "check/write verdict drift"
    );
    assert_eq!(predicted.code, first.code, "check/write output drift");
    assert_eq!(
        first.code, second.code,
        "first write must reach a fixed point"
    );
    first
}

fn script_bodies(source: &str) -> Vec<&str> {
    let mut rest = source;
    let mut bodies = Vec::new();
    while let Some(start) = rest.find("<script") {
        rest = &rest[start..];
        let content_start = rest.find('>').expect("script opening tag") + 1;
        rest = &rest[content_start..];
        let content_end = rest.find("</script>").expect("script closing tag");
        bodies.push(&rest[..content_end]);
        rest = &rest[content_end + "</script>".len()..];
    }
    bodies
}

fn snapshot_bytes(name: &str, output: &str) {
    insta::with_settings!({prepend_module_to_snapshot => false}, {
        insta::assert_binary_snapshot!(name, output.as_bytes().to_vec());
    });
}

#[test]
fn empty_statement_only_blocks_keep_a_canonical_statement() {
    for (index, source) in [
        "<script setup>\n;\n</script>\n",
        "<script setup lang=\"ts\">\n; ; ;\n</script>\n",
        "<script>\n;\n</script>\n",
        "<script lang=\"ts\">\n;\n;\n</script>\n",
        "<script>\n;\n</script>\n<script setup lang=\"ts\">\n;;\n</script>\n",
    ]
    .into_iter()
    .enumerate()
    {
        let output = canonical(source);
        let names = [
            "history_sfc_empty_setup_js.txt",
            "history_sfc_empty_setup_ts.txt",
            "history_sfc_empty_classic_js.txt",
            "history_sfc_empty_classic_ts.txt",
            "history_sfc_empty_both.txt",
        ];
        snapshot_bytes(names[index], &output.code);
        let bodies = script_bodies(output.code.as_str());
        assert!(!bodies.is_empty(), "fixture must retain a script block");
        assert!(
            bodies.iter().all(|body| body.trim() == ";"),
            "empty statements must canonicalize without erasing the block:\n{}",
            output.code
        );
    }
}

#[test]
fn comments_and_directives_keep_script_blocks_nonempty() {
    for (index, (source, marker)) in [
        (
            "<script setup>\n; // keep line\n;\n</script>\n",
            "// keep line",
        ),
        (
            "<script lang=\"ts\">\n; /* keep block */ ;\n</script>\n",
            "/* keep block */",
        ),
        ("<script>\n\"use strict\"; ;\n</script>\n", "\"use strict\""),
    ]
    .into_iter()
    .enumerate()
    {
        let output = canonical(source);
        let names = [
            "history_sfc_line_comment_script.txt",
            "history_sfc_block_comment_script.txt",
            "history_sfc_directive_script.txt",
        ];
        snapshot_bytes(names[index], &output.code);
        assert!(
            output.code.contains(marker),
            "script marker disappeared:\n{}",
            output.code
        );
        assert!(
            script_bodies(output.code.as_str())
                .iter()
                .all(|body| !body.trim().is_empty()),
            "script block became compiler-empty:\n{}",
            output.code
        );
    }
}

#[test]
fn standalone_script_cleanup_still_removes_empty_statements() {
    let output = format_script("; ; ;", &FormatOptions::default()).expect("standalone format");
    snapshot_bytes("history_script_empty_statements.txt", &output);
    assert!(
        output.trim().is_empty(),
        "standalone cleanup changed: {output}"
    );
}
