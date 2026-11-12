//! #12889, #12892, #12898: union/intersection rules read checker facts about
//! type nodes (resolved aliases, union constituents, the empty object type).
use super::{corsa_available, lint_sfc_with_corsa};
use crate::{LintPreset, Linter};

const RULES: &[&str] = &[
    "type/no-duplicate-type-constituents",
    "type/no-generated-empty-object-type",
    "type/no-redundant-type-constituents",
];

const SOURCE: &str = r#"<script setup lang="ts">
type A = 'A';
type B = boolean;
type Data = { name: string; num: number };
type NullableData = null | Data;
type Lits = A | string;
type Bools = B | false;
type Never = 'a' | ('b' & 'c');
type Dup = A | A;
type Same = { a: string } | { a: string };
type Nested = (A | 'B') | (A | 'B');
function returnsNever(): string | never {
  return '';
}
function optional(a?: string | undefined) {}
type Unexpected = Omit<NullableData, 'name'>;
type Kept = Omit<Data, 'name'>;
type Both = Omit<Data, 'name' | 'num'> & Omit<Data, 'name' | 'num'>;
type Literal = {} | {};
type Interface = Array<string>;
</script>
"#;

/// (line, column, rule) inside `SOURCE`.
const EXPECTED: &[(u32, u32, &str)] = &[
    (6, 13, "type/no-redundant-type-constituents"),
    (7, 18, "type/no-redundant-type-constituents"),
    (8, 21, "type/no-redundant-type-constituents"),
    (9, 16, "type/no-duplicate-type-constituents"),
    (10, 29, "type/no-duplicate-type-constituents"),
    (11, 28, "type/no-duplicate-type-constituents"),
    (15, 32, "type/no-duplicate-type-constituents"),
    (16, 19, "type/no-generated-empty-object-type"),
    (18, 42, "type/no-duplicate-type-constituents"),
    (18, 13, "type/no-generated-empty-object-type"),
    (19, 21, "type/no-duplicate-type-constituents"),
];

#[test]
fn type_node_facts_match_typescript_eslint() {
    if !corsa_available() {
        return;
    }
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let result = lint_sfc_with_corsa(&linter, SOURCE, "Constituents.vue");
    let mut actual = result
        .diagnostics
        .iter()
        .filter(|diagnostic| RULES.contains(&diagnostic.rule_name))
        .map(|diagnostic| {
            let (line, column) = line_column(SOURCE, diagnostic.start);
            (line, column, diagnostic.rule_name)
        })
        .collect::<Vec<_>>();
    actual.sort_unstable();
    let mut expected = EXPECTED.to_vec();
    expected.sort_unstable();
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
