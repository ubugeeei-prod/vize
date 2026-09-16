use vize_impeto::op::{Op, OpId, OpKind, Phase, Program, Region, RegionId};
use vize_impeto::operand::{Operand, OperandRole as Role, OperandValue, ValueKind};
use vize_impeto::verify::{ViolationCode, verify};
use vize_s0::{Allocator, Span, cstr};

fn program(arena: &Allocator) -> Program<'_> {
    let mut program = Program::new(arena, Phase::Built);
    program.push_region(Region::root(Span::new(0, 100)));
    for (id, kind, span) in [
        (0, OpKind::If, Span::new(0, 100)),
        (1, OpKind::InsertNode, Span::new(10, 90)),
        (2, OpKind::SetProp, Span::new(20, 30)),
    ] {
        program.push_op(Op::new(OpId::new(id), kind, RegionId::ROOT, span));
    }
    program.push_region(Region::child(
        RegionId::new(1),
        RegionId::ROOT,
        OpId::new(0),
        Span::new(40, 90),
    ));
    program
}

fn binding() -> Operand<'static> {
    Operand {
        op: OpId::new(2),
        role: Role::Value,
        target: Some(OpId::new(1)),
        region: None,
        name: None,
        value: OperandValue {
            kind: ValueKind::Js,
            text: "value",
            qualifier: "",
            span: Span::new(21, 26),
        },
    }
}

fn rejects(operand: Operand<'_>, message: &str) {
    let arena = Allocator::default();
    let mut program = program(&arena);
    program.operands.push(operand);
    let violations = verify(&program);
    assert_eq!(violations.len(), 1, "{violations:?}");
    assert_eq!(violations[0].code, ViolationCode::Operand);
    assert_eq!(violations[0].span, operand.value.span);
    assert_eq!(
        violations[0].message,
        cstr!(
            "operand {} for {}: {message}",
            operand.role.as_str(),
            operand.op
        )
    );
}

#[test]
fn valid_binding_and_branch_references_are_accepted_in_every_phase() {
    let arena = Allocator::default();
    let mut program = program(&arena);
    program.operands.push(binding());
    program.operands.push(Operand {
        op: OpId::new(0),
        role: Role::Condition,
        target: None,
        region: Some(RegionId::new(1)),
        ..binding()
    });
    for phase in [Phase::Built, Phase::Partitioned, Phase::Scheduled] {
        program.phase = phase;
        assert_eq!(verify(&program), []);
    }
}

#[test]
fn operand_owners_targets_and_spans_must_resolve() {
    rejects(
        Operand {
            op: OpId::new(99),
            ..binding()
        },
        "owner does not resolve",
    );
    rejects(
        Operand {
            target: Some(OpId::new(99)),
            ..binding()
        },
        "target does not resolve",
    );
    for target in [0, 2] {
        rejects(
            Operand {
                target: Some(OpId::new(target)),
                ..binding()
            },
            "target is not a containing materialized operation in the same region",
        );
    }
    rejects(
        Operand {
            value: OperandValue {
                span: Span::new(19, 26),
                ..binding().value
            },
            ..binding()
        },
        "value span escapes owner",
    );
    for value in [
        OperandValue {
            span: Span::new(26, 21),
            ..binding().value
        },
        OperandValue {
            kind: ValueKind::Absent,
            ..binding().value
        },
        OperandValue {
            kind: ValueKind::Opaque,
            ..binding().value
        },
        OperandValue {
            kind: ValueKind::Foreign,
            ..binding().value
        },
        OperandValue {
            qualifier: "unexpected",
            ..binding().value
        },
    ] {
        rejects(Operand { value, ..binding() }, "malformed value");
    }
}

#[test]
fn attributes_and_conditions_enforce_their_role_contracts() {
    rejects(
        Operand {
            name: Some("id"),
            ..binding()
        },
        "attribute name must occur exactly on attribute roles",
    );
    rejects(
        Operand {
            role: Role::Attribute,
            ..binding()
        },
        "attribute name must occur exactly on attribute roles",
    );
    rejects(
        Operand {
            region: Some(RegionId::new(1)),
            ..binding()
        },
        "only condition operands may reference a branch",
    );
    let condition = Operand {
        op: OpId::new(0),
        role: Role::Condition,
        target: None,
        region: Some(RegionId::new(1)),
        ..binding()
    };
    for region in [None, Some(RegionId::new(99))] {
        rejects(
            Operand {
                region,
                ..condition
            },
            "condition branch does not resolve",
        );
    }
    rejects(
        Operand {
            op: OpId::new(2),
            ..condition
        },
        "condition branch must be owned by its if operation",
    );
    rejects(
        Operand {
            region: Some(RegionId::ROOT),
            ..condition
        },
        "condition branch must be owned by its if operation",
    );
}

#[test]
fn every_role_enforces_its_target_policy_in_every_phase() {
    for role in Role::ALL {
        for present in [false, true] {
            let expected = match role {
                Role::BindingKind
                | Role::Value
                | Role::Modifier
                | Role::ModelRead
                | Role::ModelWrite
                | Role::ModelAttribute
                | Role::Params => present,
                Role::Tag | Role::Name => true,
                _ => !present,
            };
            let arena = Allocator::default();
            let mut program = program(&arena);
            let condition = role == Role::Condition;
            program.operands.push(Operand {
                op: if condition {
                    OpId::new(0)
                } else {
                    OpId::new(2)
                },
                role,
                target: present.then_some(OpId::new(1)),
                region: condition.then_some(RegionId::new(1)),
                name: matches!(role, Role::Attribute | Role::ModelAttribute).then_some("id"),
                ..binding()
            });
            for phase in [Phase::Built, Phase::Partitioned, Phase::Scheduled] {
                program.phase = phase;
                let violations = verify(&program);
                if expected {
                    assert_eq!(
                        violations,
                        [],
                        "{role:?}/{present}/{phase:?}: {violations:?}"
                    );
                } else {
                    assert_eq!(violations.len(), 1, "{violations:?}");
                    assert_eq!(violations[0].code, ViolationCode::Operand);
                    assert_eq!(
                        violations[0].message,
                        cstr!(
                            "operand {} for {}: target presence does not match operand role",
                            role.as_str(),
                            program.operands[0].op
                        )
                    );
                }
            }
        }
    }
}
