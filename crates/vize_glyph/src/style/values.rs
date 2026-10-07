use super::blank_lines::{Adjustment, Token};

// Reuse the complete-token custody pass for declaration continuation gaps.
// A nested-rule prelude can look like a property until its opening brace;
// no pending value edit is committed before a real declaration boundary.
#[derive(Default)]
pub(super) struct DeclarationValues {
    blocks: Vec<bool>,
    start: usize,
    colon: bool,
    force: bool,
    comma: bool,
    multiline: bool,
    next_value: bool,
    pending: Vec<Adjustment>,
}

impl DeclarationValues {
    pub(super) fn observe(
        &mut self,
        source: &str,
        authored: &Token<'_>,
        target: &Token<'_>,
        rule_brace: bool,
        adjustments: &mut Vec<Adjustment>,
    ) {
        if authored.parens != 0 || authored.brackets != 0 {
            return;
        }
        match authored.text {
            "{" => {
                self.blocks.push(rule_brace);
                self.reset(authored.start + 1);
            }
            ";" | "}" => {
                if self.colon && self.comma && (self.force || self.multiline) {
                    adjustments.append(&mut self.pending);
                }
                if authored.text == "}" {
                    self.blocks.pop();
                }
                self.reset(authored.start + 1);
            }
            ":" if !self.colon && self.blocks.last() == Some(&true) => {
                let property = source
                    .get(self.start..authored.start)
                    .unwrap_or_default()
                    .trim();
                // Custom values retain their token-stream whitespace ownership.
                // Restrict recognition to an ordinary CSS property identifier.
                if !property.starts_with("--")
                    && property.bytes().next().is_some_and(|first| {
                        first.is_ascii_alphabetic() || matches!(first, b'-' | b'_')
                    })
                    && property
                        .bytes()
                        .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
                {
                    self.colon = true;
                    self.force = property.eq_ignore_ascii_case("transition")
                        || property.eq_ignore_ascii_case("box-shadow")
                        || [
                            "-webkit-transition",
                            "-moz-transition",
                            "-ms-transition",
                            "-webkit-box-shadow",
                            "-moz-box-shadow",
                        ]
                        .iter()
                        .any(|name| property.eq_ignore_ascii_case(name));
                    self.multiline = true;
                    self.next_value = true;
                }
            }
            "," if self.colon => {
                self.comma = true;
                self.next_value = true;
            }
            _ if self.colon && self.next_value => {
                self.multiline &= authored.gap.contains(['\r', '\n']);
                let mut adjustment = Adjustment::new(authored, target, 1, false);
                adjustment.value = true;
                self.pending.push(adjustment);
                self.next_value = false;
            }
            _ => {}
        }
    }

    fn reset(&mut self, start: usize) {
        self.start = start;
        self.colon = false;
        self.force = false;
        self.comma = false;
        self.multiline = false;
        self.next_value = false;
        self.pending.clear();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn parsed_custom_brace_values_cannot_become_declaration_blocks() {
        use crate::FormatOptions;
        use lightningcss::stylesheet::{ParserOptions, StyleSheet};
        let source = ".a {\n  --values: {transition: x, y;};\n}\n";
        let parsed = StyleSheet::parse(source, ParserOptions::default()).unwrap();
        let layout = super::super::rule_layout::RuleLayout::from_parse(source, &parsed.rules);
        let printed = source.into();
        assert_eq!(
            super::super::blank_lines::preserve_rule_layout(
                source,
                printed,
                &FormatOptions::default(),
                Some(&layout),
                source
            ),
            source,
        );
    }
}
