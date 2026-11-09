//! Keep the bytewise boundary walk as an independent optimization oracle.
#![expect(clippy::disallowed_macros, reason = "tests assemble boundary fixtures")]
use super::*;
use alloc::format;

#[test]
fn vectorized_template_scan_matches_bytewise_boundaries() {
    let tokens = [
        "雪🌸 text",
        "{",
        "{{",
        "{{ x }}",
        "{{ '</template>' }}",
        "<div class='</template>'>",
        "</div>",
        "<template #x>",
        "</template>",
        "<template/>",
        "<!-- </template> -->",
        "<!--",
        "<div x='",
        "<textarea></template></textarea>",
        "<script> '</template>' </script>",
        "<style>.x {}</style>",
        "{{ /}}/.test(x) }}",
        "{{ `x${x}` }}",
    ];
    let arena = Allocator::default();
    for first in tokens {
        for second in tokens {
            for third in tokens {
                let source = format!("<template>{first}{second}{third}</template>");
                assert_eq!(
                    find_template_close(&source, 10),
                    bytewise_template_close(&arena, &source, 10),
                    "{source}"
                );
            }
        }
    }
}

#[test]
fn boundary_only_open_tag_scan_keeps_spelling_and_rejects_malformed_attributes() {
    let arena = Allocator::default();
    for source in [
        "<div>",
        "<template #x/>",
        "<div x='</template>' :y=foo>",
        "<div x='",
        "<div / x>",
        "</div>",
        "<雪>",
        "<div 雪=🌸>",
    ] {
        let captured = read_open(&arena, source, 0);
        let boundary = read_open_boundary(source, 0, |_| {});
        match (captured, boundary) {
            (Ok(Some(captured)), Ok(Some(boundary))) => {
                assert_eq!(captured.name, boundary.name);
                assert_eq!(captured.end, boundary.end);
                assert_eq!(captured.self_closing, boundary.self_closing);
            }
            (Ok(None), Ok(None)) | (Err(()), Err(())) => {}
            _ => panic!("boundary/capture disagreement: {source}"),
        }
    }
}

fn bytewise_template_close<'a>(
    allocator: &'a Allocator,
    source: &'a str,
    from: usize,
) -> (usize, Option<MatchedClose>, bool) {
    let bytes = source.as_bytes();
    let (mut pos, mut depth) = (from, 1usize);
    let mut uncertain = false;
    while pos < bytes.len() {
        if bytes
            .get(pos..)
            .is_some_and(|rest| rest.starts_with(b"<!--"))
        {
            pos = find_bytes(bytes, pos + 4, b"-->").map_or(bytes.len(), |end| end + 3);
            continue;
        }
        if bytes.get(pos..).is_some_and(|rest| rest.starts_with(b"{{")) {
            if let Some(end) = skip_interpolation(bytes, pos + 2) {
                pos = end;
            } else {
                uncertain = true;
                // An unclosed interpolation owns the remaining input. Do not
                // re-scan its suffix or guess a root closing tag inside it.
                pos = bytes.len();
            }
            continue;
        }
        if bytes.get(pos) != Some(&b'<') {
            pos += 1;
            continue;
        }
        if let Some(close) = close_at(bytes, pos, "template") {
            depth -= 1;
            if depth == 0 {
                return (pos, Some(close), uncertain);
            }
            pos = close.end();
            continue;
        }
        if let Ok(Some(open)) = read_open(allocator, source, pos) {
            if open.name.eq_ignore_ascii_case("template") && !open.self_closing {
                depth += 1;
            }
            if ["script", "style", "textarea", "title"]
                .iter()
                .any(|name| open.name.eq_ignore_ascii_case(name))
                && !open.self_closing
                && let Some(close) = find_close(bytes, open.end, open.name)
            {
                pos = close.end();
                continue;
            }
            pos = open.end;
            continue;
        }
        pos += 1;
    }
    (bytes.len(), None, uncertain)
}
