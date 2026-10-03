//! Borrow the default-export entry already retained by the original parser.

use oxc_span::Span;
use oxc_syntax::module_record::{ExportEntry, ExportExportName};

use super::AdmittedProgram;

/// The sole direct value-default export in the original parser's ModuleRecord.
///
/// These ranges borrow the actual parser entry and keyword, not caller-provided
/// spans or a second AST/source walk. This does not certify an empty object,
/// strict-module semantics, whole File eligibility or native product admission.
/// Named-as-default, type-only, indirect, star and multiple entries do not mint
/// this bounded receipt. Other statements and imports remain original syntax.
///
/// ```compile_fail
/// use oxc_parser::OriginalDefaultExport;
/// fn forge() { let _ = OriginalDefaultExport { keyword: true }; }
/// ```
/// ```compile_fail
/// use oxc_parser::OriginalDefaultExport;
/// fn copy(receipt: OriginalDefaultExport<'_, '_>) { let _ = receipt.clone(); }
/// ```
/// The actual parser observation stays borrowed:
/// ```compile_fail
/// use oxc_parser::ProgramObservation;
/// fn discard(observation: ProgramObservation<'_>) {
///     let receipt = observation.admitted().unwrap().sole_default_export().unwrap();
///     drop(observation);
///     let _ = receipt.statement_span();
/// }
/// ```
pub struct OriginalDefaultExport<'p, 'a> {
    program: AdmittedProgram<'p, 'a>,
    entry: &'p ExportEntry<'a>,
    keyword: &'p Span,
}

impl<'p, 'a> AdmittedProgram<'p, 'a> {
    /// Project one existing original local Default entry in constant time.
    /// Syntax admission remains separate from the entry's semantic consumers.
    #[must_use]
    pub fn sole_default_export(&self) -> Option<OriginalDefaultExport<'p, 'a>> {
        let records = &self.observation.parsed.module_record;
        if !records.indirect_export_entries.is_empty() || !records.star_export_entries.is_empty() {
            return None;
        }
        let [entry] = records.local_export_entries.as_slice() else {
            return None;
        };
        let ExportExportName::Default(keyword) = &entry.export_name else {
            return None;
        };
        if entry.is_type || entry.module_request.is_some() {
            return None;
        }
        Some(OriginalDefaultExport {
            program: AdmittedProgram { observation: self.observation },
            entry,
            keyword,
        })
    }
}

impl<'p, 'a> OriginalDefaultExport<'p, 'a> {
    /// The same complete original Program/source/profile/options observation.
    #[must_use]
    pub fn program(&self) -> &AdmittedProgram<'p, 'a> {
        &self.program
    }

    #[must_use]
    pub fn statement_span(&self) -> Span {
        self.entry.statement_span
    }

    /// The original declaration or expression range, without claiming its kind.
    #[must_use]
    pub fn declaration_span(&self) -> Span {
        self.entry.span
    }

    #[must_use]
    pub fn default_keyword_span(&self) -> Span {
        *self.keyword
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ParseOptions, Parser};
    use oxc_allocator::Allocator;
    use oxc_ast::ast::Statement;
    use oxc_span::{GetSpan, SourceType};

    #[test]
    fn original_default_entry_keeps_actual_utf8_crlf_keyword_and_ast_ranges() {
        let arena = Allocator::default();
        let source = "/*雪🌸*/\r\nexport /*🌸*/ default { /*retained*/ }; //尾\r\n";
        for profile in [SourceType::mjs(), SourceType::ts()] {
            let observation = Parser::new(&arena, source, profile).parse_observed();
            let admitted = observation.admitted().expect("original admitted syntax");
            let receipt = admitted.sole_default_export().expect("original default entry");
            let Statement::ExportDefaultDeclaration(export) = &admitted.program().body[0] else {
                panic!("original default AST");
            };
            assert_eq!(receipt.statement_span(), export.span);
            assert_eq!(receipt.declaration_span(), export.declaration.span());
            assert_eq!(
                &source[receipt.default_keyword_span().start as usize
                    ..receipt.default_keyword_span().end as usize],
                "default"
            );
            assert_eq!(
                &source[receipt.declaration_span().start as usize
                    ..receipt.declaration_span().end as usize],
                "{ /*retained*/ }"
            );
            assert!(core::ptr::eq(
                receipt.entry,
                &observation.parsed.module_record.local_export_entries[0]
            ));
            let ExportExportName::Default(keyword) = &receipt.entry.export_name else {
                panic!("actual default variant");
            };
            assert!(core::ptr::eq(receipt.keyword, keyword));
            assert!(core::ptr::eq(receipt.program().program(), admitted.program()));
            assert!(core::ptr::eq(receipt.program().source(), source));
            assert_eq!(receipt.program().source_type(), profile);
            assert_eq!(receipt.program().options(), ParseOptions::default());
            assert_eq!(receipt.program().program().comments.len(), 4);
            assert!(observation.diagnostics().is_empty());
        }
    }

    #[test]
    fn receipt_does_not_claim_empty_objects_or_default_options() {
        let arena = Allocator::default();
        for source in [
            "export default 42;",
            "export default function f() {}",
            "export default class C {}",
            "import value from 'dep'; export default value;",
        ] {
            let observation = Parser::new(&arena, source, SourceType::mjs()).parse_observed();
            let receipt = observation.admitted().unwrap().sole_default_export().unwrap();
            assert_eq!(receipt.program().source(), source);
            assert!(receipt.declaration_span().start < receipt.declaration_span().end);
        }
        let options = ParseOptions { preserve_parens: false, ..ParseOptions::default() };
        let observation = Parser::new(&arena, "export default ({});", SourceType::ts())
            .with_options(options)
            .parse_observed();
        let receipt = observation.admitted().unwrap().sole_default_export().unwrap();
        assert_eq!(receipt.program().options(), options);
        let observation =
            Parser::new(&arena, "const value=010; export default {};", SourceType::mjs())
                .parse_observed();
        let receipt = observation.admitted().unwrap().sole_default_export().unwrap();
        assert!(receipt.program().has_legacy_literals());
        assert!(observation.diagnostics().is_empty());
    }

    #[test]
    fn named_indirect_star_type_and_ts_duplicate_exports_cannot_supply_receipt() {
        let arena = Allocator::default();
        for source in [
            "const value=1; export { value as default };",
            "export { default } from 'dep';",
            "export * from 'dep';",
            "export default {}; export const other=1;",
            "export default {}; export * from 'dep';",
            "export default {}; export { other } from 'dep';",
            "export default interface Value {}",
            "export default {}; export default {};",
            "const value=1; export default {}; export { value as default };",
            "const value=1;",
        ] {
            let observation = Parser::new(&arena, source, SourceType::ts()).parse_observed();
            let admitted = observation.admitted().expect(source);
            assert!(admitted.sole_default_export().is_none(), "{source}");
            assert!(observation.diagnostics().is_empty(), "unchanged TS diagnostics: {source}");
        }
    }

    #[test]
    fn original_owner_move_retains_module_entry_ast_and_comment_identity() {
        let arena = Allocator::default();
        let source = "/*雪*/ export default {}; //🌸\r\n";
        let observation = Parser::new(&arena, source, SourceType::mjs()).parse_observed();
        let admitted = observation.admitted().unwrap();
        let receipt = admitted.sole_default_export().unwrap();
        let entry = receipt.entry as *const _;
        let body = receipt.program().program().body.as_ptr();
        let comments = receipt.program().program().comments.as_ptr();
        let statement_span = receipt.statement_span();
        let observation = Box::new(observation);
        let receipt = observation.admitted().unwrap().sole_default_export().unwrap();
        assert_eq!(receipt.entry as *const _, entry);
        assert_eq!(receipt.program().program().body.as_ptr(), body);
        assert_eq!(receipt.program().program().comments.as_ptr(), comments);
        assert_eq!(receipt.statement_span(), statement_span);
        let foreign = Parser::new(&arena, source, SourceType::mjs()).parse_observed();
        let foreign = foreign.admitted().unwrap().sole_default_export().unwrap();
        assert!(!core::ptr::eq(receipt.program().program(), foreign.program().program()));
        assert!(!core::ptr::eq(receipt.entry, foreign.entry));
    }

    #[test]
    fn recovered_duplicate_and_escaped_keyword_syntax_keep_original_refusal() {
        let arena = Allocator::default();
        for source in [
            "export default {}; export default {};",
            "export default {",
            "export def\\u0061ult {};",
        ] {
            let observation = Parser::new(&arena, source, SourceType::mjs()).parse_observed();
            assert!(observation.admitted().is_none(), "{source}");
            assert!(observation.diagnostics().has_errors(), "{source}");
        }
    }
}
