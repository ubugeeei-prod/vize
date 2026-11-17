//! #12994: qualified enum flows match typescript-eslint without changing safe flows.
//! Remaining checker-surface-dependent flows are tracked in #13070.
use super::{corsa_available, lint_sfc_with_corsa};
use crate::diagnostic::Severity;
use crate::{LintPreset, Linter};

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
            packet(
                (40, 11),
                (40, 12),
                "The computed key used here does not have a shared enum type with the expected enum 'Fruit'.",
            ),
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
