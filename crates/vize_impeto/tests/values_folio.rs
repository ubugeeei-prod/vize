use serde_json::json;
use vize_davinci::folio::value::FolioValue;
use vize_davinci::folio::{Folio, FolioError, FolioMode};
use vize_impeto::op::{OpId, Phase, Program, RegionId};
use vize_impeto::operand::{Operand, OperandRole, OperandValue, ValueKind};
use vize_impeto::values_folio::{FolioOperand, S3ValuesFolio};
use vize_s0::{Allocator, Span, String, cstr};

fn parse_row(json: &str, line: usize) -> Result<FolioOperand, FolioError> {
    FolioOperand::parse_value(&cstr!("operand={json}"), line)
}

#[test]
fn every_role_and_value_kind_round_trips_without_losing_escaped_text() {
    let arena = Allocator::default();
    let mut program = Program::new(&arena, Phase::Built);
    for role in OperandRole::ALL {
        for kind in ValueKind::ALL {
            let attribute = matches!(role, OperandRole::Attribute | OperandRole::ModelAttribute);
            program.operands.push(Operand {
                op: OpId::new(0),
                role,
                target: None,
                region: (role == OperandRole::Condition).then_some(RegionId::new(1)),
                name: attribute.then_some("data-\"key\\\n\u{03bb}"),
                value: OperandValue {
                    kind,
                    text: if kind == ValueKind::Absent {
                        ""
                    } else {
                        "\"\\\n\r\t\u{0}\u{03bb}\u{1f600}"
                    },
                    qualifier: match kind {
                        ValueKind::Opaque => "parse-failure",
                        ValueKind::Foreign => "moonbit",
                        _ => "",
                    },
                    span: Span::new(4, 19),
                },
            });
        }
    }
    let folio = S3ValuesFolio::of(&program);
    assert_eq!(folio.operands.len(), 108);
    let printed = folio.print_to_string(FolioMode::Full);
    let reparsed = S3ValuesFolio::parse(printed.as_str()).unwrap();
    assert_eq!(reparsed, folio);
    assert_eq!(reparsed.print_to_string(FolioMode::Full), printed);
    for operand in folio.operands {
        let mut text = String::default();
        operand.print_value(&mut text).unwrap();
        assert!(text.starts_with("operand=["));
        assert_eq!(text.lines().count(), 1);
    }
}

#[test]
fn malformed_rows_fail_on_the_authored_line() {
    let valid = json!([0, "text", null, null, null, "literal", "hello", "", 0, 5]);
    for (index, value) in [
        (0, json!(-1)),
        (1, json!("unknown")),
        (3, json!(1)),
        (4, json!("unexpected-name")),
        (5, json!("unknown")),
        (5, json!("absent")),
        (5, json!("opaque")),
        (5, json!("foreign")),
        (7, json!("unexpected-qualifier")),
        (8, json!(6)),
    ] {
        let mut row = valid.clone();
        row[index] = value;
        let error = parse_row(&serde_json::to_string(&row).unwrap(), 17).unwrap_err();
        assert_eq!(error.line, 17);
    }
    for text in [
        "[]",
        "[0]",
        "not json",
        "[0,\"text\",null,null,null,\"literal\",\"x\",\"\",0,1,2]",
    ] {
        assert_eq!(parse_row(text, 23).unwrap_err().line, 23);
    }
    for row in [
        json!([0, "attribute", null, null, null, "literal", "", "", 0, 0]),
        json!([0, "condition", null, null, null, "literal", "", "", 0, 0]),
        json!([0, "condition", 1, 2, null, "literal", "", "", 0, 0]),
    ] {
        assert!(parse_row(&serde_json::to_string(&row).unwrap(), 1).is_err());
    }
}

#[test]
fn absent_and_empty_literal_values_remain_distinct() {
    let absent = parse_row("[0,\"value\",null,null,null,\"absent\",\"\",\"\",0,0]", 1).unwrap();
    let literal = parse_row("[0,\"value\",null,null,null,\"literal\",\"\",\"\",0,0]", 1).unwrap();
    assert_ne!(absent, literal);
}
