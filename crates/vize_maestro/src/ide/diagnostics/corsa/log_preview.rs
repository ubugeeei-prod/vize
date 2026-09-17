//! Bounded log previews must not make authored Unicode crash diagnostics.

pub(super) fn log_preview(text: &str, max_bytes: usize) -> &str {
    &text[..text.floor_char_boundary(max_bytes)]
}

#[cfg(test)]
mod tests {
    use super::log_preview;
    use vize_s0::cstr;

    #[test]
    fn preserves_short_empty_and_exact_boundary_text() {
        assert_eq!(log_preview("", 100), "");
        assert_eq!(log_preview("text", 0), "");
        assert_eq!(log_preview("text", 2), "te");
        assert_eq!(log_preview("text", 4), "text");
        assert_eq!(log_preview("text", usize::MAX), "text");
        assert_eq!(log_preview("\u{3042}\u{10400}", 3), "\u{3042}");
        assert_eq!(log_preview("\u{3042}\u{10400}", 7), "\u{3042}\u{10400}");
    }

    #[test]
    fn every_diagnostic_log_budget_stops_before_split_multibyte_characters() {
        for budget in [50, 80, 100] {
            for character in ['\u{e9}', '\u{3042}', '\u{10400}'] {
                for split in 1..character.len_utf8() {
                    let prefix = "x".repeat(budget - split);
                    let text = cstr!("{prefix}{character}tail");
                    assert_eq!(log_preview(&text, budget), prefix);
                }
            }
        }
    }

    #[test]
    fn debug_source_map_logging_handles_multibyte_previews() {
        let line = cstr!("// @vize-map: {} -> 7:9", "x\u{3042}".repeat(25));
        assert!(!line.is_char_boundary(80));
        let subscriber = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .with_writer(std::io::sink)
            .finish();
        tracing::subscriber::with_default(subscriber, || {
            let mappings =
                crate::ide::DiagnosticService::parse_vize_map_comments(&cstr!("label;\n{line}\n"));
            let ranges = mappings
                .iter()
                .map(|mapping| mapping.as_ref().map(|m| (m.start, m.end)))
                .collect::<Vec<_>>();
            assert_eq!(ranges, [Some((7, 9)), None]);
        });
    }
}
