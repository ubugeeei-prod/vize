//! Speculative regex scanning must not corrupt the enclosing block's location.

use super::parse_sfc;
use crate::sfc::types::SfcParseOptions;

#[test]
fn failed_regex_lookahead_preserves_the_authored_script_boundary() {
    // Exact crash-cb5e9e231223f2ca26ea0c8afa91c13ebc01b75e from #6277,
    // followed by smaller regressions and independent newline variants.
    for source in [
        "<script>[/[/A[u</script>`p\\\n \\;\\\np\\c\nX\\\n",
        "<script>[/[</script>\\\n",
        "<script>[/[</script>\\\nunterminated\n",
        "\n<script>\n[/[</script>\\\n",
        "\r\n<script>\r\n[/[</script>\\\r\n",
    ] {
        let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
        let script = descriptor.script.unwrap();
        let content_start = source.find('>').unwrap() + 1;
        let content_end = source.find("</script>").unwrap();
        let end_line = source[..content_end]
            .bytes()
            .filter(|b| *b == b'\n')
            .count()
            + 1;
        let end_column = source[..content_end]
            .rfind('\n')
            .map_or(content_end + 1, |offset| content_end - offset);
        assert_eq!(script.content, &source[content_start..content_end]);
        assert_eq!(script.loc.start, content_start);
        assert_eq!(script.loc.end, content_end);
        assert_eq!(script.loc.end_line, end_line);
        assert_eq!(script.loc.end_column, end_column);
        assert_eq!(script.loc.tag_end, content_end + "</script>".len());
    }
}

#[test]
fn regex_recovery_does_not_consume_the_following_style_block() {
    let source = "<script>[/[</script>\n<style>\\\n.x { color: red }</style>";
    let descriptor = parse_sfc(source, SfcParseOptions::default()).unwrap();
    assert_eq!(descriptor.script.unwrap().content, "[/[");
    assert_eq!(descriptor.styles.len(), 1);
    assert_eq!(descriptor.styles[0].content, "\\\n.x { color: red }");
}
