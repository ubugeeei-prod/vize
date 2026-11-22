{
    let mut result = CssLintResult::default();

    // Parse disable comments
    let disabled_rules = if respect_disable_comments {
        Some(DisabledRules::parse(source))
    } else {
        None
    };

    // The structure is read without evaluating anything; an unbalanced
    // block or an unterminated string or comment has no reliable one.
    if !self.style_rules.is_empty()
        && let Ok(base) = u32::try_from(offset)
    {
        let sheet = read_structure(language, source, base);
        if sheet.is_balanced() && sheet.is_terminated() {
            for rule in &self.style_rules {
                rule.check(&sheet, &mut result);
            }
        }
    }

    // Parse plain CSS with lightning-css; if parsing fails, those rules
    // are skipped.
    if language == StyleLanguage::Css
        && !self.rules.is_empty()
        && let Ok(stylesheet) = crate::rules::css_engine::parse_stylesheet(source)
    {
        for rule in &self.rules {
            rule.check(source, &stylesheet, offset, &mut result);
        }
    }

    // Filter out disabled diagnostics
    if let Some(disabled) = disabled_rules {
        let line_starts: Vec<usize> = std::iter::once(0)
            .chain(
                source
                    .bytes()
                    .enumerate()
                    .filter_map(|(i, b)| if b == b'\n' { Some(i + 1) } else { None }),
            )
            .collect();

        let get_line =
            |pos: u32| -> usize { line_starts.partition_point(|&start| start <= pos as usize) };

        result.diagnostics.retain(|d| {
            // Rule spans are file-absolute. Disable comments are measured
            // in this style block, so subtract the block's file offset
            // before choosing a line. Leaving the offset in maps every
            // diagnostic onto the block's last line.
            let local = d.start.saturating_sub(offset as u32);
            let line = get_line(local);
            !disabled.is_disabled(d.rule_name, line)
        });

        // Recalculate counts
        result.error_count = result
            .diagnostics
            .iter()
            .filter(|d| matches!(d.severity, Severity::Error))
            .count();
        result.warning_count = result
            .diagnostics
            .iter()
            .filter(|d| matches!(d.severity, Severity::Warning))
            .count();
    }

    result
}
