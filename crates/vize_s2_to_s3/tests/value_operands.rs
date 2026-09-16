use vize_davinci::folio::{Folio, FolioMode};
use vize_s0::{Allocator, String};
use vize_s2_to_s3::lower;
use vize_s3::operand::{OperandRole as Role, ValueKind};
use vize_s3::values_folio::S3ValuesFolio;
use vize_s3::verify::verify;

#[test]
fn lowered_values_outlive_both_source_text_and_the_s2_arena() {
    let s3_arena = Allocator::default();
    let lowered = {
        let source = String::from(
            r#"<Widget v-model:title.trim="title" @save.once.capture="save"><template #default="{ item }"><input disabled value="" :[field].prop="item" /><span v-if="ready">{{ item }}</span><span v-else>wait</span><p v-for="(value, key, index) in items">{{ value }}</p><!-- note --></template></Widget>"#,
        );
        let s2_arena = Allocator::default();
        let (tree, errors) = vize_s1::parse(&s2_arena, &source);
        assert!(errors.is_empty(), "{errors:?}");
        let s2 = vize_s1_to_s2::lower(&s2_arena, &tree, &errors);
        lower(&s3_arena, &s2.root)
    };
    assert_eq!(verify(&lowered.program), []);
    let operands = &lowered.program.operands;
    for (role, text) in [
        (Role::Tag, "Widget"),
        (Role::ModelRead, "title"),
        (Role::Modifier, "once"),
        (Role::Modifier, "capture"),
        (Role::Modifier, "prop"),
        (Role::Params, "{ item }"),
        (Role::Name, "field"),
        (Role::Condition, "ready"),
        (Role::ForSource, "items"),
        (Role::ForValue, "value"),
        (Role::ForKey, "key"),
        (Role::ForIndex, "index"),
        (Role::Text, "wait"),
    ] {
        assert!(
            operands
                .iter()
                .any(|op| op.role == role && op.value.text == text),
            "missing {role:?}: {text}; {operands:?}"
        );
    }
    assert!(
        operands
            .iter()
            .any(|op| op.role == Role::ModelWrite && op.target.is_some())
    );
    let modifiers: Vec<_> = operands
        .iter()
        .filter(|op| op.role == Role::Modifier)
        .map(|op| op.value.text)
        .collect();
    assert_eq!(modifiers, ["once", "capture", "prop"]);
    let model_attributes: Vec<_> = operands
        .iter()
        .filter(|op| op.role == Role::ModelAttribute)
        .map(|op| (op.name, op.value.kind, op.value.text))
        .collect();
    assert_eq!(
        model_attributes,
        [
            (Some("element-kind"), ValueKind::Literal, "component"),
            (Some("trim"), ValueKind::Absent, ""),
        ]
    );
    for (name, kind) in [
        ("disabled", ValueKind::Absent),
        ("value", ValueKind::Literal),
    ] {
        let operand = operands
            .iter()
            .find(|op| op.role == Role::Attribute && op.name == Some(name))
            .unwrap();
        assert_eq!(operand.value.kind, kind);
        assert_eq!(operand.value.text, "");
    }
    let branches: Vec<_> = operands
        .iter()
        .filter(|op| op.role == Role::Condition)
        .collect();
    assert_eq!(branches.len(), 2);
    assert_ne!(branches[0].region, branches[1].region);
    assert_eq!(branches[1].value.kind, ValueKind::Absent);
    let folio = S3ValuesFolio::of(&lowered.program);
    assert_eq!(
        S3ValuesFolio::parse(folio.print_to_string(FolioMode::Full).as_str()).unwrap(),
        folio
    );
}

#[test]
fn an_s2_preserved_comment_is_copied_into_the_s3_arena() {
    use vize_s0::{Box, Span, Vec};
    use vize_s2::op::{CommentOp, Op, Region};

    let s3_arena = Allocator::default();
    let lowered = {
        let s2_arena = Allocator::default();
        let mut ops = Vec::new_in(&&s2_arena);
        ops.push(Op::Comment(Box::new_in(
            CommentOp {
                content: s2_arena.alloc_str(" note "),
                span: Span::new(0, 13),
            },
            &&s2_arena,
        )));
        lower(&s3_arena, &Region { ops })
    };
    assert_eq!(verify(&lowered.program), []);
    assert_eq!(lowered.program.operands.len(), 1);
    let operand = &lowered.program.operands[0];
    assert_eq!(operand.role, Role::Comment);
    assert_eq!(operand.value.kind, ValueKind::Literal);
    assert_eq!(operand.value.text, " note ");
}
