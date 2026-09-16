use alloc::vec::Vec;
use vize_s0::cstr;

use crate::op::{OpKind, Program};
use crate::operand::{Operand, OperandRole};

use super::lookup::{contains, op, region};
use super::{Violation, ViolationCode};

pub(super) fn check(program: &Program<'_>, out: &mut Vec<Violation>) {
    for operand in &program.operands {
        if let Some(message) = invalid(program, operand) {
            out.push(Violation {
                code: ViolationCode::Operand,
                span: operand.value.span,
                message: cstr!(
                    "operand {} for {}: {message}",
                    operand.role.as_str(),
                    operand.op
                ),
            });
        }
    }
}

fn invalid(program: &Program<'_>, operand: &Operand<'_>) -> Option<&'static str> {
    let Some(owner) = op(program, operand.op) else {
        return Some("owner does not resolve");
    };
    if !operand.value.is_well_formed() {
        return Some("malformed value");
    }
    if !contains(owner.span, operand.value.span) {
        return Some("value span escapes owner");
    }
    let attribute = matches!(
        operand.role,
        OperandRole::Attribute | OperandRole::ModelAttribute
    );
    if operand.name.is_some() != attribute {
        return Some("attribute name must occur exactly on attribute roles");
    }
    if let Some(target_id) = operand.target {
        let Some(target) = op(program, target_id) else {
            return Some("target does not resolve");
        };
        if target_id == owner.id
            || target.region != owner.region
            || !contains(target.span, owner.span)
            || !matches!(
                target.kind,
                OpKind::InsertNode | OpKind::CreateComponent | OpKind::SlotOutlet
            )
        {
            return Some("target is not a containing materialized operation in the same region");
        }
    }
    if operand.role == OperandRole::Condition {
        let Some(branch) = operand.region.and_then(|id| region(program, id)) else {
            return Some("condition branch does not resolve");
        };
        if owner.kind != OpKind::If
            || branch.owner != Some(owner.id)
            || branch.parent != Some(owner.region)
            || operand.target.is_some()
        {
            return Some("condition branch must be owned by its if operation");
        }
    } else if operand.region.is_some() {
        return Some("only condition operands may reference a branch");
    }
    None
}
