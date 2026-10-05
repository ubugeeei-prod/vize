//! Issue #7914: replacing a binding does not mutate its old reactive snapshot.

use vize_croquis::{
    reactivity::{ReactivityLoss, ReactivityLossKind},
    script_parser::parse_script_setup,
};

fn property(source: &str, target: &str, initializer: &str) -> ReactivityLoss {
    let start = source.find(initializer).unwrap() as u32;
    ReactivityLoss {
        kind: ReactivityLossKind::ReactivePropertyExtract {
            source_name: "props".into(),
            prop_name: "modelValue".into(),
            target_name: target.into(),
        },
        start,
        end: start + initializer.len() as u32,
    }
}

fn assert_losses(source: &str, expected: Vec<ReactivityLoss>) {
    let parsed = parse_script_setup(source);
    assert_eq!(
        format!("{:#?}", parsed.reactivity.losses()),
        format!("{expected:#?}")
    );
}

fn argument(source: &str) -> ReactivityLoss {
    let start = (source.find("useFeature(last)").unwrap() + "useFeature(".len()) as u32;
    ReactivityLoss {
        kind: ReactivityLossKind::FunctionArgumentExtract {
            source_name: "props.modelValue".into(),
            argument_name: "last".into(),
            callee_name: "useFeature".into(),
        },
        start,
        end: start + "last".len() as u32,
    }
}

#[test]
fn original_reporter_retains_only_the_initial_snapshot() {
    let file = include_str!("fixtures/issue-7914/LazyInput.vue.fixture");
    let source = file
        .strip_prefix("<script setup lang=\"ts\">\n")
        .unwrap()
        .split("</script>")
        .next()
        .unwrap();
    assert_losses(
        source,
        vec![property(source, "lastEmitted", "props.modelValue")],
    );
}

#[test]
fn later_copy_and_composable_reads_do_not_keep_replaced_provenance() {
    let source = "const props = defineProps<{ modelValue: string }>();\nlet last = props.modelValue;\nlast = 'fresh';\nconst copy = last;\nuseFeature(last);";
    assert_losses(source, vec![property(source, "last", "props.modelValue")]);
}

#[test]
fn right_hand_side_keeps_previous_origin_until_the_assignment_finishes() {
    let source = include_str!("fixtures/issue-7914/RightHandSide.vue.fixture")
        .strip_prefix("<script setup lang=\"ts\">\n")
        .unwrap()
        .split("</script>")
        .next()
        .unwrap();
    assert_losses(
        source,
        vec![
            property(source, "last", "props.modelValue"),
            argument(source),
        ],
    );
}

#[test]
fn sequence_and_self_assignment_keep_rhs_reads_before_replacement() {
    for assignment in [
        "last = (useFeature(last), 'fresh');",
        "last = last; useFeature(last);",
    ] {
        let source = format!(
            "const props = defineProps<{{ modelValue: string }}>();\nlet last = props.modelValue;\n{assignment}"
        );
        assert_losses(
            &source,
            vec![
                property(&source, "last", "props.modelValue"),
                argument(&source),
            ],
        );
    }
}

#[test]
fn a_member_write_still_reports_the_complete_original_loss() {
    let source = "const props = defineProps<{ modelValue: { count: number } }>();\nlet last = props.modelValue;\nlast.count = 1;";
    let start = source.find("last.count").unwrap() as u32;
    let mutation = ReactivityLoss {
        kind: ReactivityLossKind::PlainValueAlias {
            source_name: "props.modelValue".into(),
            alias_name: "<mutation>".into(),
            target_name: "last.count".into(),
        },
        start,
        end: start + "last.count".len() as u32,
    };
    assert_losses(
        source,
        vec![property(source, "last", "props.modelValue"), mutation],
    );
}

#[test]
fn replacing_a_shadowing_parameter_does_not_clear_the_outer_snapshot() {
    let source = "const props = defineProps<{ modelValue: { count: number } }>();\nlet last = props.modelValue;\nfunction replace(last: unknown) { last = {}; }\nlast.count = 1;";
    let start = source.find("last.count").unwrap() as u32;
    let mutation = ReactivityLoss {
        kind: ReactivityLossKind::PlainValueAlias {
            source_name: "props.modelValue".into(),
            alias_name: "<mutation>".into(),
            target_name: "last.count".into(),
        },
        start,
        end: start + "last.count".len() as u32,
    };
    assert_losses(
        source,
        vec![property(source, "last", "props.modelValue"), mutation],
    );
}
