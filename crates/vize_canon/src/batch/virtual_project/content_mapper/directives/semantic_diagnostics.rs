//! Mapper diagnostics reported from the host's semantic pass.
//!
//! TypeScript appends a transform's `diagnostics` to the virtual file's parse
//! diagnostics, and `tsc` and `tsrs` skip semantic collection for the whole
//! program while any parse diagnostic exists, so one Vue compile error hid
//! every type error of the project (#12522). An expect directive whose virtual
//! range is empty never matches a diagnostic. The host therefore always
//! reports its unused-expect diagnostic, from this file's semantic pass, with
//! the directive's code, message, source and original range: the diagnostic
//! the user saw before.
//!
//! The semantic pass skips a file that starts with `// @ts-nocheck`, which
//! Vize writes for unchecked JavaScript and for a script that opts out. When
//! such a file carries mapper diagnostics, its pragma gives way to an ignore
//! directive over the whole virtual text, which drops every TypeScript
//! diagnostic of the file as the pragma did.

use vize_l0::String as CompactString;

use super::super::protocol::{
    ContentMapperDiagnosticDirective, ContentMapperDiagnosticDirectives, ContentMapperTransform,
    ContentMapperUnusedExpectDiagnostic, DIRECTIVE_POLICY_EXPECT, DIRECTIVE_POLICY_IGNORE,
};
use super::super::publish::TYPESCRIPT_NOCHECK_DIRECTIVE;

impl ContentMapperTransform {
    /// Move every mapper diagnostic onto an expect directive that nothing can
    /// use, so the host reports it without blocking semantic diagnostics.
    pub fn with_semantic_diagnostics(mut self) -> Self {
        if self.diagnostics.is_empty() {
            return self;
        }
        let unchecked = blank_leading_nocheck(&mut self.text);
        let end = self.text.len();
        let mut table = match self.diagnostic_directives.take() {
            // Unchecked files never reached the semantic pass, so their other
            // directives never applied; the ignore below covers all of them.
            Some(table) if !unchecked => table,
            _ => ContentMapperDiagnosticDirectives {
                unused_expect_directive_diagnostics: Vec::new(),
                directives: Vec::new(),
            },
        };
        if unchecked {
            table.directives.push(ContentMapperDiagnosticDirective([
                0,
                0,
                0,
                end,
                DIRECTIVE_POLICY_IGNORE,
                0,
            ]));
        }
        for diagnostic in self.diagnostics.drain(..) {
            let index = table.unused_expect_directive_diagnostics.len();
            table
                .unused_expect_directive_diagnostics
                .push(ContentMapperUnusedExpectDiagnostic {
                    code: diagnostic.code,
                    message_text: diagnostic.message_text,
                });
            // Empty ranges at the end overlap no other directive.
            table.directives.push(ContentMapperDiagnosticDirective([
                diagnostic.start,
                diagnostic.length,
                end,
                end,
                DIRECTIVE_POLICY_EXPECT,
                index,
            ]));
        }
        self.diagnostic_directives = Some(table);
        self
    }
}

/// Blank Vize's `// @ts-nocheck` lines in the leading comment block with
/// spaces, so every offset stays put. Returns whether any was blanked.
fn blank_leading_nocheck(text: &mut CompactString) -> bool {
    let pragma = TYPESCRIPT_NOCHECK_DIRECTIVE.trim_end();
    let mut starts = Vec::new();
    let mut offset = 0;
    for line in text.split('\n') {
        let content = line.trim_end_matches('\r');
        if content == pragma {
            starts.push(offset);
        } else if !content.trim().is_empty() && !content.trim_start().starts_with("//") {
            break;
        }
        offset += line.len() + 1;
    }
    let blank = std::iter::repeat_n(' ', pragma.len()).collect::<CompactString>();
    for start in &starts {
        text.replace_range(*start..*start + pragma.len(), &blank);
    }
    !starts.is_empty()
}

#[cfg(test)]
mod tests {
    use super::super::super::protocol::{
        CONTENT_MAPPER_GENERATED_DIAGNOSTIC_CODE, CONTENT_MAPPER_SFC_PARSE_ERROR_CODE,
        ContentMapperDiagnostic, ContentMapperDiagnosticDirective,
        ContentMapperDiagnosticDirectives, ContentMapperTransform,
        ContentMapperUnusedExpectDiagnostic,
    };

    fn transform(
        text: &str,
        directives: Option<ContentMapperDiagnosticDirectives>,
    ) -> ContentMapperTransform {
        ContentMapperTransform {
            text: text.into(),
            extension: ".tsx",
            mappings: Vec::new(),
            semantic_links: Vec::new(),
            diagnostic_directives: directives,
            diagnostics: vec![
                ContentMapperDiagnostic {
                    message_text: "Vue compile error: a".into(),
                    start: 7,
                    length: 3,
                    code: CONTENT_MAPPER_GENERATED_DIAGNOSTIC_CODE,
                },
                ContentMapperDiagnostic {
                    message_text: "b".into(),
                    start: 20,
                    length: 1,
                    code: CONTENT_MAPPER_SFC_PARSE_ERROR_CODE,
                },
            ],
        }
    }

    fn unused(code: i32, message_text: &str) -> ContentMapperUnusedExpectDiagnostic {
        ContentMapperUnusedExpectDiagnostic {
            code,
            message_text: message_text.into(),
        }
    }

    #[test]
    fn diagnostics_become_empty_expect_directives_after_the_existing_table() {
        let existing = ContentMapperDiagnosticDirectives {
            unused_expect_directive_diagnostics: vec![unused(
                4,
                "Unused '@vue-expect-error' directive.",
            )],
            directives: vec![
                ContentMapperDiagnosticDirective([1, 2, 3, 9, 1, 0]),
                ContentMapperDiagnosticDirective([0, 0, 9, 12, 0, 0]),
            ],
        };
        let result = transform("const a = 1;", Some(existing)).with_semantic_diagnostics();
        assert_eq!(result.text, "const a = 1;");
        assert_eq!(result.diagnostics, Vec::new());
        assert_eq!(
            result.diagnostic_directives,
            Some(ContentMapperDiagnosticDirectives {
                unused_expect_directive_diagnostics: vec![
                    unused(4, "Unused '@vue-expect-error' directive."),
                    unused(
                        CONTENT_MAPPER_GENERATED_DIAGNOSTIC_CODE,
                        "Vue compile error: a"
                    ),
                    unused(CONTENT_MAPPER_SFC_PARSE_ERROR_CODE, "b"),
                ],
                directives: vec![
                    ContentMapperDiagnosticDirective([1, 2, 3, 9, 1, 0]),
                    ContentMapperDiagnosticDirective([0, 0, 9, 12, 0, 0]),
                    ContentMapperDiagnosticDirective([7, 3, 12, 12, 1, 1]),
                    ContentMapperDiagnosticDirective([20, 1, 12, 12, 1, 2]),
                ],
            })
        );
    }

    #[test]
    fn an_unchecked_file_trades_its_pragma_for_one_ignore_over_the_whole_text() {
        let text = "/// <reference types=\"vue/jsx\" />\r\n// @ts-nocheck\n// ====\n\nconst a = 1;\n// @ts-nocheck\n";
        let existing = ContentMapperDiagnosticDirectives {
            unused_expect_directive_diagnostics: Vec::new(),
            directives: vec![ContentMapperDiagnosticDirective([0, 0, 40, 50, 0, 0])],
        };
        let result = transform(text, Some(existing)).with_semantic_diagnostics();
        assert_eq!(
            result.text,
            "/// <reference types=\"vue/jsx\" />\r\n              \n// ====\n\nconst a = 1;\n// @ts-nocheck\n"
        );
        let end = text.len();
        assert_eq!(
            result.diagnostic_directives,
            Some(ContentMapperDiagnosticDirectives {
                unused_expect_directive_diagnostics: vec![
                    unused(
                        CONTENT_MAPPER_GENERATED_DIAGNOSTIC_CODE,
                        "Vue compile error: a"
                    ),
                    unused(CONTENT_MAPPER_SFC_PARSE_ERROR_CODE, "b"),
                ],
                directives: vec![
                    ContentMapperDiagnosticDirective([0, 0, 0, end, 0, 0]),
                    ContentMapperDiagnosticDirective([7, 3, end, end, 1, 0]),
                    ContentMapperDiagnosticDirective([20, 1, end, end, 1, 1]),
                ],
            })
        );
    }

    #[test]
    fn a_transform_without_diagnostics_is_unchanged() {
        let mut clean = transform("// @ts-nocheck\nconst a = 1;", None);
        clean.diagnostics.clear();
        let result = clean.with_semantic_diagnostics();
        assert_eq!(result.text, "// @ts-nocheck\nconst a = 1;");
        assert_eq!(result.diagnostic_directives, None);
    }
}
