//! The nested static-bind-prop scans `can_hoist_static_props` consults.
//!
//! Split out of `call_props.rs` to keep both files inside the source
//! budget; the walks themselves are unchanged.

use vize_s2::op::{ComponentOp, Op, Region};

use super::super::super::{EmitError, props_static};

pub(super) fn has_nested_for_component_static_bind_props(
    region: &Region<'_>,
    is_ts: bool,
) -> Result<bool, EmitError> {
    for op in &region.ops {
        let found = match op {
            Op::Element(element) => {
                has_nested_for_component_static_bind_props(&element.children, is_ts)?
            }
            Op::Component(component) => {
                has_nested_for_component_static_bind_props(&component.children, is_ts)?
            }
            Op::If(if_op) => {
                let mut found = false;
                for branch in &if_op.branches {
                    found |= has_nested_for_component_static_bind_props(&branch.region, is_ts)?;
                }
                found
            }
            Op::For(for_op) => region_has_component_static_bind_props(&for_op.region, is_ts)?,
            Op::Slot(slot) => has_nested_for_component_static_bind_props(&slot.fallback, is_ts)?,
            Op::Text(_) | Op::Interpolation(_) => false,
        };
        if found {
            return Ok(true);
        }
    }
    Ok(false)
}

fn region_has_component_static_bind_props(
    region: &Region<'_>,
    is_ts: bool,
) -> Result<bool, EmitError> {
    for op in &region.ops {
        let found = match op {
            Op::Element(element) => {
                region_has_component_static_bind_props(&element.children, is_ts)?
            }
            Op::Component(component) => {
                component_has_static_bind_props(component, is_ts)?
                    || region_has_component_static_bind_props(&component.children, is_ts)?
            }
            Op::If(if_op) => {
                let mut found = false;
                for branch in &if_op.branches {
                    found |= region_has_component_static_bind_props(&branch.region, is_ts)?;
                }
                found
            }
            Op::For(for_op) => region_has_component_static_bind_props(&for_op.region, is_ts)?,
            Op::Slot(slot) => region_has_component_static_bind_props(&slot.fallback, is_ts)?,
            Op::Text(_) | Op::Interpolation(_) => false,
        };
        if found {
            return Ok(true);
        }
    }
    Ok(false)
}

fn component_has_static_bind_props(
    component: &ComponentOp<'_>,
    is_ts: bool,
) -> Result<bool, EmitError> {
    Ok(
        props_static::component_hoist_props(&component.attributes, &component.bindings, is_ts)?
            .is_some_and(|props| props.all_static_binds && props.valued_prop),
    )
}
