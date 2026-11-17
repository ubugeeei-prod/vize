//! #12994, #13070: enum flows match typescript-eslint without changing safe flows.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::diagnostic::Severity;
use crate::{LintPreset, Linter};
use std::path::Path;

const RULE: &str = "type/no-unsafe-enum-assignment";
const SOURCE: &str = include_str!(
    "../../../../../../tests/_fixtures/differential/linter/no-unsafe-enum-assignment-12994/Matrix.vue.txt"
);

const ASSIGNMENT: &str =
    "The value assigned here does not have a shared enum type with the expected enum 'Fruit'.";
const ARGUMENT: &str =
    "The argument passed here does not have a shared enum type with the expected enum 'Fruit'.";
const RETURN: &str =
    "The value returned here does not have a shared enum type with the expected enum 'Fruit'.";
const ACCESS: &str =
    "The computed key used here does not have a shared enum type with the expected enum 'Fruit'.";
const ACCESS_BOTH: &str = "The computed key used here does not have a shared enum type with the expected enum 'Fruit', 'Vegetable'.";

/// `(start line, start column, end line, end column, severity, message)`.
type Packet = (u32, u32, u32, u32, Severity, String);

fn packets(source: &str) -> Vec<Packet> {
    let linter = Linter::with_preset(LintPreset::Opinionated);
    let result = lint_sfc_with_corsa(&linter, source, "Matrix.vue");
    let mut packets = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .map(|diagnostic| {
            let (line, column) = line_column(source, diagnostic.start);
            let (end_line, end_column) = line_column(source, diagnostic.end);
            (
                line,
                column,
                end_line,
                end_column,
                diagnostic.severity,
                diagnostic.message.to_string(),
            )
        })
        .collect::<Vec<_>>();
    packets.sort_by_key(|packet| (packet.0, packet.1, packet.2, packet.3));
    packets
}

fn packet(start: (u32, u32), end: (u32, u32), message: &str) -> Packet {
    (
        start.0,
        start.1,
        end.0,
        end.1,
        Severity::Warning,
        message.to_string(),
    )
}

#[test]
fn qualified_enum_flows_have_exact_packets_and_safe_controls() {
    if !corsa_available() {
        return;
    }
    let expected = vec![
        packet((14, 3), (14, 13), ASSIGNMENT),
        packet((20, 3), (20, 13), ASSIGNMENT),
        // A mapped enum key, read from the object's declaration.
        packet((24, 7), (24, 8), ACCESS),
        packet((29, 10), (29, 21), ACCESS_BOTH),
        packet((32, 7), (32, 38), ASSIGNMENT),
        packet((34, 7), (34, 43), ASSIGNMENT),
        packet((35, 36), (35, 45), ASSIGNMENT),
        packet((39, 7), (39, 48), ASSIGNMENT),
        packet((40, 7), (40, 54), ASSIGNMENT),
        packet((43, 13), (43, 23), ARGUMENT),
        packet((44, 26), (44, 36), ARGUMENT),
        packet((44, 38), (44, 39), ARGUMENT),
        // Recursive named interface members and a structural generic-box
        // return (typescript-eslint `invalid[113]` and `invalid[129]`).
        packet((55, 7), (55, 40), ASSIGNMENT),
        packet((66, 3), (66, 14), RETURN),
        packet((70, 7), (70, 69), ASSIGNMENT),
        packet((73, 3), (73, 22), ASSIGNMENT),
        // Arguments after a variadic tuple spread take the rest parameter
        // (typescript-eslint `invalid[5]` and `invalid[121]`).
        packet((77, 29), (77, 30), ARGUMENT),
        packet((79, 30), (79, 31), ARGUMENT),
        packet((81, 13), (81, 25), ARGUMENT),
        packet((85, 10), (85, 11), ARGUMENT),
        packet((86, 11), (86, 12), ARGUMENT),
    ];
    assert_eq!(packets(SOURCE), expected);
}

/// typescript-eslint exempts an enum's own bit flags wherever it checks a
/// flow, compares a private `#name` with no inherited member, and reads a
/// computed key only from a member's name, wherever its comments are.
#[test]
fn bit_flags_private_names_and_member_bodies_stay_safe() {
    if !corsa_available() {
        return;
    }
    let source = r#"<script setup lang="ts">
enum Fruit {
  Apple,
  Banana,
}
enum Flags {
  A = 1,
  B = 2,
}
function combined(): Flags {
  return Flags.A | Flags.B;
}
const arrow = (): Flags => Flags.A | ~Flags.B;
declare function consume(flags: Flags): void;
consume(Flags.A | Flags.B);
interface HasFlags {
  flags: Flags;
}
class Holder implements HasFlags {
  flags = Flags.A | Flags.B;
}
class Base {
  fruit!: Fruit;
}
class Derived extends Base {
  #fruit = 0;
}
const fruitNames = { [Fruit.Apple]: "apple" };
const described = {
  describe() {
    return fruitNames[Fruit.Apple];
  },
};
function plain(): Flags {
  return 3;
}
consume(4);
described["describe"];
declare const commented: { [Fruit.Apple /* key: fruit */]: string };
commented[1];
void [combined, arrow, described, plain, Holder, Derived];
</script>
"#;
    let flags = |message: &str| message.replace("'Fruit'", "'Flags'");
    assert_eq!(
        packets(source),
        vec![
            packet((35, 3), (35, 12), &flags(RETURN)),
            packet((37, 9), (37, 10), &flags(ARGUMENT)),
            packet((40, 11), (40, 12), ACCESS,),
        ]
    );
}

/// The #12994 comment's component: a form object handed to a typed service,
/// and numbers given to `(() => Chip) | Chip` and `Chip | Chip[]`. Each flow
/// is reported once, where typescript-eslint reports it: an object literal
/// argument by its member, not as a whole.
#[test]
fn reported_form_flows_match_upstream_packets() {
    if !corsa_available() {
        return;
    }
    let chips = r#"export enum Chip {
  None,
  Small,
  Large,
}

export interface Info {
  chip: Chip;
  dna: string;
}

export declare function submit(info: Info): void;
export declare function useChip(source: (() => Chip) | Chip): void;
export declare function pick(source: Chip | Chip[]): void;
"#;
    let form = r#"<script setup lang="ts">
import { type Chip, pick, submit, useChip } from "./chips";

const form = { chip: 0, dna: "" };
submit(form);
submit({ chip: 1, dna: "" });
useChip(0);
pick(2);
const direct: Chip = 0;
</script>

<template>
  <p>{{ direct }}</p>
</template>
"#;
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/vize-tests");
    std::fs::create_dir_all(&workspace).expect("test workspace");
    let project = tempfile::Builder::new()
        .prefix("enum-form-12994-")
        .tempdir_in(workspace)
        .expect("temporary project");
    let source_dir = project.path().join("src");
    std::fs::create_dir(&source_dir).expect("source directory");
    std::fs::write(
        project.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"Bundler","strict":true,"target":"ES2022","noEmit":true},"include":["src/**/*.ts","src/**/*.vue"]}"#,
    )
    .expect("tsconfig");
    std::fs::write(source_dir.join("chips.ts"), chips).expect("chips module");
    let path = source_dir.join("Form.vue");
    std::fs::write(&path, form).expect("form component");
    let linter =
        Linter::with_preset(LintPreset::Opinionated).with_enabled_rules(Some(vec![RULE.into()]));
    let result = lint_sfc_with_corsa(&linter, form, path.to_str().expect("UTF-8 path"));
    let mut actual = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .map(|diagnostic| {
            let (line, column) = line_column(form, diagnostic.start);
            let (end_line, end_column) = line_column(form, diagnostic.end);
            (
                line,
                column,
                end_line,
                end_column,
                diagnostic.message.to_string(),
            )
        })
        .collect::<Vec<_>>();
    actual.sort();
    let chip = |message: &str| message.replace("'Fruit'", "'Chip'");
    assert_eq!(
        actual,
        vec![
            (5, 8, 5, 12, chip(ARGUMENT)),
            (6, 10, 6, 17, chip(ASSIGNMENT)),
            (7, 9, 7, 10, chip(ARGUMENT)),
            (8, 6, 8, 7, chip(ARGUMENT)),
            (9, 7, 9, 23, chip(ASSIGNMENT)),
        ]
    );
}

/// `implements Basket` is a type node to this rule and a symbol to
/// `type/no-deprecated`; with both on, each still reads its own node.
#[test]
fn implemented_interfaces_type_as_type_nodes_beside_deprecation() {
    if !corsa_available() {
        return;
    }
    let source = r#"<script setup lang="ts">
enum Fruit {
  Apple,
}
/** @deprecated */
interface Basket {
  fruit: Fruit;
}
class Plain implements Basket {
  fruit = 1;
}
type Alias = { fruit: Fruit };
class Aliased implements Alias {
  accessor fruit = 1;
}
interface Defaulted<T = Fruit> {
  fruit: T;
}
class Fallback implements Defaulted {
  fruit = 1;
}
class Safe implements Basket {
  fruit = Fruit.Apple;
}
void [Plain, Aliased, Fallback, Safe];
</script>
"#;
    let linter = Linter::with_preset(LintPreset::Opinionated)
        .with_enabled_rules(Some(vec![RULE.into(), "type/no-deprecated".into()]));
    let result = lint_sfc_with_corsa(&linter, source, "Heritage.vue");
    let mut actual = result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let (line, column) = line_column(source, diagnostic.start);
            (line, column, diagnostic.rule_name.to_string())
        })
        .collect::<Vec<_>>();
    actual.sort();
    assert_eq!(
        actual,
        vec![
            (9, 24, "type/no-deprecated".to_string()),
            (10, 3, RULE.to_string()),
            (14, 3, RULE.to_string()),
            (20, 3, RULE.to_string()),
            (22, 23, "type/no-deprecated".to_string()),
        ]
    );
}

/// typescript-eslint reads mapped enum keys from the declaration of the
/// accessed symbol: its own annotation, in this file or another one. An alias
/// (`type Keyed`, `Record`) or an import binding is not a mapped type node.
#[test]
fn mapped_keys_come_from_the_accessed_declaration() {
    if !corsa_available() {
        return;
    }
    let keys = r#"export enum Fruit {
  Apple,
}
export declare const keyed: { [key in Fruit]: string };
export declare const holder: { keyed: { [key in Fruit]: string } };
"#;
    let component = r#"<script setup lang="ts">
import { Fruit, holder, keyed } from "./keys";

type Keyed = { [key in Fruit]: string };
declare const aliased: Keyed;
declare const record: Record<Fruit, string>;
declare const optional: { keyed?: { [key in Fruit]: string } };
function generic<K extends Fruit>(map: { [P in K]: string }, key: number) {
  return map[key];
}
keyed[0];
holder.keyed[0];
aliased[0];
record[0];
optional.keyed?.[0];
holder.keyed[Fruit.Apple];
void generic;
</script>
"#;
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../target/vize-tests");
    std::fs::create_dir_all(&workspace).expect("test workspace");
    let project = tempfile::Builder::new()
        .prefix("enum-mapped-keys-13070-")
        .tempdir_in(workspace)
        .expect("temporary project");
    let source_dir = project.path().join("src");
    std::fs::create_dir(&source_dir).expect("source directory");
    std::fs::write(
        project.path().join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"Bundler","strict":true,"target":"ES2022","noEmit":true},"include":["src/**/*.ts","src/**/*.vue"]}"#,
    )
    .expect("tsconfig");
    std::fs::write(source_dir.join("keys.ts"), keys).expect("keys module");
    let path = source_dir.join("Keys.vue");
    std::fs::write(&path, component).expect("component");
    let linter =
        Linter::with_preset(LintPreset::Opinionated).with_enabled_rules(Some(vec![RULE.into()]));
    let result = lint_sfc_with_corsa(&linter, component, path.to_str().expect("UTF-8 path"));
    let mut actual = result
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.rule_name == RULE)
        .map(|diagnostic| {
            let (line, column) = line_column(component, diagnostic.start);
            let (end_line, end_column) = line_column(component, diagnostic.end);
            (
                line,
                column,
                end_line,
                end_column,
                diagnostic.message.to_string(),
            )
        })
        .collect::<Vec<_>>();
    actual.sort();
    assert_eq!(
        actual,
        vec![
            (9, 14, 9, 17, ACCESS.to_string()),
            (12, 14, 12, 15, ACCESS.to_string()),
            (15, 18, 15, 19, ACCESS.to_string()),
        ]
    );
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
