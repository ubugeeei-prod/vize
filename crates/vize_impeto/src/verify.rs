//! The Impeto phase validator.
//!
//! The validator is whole-artifact rather than local: S3 is a graph, so edge
//! resolution and effect scopes are artifact invariants. It still does no
//! inference and no repair. Invalid S3 is rejected exactly where it stands.

use alloc::vec::Vec;

use vize_s0::{Span, cstr};

use crate::op::{Phase, Program, RegionId};
use lookup::{
    Tables, contains, effect_scope, op, op_index, op_inside_region, region, region_has_cycle,
    region_is_or_descendant,
};

mod lookup;
mod operands;
mod placement;
mod violation;

pub use violation::{Violation, ViolationCode};

/// Validate all invariants required by the artifact's current phase.
#[must_use]
pub fn verify(program: &Program<'_>) -> Vec<Violation> {
    let program = &Tables::new(program);
    let mut out = Vec::new();
    check_duplicates(program, &mut out);
    check_root(program, &mut out);
    check_regions(program, &mut out);
    check_ops(program, &mut out);
    check_effects(program, &mut out);
    check_edges(program, &mut out);
    operands::check(program, &mut out);
    placement::check(program, &mut out);
    out
}

fn check_duplicates(program: &Tables<'_, '_>, out: &mut Vec<Violation>) {
    // Dense tables cannot repeat an id, so only the others are scanned.
    for (index, op) in (program.ops.iter().enumerate()).filter(|_| !program.dense_ops) {
        if program.ops[..index].iter().any(|other| other.id == op.id) {
            out.push(Violation {
                code: ViolationCode::DuplicateId,
                span: op.span,
                message: cstr!("duplicate op id {}", op.id),
            });
        }
    }
    for (index, region) in (program.regions.iter().enumerate()).filter(|_| !program.dense_regions) {
        if program.regions[..index]
            .iter()
            .any(|other| other.id == region.id)
        {
            out.push(Violation {
                code: ViolationCode::DuplicateId,
                span: region.span,
                message: cstr!("duplicate region id {}", region.id),
            });
        }
    }
    for (index, effect) in (program.effects.iter().enumerate()).filter(|_| !program.dense_effects) {
        if program.effects[..index]
            .iter()
            .any(|other| other.id == effect.id)
        {
            out.push(Violation {
                code: ViolationCode::DuplicateId,
                span: effect.span,
                message: cstr!("duplicate effect id {}", effect.id),
            });
        }
    }
}

fn check_root(program: &Tables<'_, '_>, out: &mut Vec<Violation>) {
    let Some(root) = region(program, RegionId::ROOT) else {
        out.push(Violation {
            code: ViolationCode::RootRegion,
            span: Span::new(0, 0),
            message: cstr!("Impeto program does not define root region r#0"),
        });
        return;
    };
    if root.parent.is_some() || root.owner.is_some() {
        out.push(Violation {
            code: ViolationCode::RootRegion,
            span: root.span,
            message: cstr!("root region r#0 must not have a parent or owner"),
        });
    }
}

fn check_regions(program: &Tables<'_, '_>, out: &mut Vec<Violation>) {
    for item in &program.regions {
        if item.id != RegionId::ROOT && (item.parent.is_none() || item.owner.is_none()) {
            out.push(Violation {
                code: ViolationCode::RegionResolution,
                span: item.span,
                message: cstr!(
                    "non-root region {} must name both parent and owner",
                    item.id
                ),
            });
        }
        if let Some(parent) = item.parent {
            match region(program, parent) {
                Some(parent_region) if !contains(parent_region.span, item.span) => {
                    out.push(Violation {
                        code: ViolationCode::RegionNesting,
                        span: item.span,
                        message: cstr!("region {} escapes parent region {}", item.id, parent),
                    });
                }
                Some(_) => {}
                None => out.push(Violation {
                    code: ViolationCode::RegionResolution,
                    span: item.span,
                    message: cstr!("parent region {} for {} does not resolve", parent, item.id),
                }),
            }
        }
        if let Some(owner) = item.owner {
            match op(program, owner) {
                Some(owner_op) if Some(owner_op.region) != item.parent => {
                    out.push(Violation {
                        code: ViolationCode::RegionNesting,
                        span: item.span,
                        message: cstr!(
                            "region {} is owned by {}, but the owner lives in {}",
                            item.id,
                            owner,
                            owner_op.region
                        ),
                    });
                }
                Some(owner_op) if !contains(owner_op.span, item.span) => {
                    out.push(Violation {
                        code: ViolationCode::RegionNesting,
                        span: item.span,
                        message: cstr!("region {} escapes owner {}", item.id, owner),
                    });
                }
                Some(_) => {}
                None => out.push(Violation {
                    code: ViolationCode::RegionResolution,
                    span: item.span,
                    message: cstr!("owner op {} for {} does not resolve", owner, item.id),
                }),
            }
        }
        if region_has_cycle(program, item.id) {
            out.push(Violation {
                code: ViolationCode::RegionNesting,
                span: item.span,
                message: cstr!("region {} participates in a parent cycle", item.id),
            });
        }
    }
}

fn check_ops(program: &Tables<'_, '_>, out: &mut Vec<Violation>) {
    for item in &program.ops {
        match region(program, item.region) {
            Some(owner) if !contains(owner.span, item.span) => {
                out.push(Violation {
                    code: ViolationCode::RegionNesting,
                    span: item.span,
                    message: cstr!("{} escapes containing region {}", item.id, item.region),
                });
            }
            Some(_) => {}
            None => out.push(Violation {
                code: ViolationCode::OpRegion,
                span: item.span,
                message: cstr!("{} references missing region {}", item.id, item.region),
            }),
        }
        if let Some(effect) = item.effect
            && effect_scope(program, effect).is_none()
        {
            out.push(Violation {
                code: ViolationCode::EffectScope,
                span: item.span,
                message: cstr!("{} references missing effect scope {}", item.id, effect),
            });
        }
    }
}

fn check_effects(program: &Tables<'_, '_>, out: &mut Vec<Violation>) {
    for item in &program.effects {
        let owner = op(program, item.owner);
        let scope_region = region(program, item.region);
        match (owner, scope_region) {
            (Some(owner), Some(region)) => {
                if !region_is_or_descendant(program, owner.region, item.region) {
                    out.push(Violation {
                        code: ViolationCode::EffectScope,
                        span: item.span,
                        message: cstr!(
                            "effect {} owner {} is outside region {}",
                            item.id,
                            item.owner,
                            item.region
                        ),
                    });
                }
                if !contains(region.span, item.span) {
                    out.push(Violation {
                        code: ViolationCode::EffectScope,
                        span: item.span,
                        message: cstr!("effect {} span escapes region {}", item.id, item.region),
                    });
                }
            }
            (None, _) => out.push(Violation {
                code: ViolationCode::EffectScope,
                span: item.span,
                message: cstr!("effect {} references missing owner {}", item.id, item.owner),
            }),
            (_, None) => out.push(Violation {
                code: ViolationCode::EffectScope,
                span: item.span,
                message: cstr!(
                    "effect {} references missing region {}",
                    item.id,
                    item.region
                ),
            }),
        }
    }
}

fn check_edges(program: &Tables<'_, '_>, out: &mut Vec<Violation>) {
    for edge in &program.edges {
        let from = op(program, edge.from);
        let to = op(program, edge.to);
        if from.is_none() {
            out.push(Violation {
                code: ViolationCode::EdgeEndpoint,
                span: Span::new(0, 0),
                message: cstr!("state edge source {} does not resolve", edge.from),
            });
        }
        if to.is_none() {
            out.push(Violation {
                code: ViolationCode::EdgeEndpoint,
                span: Span::new(0, 0),
                message: cstr!("state edge target {} does not resolve", edge.to),
            });
        }
        if let Some(scope_id) = edge.effect {
            match effect_scope(program, scope_id) {
                Some(scope) => {
                    if let (Some(from), Some(to)) = (from, to)
                        && (!op_inside_region(program, from.id, scope.region)
                            || !op_inside_region(program, to.id, scope.region))
                    {
                        out.push(Violation {
                            code: ViolationCode::EffectScope,
                            span: scope.span,
                            message: cstr!(
                                "edge {} -> {} leaves effect scope {}",
                                edge.from,
                                edge.to,
                                scope_id
                            ),
                        });
                    }
                }
                None => out.push(Violation {
                    code: ViolationCode::EffectScope,
                    span: Span::new(0, 0),
                    message: cstr!("state edge references missing effect scope {}", scope_id),
                }),
            }
        }
        if program.phase == Phase::Scheduled
            && let (Some(from_index), Some(to_index)) =
                (op_index(program, edge.from), op_index(program, edge.to))
            && from_index >= to_index
        {
            out.push(Violation {
                code: ViolationCode::ScheduledOrder,
                span: Span::new(0, 0),
                message: cstr!(
                    "scheduled edge {} -> {} points backward or to itself",
                    edge.from,
                    edge.to
                ),
            });
        }
    }
}
