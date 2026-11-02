use super::{VifControlFlowChain, find_branch_condition_expr};
use vize_carton::CompactString;
use vize_croquis::{ScopeId, TemplateExpression, TemplateExpressionKind};

fn expression(content: &str, kind: TemplateExpressionKind, guard: &str) -> TemplateExpression {
    TemplateExpression {
        content: CompactString::new(content),
        kind,
        start: 0,
        end: content.len() as u32,
        scope_id: ScopeId::ROOT,
        vif_guard: Some(CompactString::new(guard)),
    }
}

#[derive(Debug, PartialEq)]
struct Branch {
    condition: Option<std::string::String>,
    range: std::ops::Range<usize>,
    expression: Option<usize>,
}

#[test]
fn commented_else_if_keeps_the_complete_branch_plan_and_its_authored_owner() {
    for condition in [
        "\nprops.item.kind !== 'a' &&\n// comment\nprops.item.kind !== 'b'\n",
        "\nprops.item.kind !== 'a' &&\nprops.item.kind !== 'b'\n",
        "\r\nprops.item.kind !== 'a' &&\r\n// comment\r\nprops.item.kind !== 'b'\r\n",
        "props.item.kind !== 'a' /* block */ && props.item.kind !== 'b'",
    ] {
        let guard = format!("(props.item) && !(props.flag) && ({condition})");
        let rows = [
            expression(
                "props.flag",
                TemplateExpressionKind::VIf,
                "(props.item) && (props.flag)",
            ),
            expression(condition, TemplateExpressionKind::VIf, &guard),
            expression(
                "props.item.kind",
                TemplateExpressionKind::Interpolation,
                &guard,
            ),
        ];
        let refs: Vec<_> = rows.iter().collect();
        let chain = VifControlFlowChain::collect(&refs, 0).unwrap();
        assert_eq!(
            chain
                .prefix
                .iter()
                .map(|term| (term.negated, term.condition, term.raw))
                .collect::<Vec<_>>(),
            [(false, "props.item", "(props.item)")]
        );
        let branches = chain
            .branches
            .iter()
            .map(|branch| Branch {
                condition: branch.condition.map(str::to_owned),
                range: branch.start..branch.end,
                expression: branch.condition_expr_index,
            })
            .collect::<Vec<_>>();
        assert_eq!(
            branches,
            [
                Branch {
                    condition: Some("props.flag".to_owned()),
                    range: 0..1,
                    expression: Some(0)
                },
                Branch {
                    condition: Some(condition.to_owned()),
                    range: 1..3,
                    expression: Some(1)
                },
            ]
        );
        assert_eq!(chain.end, 3);
    }
}

#[test]
fn comment_equivalence_does_not_match_other_roles_or_erase_real_tokens() {
    let rows = [
        expression(
            "item.kind === '// literal'",
            TemplateExpressionKind::Interpolation,
            "",
        ),
        expression(
            "item.kind === '// literal' /* owned */",
            TemplateExpressionKind::VIf,
            "",
        ),
        expression("item.kind === 'different'", TemplateExpressionKind::VIf, ""),
    ];
    let refs: Vec<_> = rows.iter().collect();
    assert_eq!(
        find_branch_condition_expr(&refs, 0, 3, "item.kind === '// literal' // retained\n"),
        Some(1)
    );
    assert_eq!(
        find_branch_condition_expr(&refs, 0, 1, "item.kind === '// literal'"),
        None
    );
    assert_eq!(
        find_branch_condition_expr(&refs, 0, 3, "item.kind === 'missing'"),
        None
    );
    assert_eq!(
        find_branch_condition_expr(&refs, 2, 3, "item.kind === 'different'"),
        Some(2)
    );
}
