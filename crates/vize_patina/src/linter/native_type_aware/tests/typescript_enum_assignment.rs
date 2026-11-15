//! #12994: qualified enum flows match typescript-eslint without changing safe flows.
//! Remaining checker-surface-dependent flows are tracked in #13070.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};

const RULE: &str = "type/no-unsafe-enum-assignment";
const SOURCE: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/linter/no-unsafe-enum-assignment-12994/Matrix.vue.txt"
);

const ASSIGNMENT: &str =
    "The value assigned here does not have a shared enum type with the expected enum 'Fruit'.";
const ARGUMENT: &str =
    "The argument passed here does not have a shared enum type with the expected enum 'Fruit'.";
const ACCESS_BOTH: &str = "The computed key used here does not have a shared enum type with the expected enum 'Fruit', 'Vegetable'.";

#[test]
fn qualified_enum_flows_have_exact_packets_and_safe_controls() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let result = lint_sfc_with_corsa(&linter, SOURCE, "Matrix.vue");
    let mut actual = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .map(|diagnostic| {
            let (line, column) = line_column(SOURCE, diagnostic.start);
            (line, column, diagnostic.message.to_string())
        })
        .collect::<Vec<_>>();
    actual.sort();

    let mut expected = vec![
        (14, 3, ASSIGNMENT.to_string()),
        (20, 3, ASSIGNMENT.to_string()),
        (29, 10, ACCESS_BOTH.to_string()),
        (32, 7, ASSIGNMENT.to_string()),
        (34, 7, ASSIGNMENT.to_string()),
        (35, 36, ASSIGNMENT.to_string()),
        (39, 7, ASSIGNMENT.to_string()),
        (40, 7, ASSIGNMENT.to_string()),
        (43, 13, ARGUMENT.to_string()),
        (44, 26, ARGUMENT.to_string()),
        (44, 38, ARGUMENT.to_string()),
        (70, 7, ASSIGNMENT.to_string()),
        (73, 3, ASSIGNMENT.to_string()),
        (81, 13, ARGUMENT.to_string()),
        (85, 10, ARGUMENT.to_string()),
        (86, 11, ARGUMENT.to_string()),
    ];
    expected.sort();
    assert_eq!(actual, expected);
}

fn line_column(source: &str, offset: u32) -> (u32, u32) {
    let mut line = 1u32;
    let mut column = 1u32;
    for (index, character) in source.char_indices() {
        if index >= offset as usize {
            break;
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}
