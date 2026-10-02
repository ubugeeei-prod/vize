use super::sfc_comment_ranges;
use tower_lsp::lsp_types::{FoldingRange, FoldingRangeKind};
use vize_l0::Allocator;
use vize_l0::config::{VueDialect, VueVersion};
use vize_l1::{SurfaceParseOptions, container::vue::DescriptorOptions};
use vize_l1_to_l2::native_file::lower_sfc_native;

fn options() -> DescriptorOptions {
    DescriptorOptions {
        version: VueVersion::V3,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    }
}

fn expected(start: u32, end: u32) -> FoldingRange {
    FoldingRange {
        start_line: start,
        start_character: None,
        end_line: end,
        end_character: None,
        kind: Some(FoldingRangeKind::Comment),
        collapsed_text: None,
    }
}

#[test]
fn actual_admitted_file_projects_nonzero_script_comments_without_reparsing() {
    let arena = Allocator::default();
    let source = "<script>\n/*\n ordinary\n*/\nconst value = 1;\n</script>";
    let observed = lower_sfc_native(&arena, source, options());
    let native = observed.admitted().expect("actual ordinary JS sealed File");
    assert!(observed.scripts()[0].block().span().start > 0);
    assert!(core::ptr::eq(
        native.file().file().artifact().source(),
        source
    ));
    assert_eq!(sfc_comment_ranges(&native), Ok(vec![expected(1, 2)]));
}

#[test]
fn authentic_setup_before_ordinary_keeps_original_document_comment_order() {
    let arena = Allocator::default();
    let source = "<script setup>\n/*\n setup\n*/\nconst setupValue = 1;\n</script>\n<script>\n/*\n ordinary\n*/\nconst ordinaryValue = 2;\n</script>";
    let observed = lower_sfc_native(&arena, source, options());
    let native = observed
        .admitted()
        .expect("actual selected role membership");
    assert_eq!(native.file().file().units().len(), 2);
    assert_eq!(
        sfc_comment_ranges(&native),
        Ok(vec![expected(1, 2), expected(7, 8)])
    );
}

#[test]
fn whole_file_crlf_utf8_and_unicode_separators_keep_authored_lsp_lines() {
    let arena = Allocator::default();
    let source = "\r\n<script lang=\"ts\">\r\n/* 😀\u{2028}\u{2029}\r\n 日本語\r\n*/\r\nconst value = 1;\r\n</script>";
    let observed = lower_sfc_native(&arena, source, options());
    let native = observed.admitted().expect("actual TypeScript File profile");
    assert_eq!(sfc_comment_ranges(&native), Ok(vec![expected(2, 3)]));
}

#[test]
fn valid_typed_declaration_retains_ts_program_without_admitting_native_file() {
    let arena = Allocator::default();
    let source = "\r\n<script lang=\"ts\">\r\n/* 😀\u{2028}\u{2029}\r\n 日本語\r\n*/\r\nconst value: number = 1;\r\n</script>";
    let observed = lower_sfc_native(&arena, source, options());
    let syntax = observed.scripts()[0].syntax().expect("retained TS Program");
    assert!(syntax.source_type().is_typescript());
    assert!(syntax.admitted_program().is_some());
    // The current genuine File family refuses type annotations, even when the
    // original TS Program is valid. A host or comment adapter cannot bypass it.
    assert!(observed.admitted().is_none());
    assert!(!observed.issues().is_empty());
}

#[test]
fn admitted_file_has_no_folds_for_literals_line_and_adjacent_block_comments() {
    let arena = Allocator::default();
    let source = "<script>const x = '/* fake */';\n// line\n/* single */\n/*\n*/</script>";
    let observed = lower_sfc_native(&arena, source, options());
    let native = observed
        .admitted()
        .expect("actual original Program comments");
    assert_eq!(sfc_comment_ranges(&native), Ok(Vec::new()));
}

#[test]
fn parser_recovery_keeps_diagnostics_without_minting_the_required_file_view() {
    let arena = Allocator::default();
    let source = "<script>/*\n retained\n*/ const = ;</script>";
    let observed = lower_sfc_native(&arena, source, options());
    assert!(observed.scripts()[0].syntax().is_some());
    assert!(observed.admitted().is_none());
    assert!(!observed.issues().is_empty());
}

#[test]
fn rejected_template_cannot_promote_valid_script_comments_to_a_complete_file() {
    let arena = Allocator::default();
    let source = "<script>/*\n valid\n*/ const value = 1;</script><template><div v-if=\"value\" /></template>";
    let observed = lower_sfc_native(&arena, source, options());
    // v-if is outside this producer's documented first ordinary native family.
    assert!(observed.scripts()[0].syntax().is_some());
    assert!(observed.admitted().is_none());
    assert!(!observed.issues().is_empty());
}
