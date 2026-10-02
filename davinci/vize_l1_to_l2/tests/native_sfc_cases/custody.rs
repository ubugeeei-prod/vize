use super::support::options;
use oxc_ast::ast::Expression;
use vize_l0::Allocator;
use vize_l1::embed::Lang;
use vize_l1_to_l2::native_file::{NativeTemplateOutcome, lower_sfc_native};
use vize_l2::file::FileIssueKind;

#[test]
fn native_ts_custody_keeps_unsupported_type_assertion_and_real_file_refusal() {
    let arena = Allocator::default();
    let source =
        "<script setup lang=ts>const value = 1;</script><template>{{value as number}}</template>";
    let observed = lower_sfc_native(&arena, source, options());
    assert!(observed.admitted().is_none());
    let script = observed.scripts().first().unwrap();
    assert!(script.syntax().unwrap().source_type().is_typescript());
    let template = observed.template().unwrap();
    assert_eq!(template.lang(), Lang::Ts);
    assert!(matches!(
        template.outcome(),
        NativeTemplateOutcome::Produced(_)
    ));
    let produced = template.produced().unwrap();
    assert!(!produced.is_supported());
    let embed = produced.embeds.first().unwrap();
    assert_eq!(embed.syntax.grammar().lang, Lang::Ts);
    assert!(matches!(
        embed.syntax.expression(),
        Some(Expression::TSAsExpression(_))
    ));
    assert!(embed.node.is_none());
    assert!(core::ptr::eq(
        template.component().unwrap().block().root_source(),
        source
    ));
    assert!(core::ptr::eq(
        template.component().unwrap().block().source(),
        template.block().source()
    ));
    assert_eq!(template.embeds().len(), produced.embeds.len());
    assert!(core::ptr::eq(template.embeds(), produced.embeds.as_slice()));
    let rejected = observed.rejected_file().unwrap();
    assert!(rejected.rejected_file().is_none());
    let partial = rejected.file().unwrap();
    assert!(!partial.is_complete());
    assert!(
        partial
            .template_issues()
            .iter()
            .any(|issue| { issue.kind == FileIssueKind::UnsupportedSyntax })
    );
}
