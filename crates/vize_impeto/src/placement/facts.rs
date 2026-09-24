//! Structural facts shared by the placement enumerator and the verifier.
//!
//! Ids resolve exactly like the phase validator: the first record with an id
//! wins, so duplicate ids (already S3V001) cannot make two readers disagree.
//! Every walk is bounded by the table it walks, so malformed parent or
//! effect-order cycles end in a refusal instead of a loop.

use alloc::vec::Vec;

use crate::op::{EdgeKind, Op, OpId, OpKind, Program, Region, RegionId};
use crate::operand::{Operand, OperandRole as Role, ValueKind};

/// Sorted lookup tables over one program.
pub(crate) struct Index<'p, 'a> {
    pub(crate) program: &'p Program<'a>,
    ops: Vec<(OpId, u32)>,
    regions: Vec<(RegionId, u32)>,
    operands: Vec<(OpId, u32)>,
    /// `(to, from)` for every effect-order edge.
    effect_preds: Vec<(OpId, OpId)>,
    /// `(owner, region)` for every owned region.
    owned: Vec<(OpId, RegionId)>,
}

fn positions<K: Ord + Copy, T>(items: &[T], key: impl Fn(&T) -> K) -> Vec<(K, u32)> {
    let mut table: Vec<(K, u32)> = items
        .iter()
        .enumerate()
        .map(|(position, item)| (key(item), position as u32))
        .collect();
    table.sort_by_key(|entry| entry.0);
    table
}

fn first<K: Ord + Copy, V: Copy>(table: &[(K, V)], key: K) -> Option<V> {
    let start = table.partition_point(|entry| entry.0 < key);
    table
        .get(start)
        .filter(|entry| entry.0 == key)
        .map(|entry| entry.1)
}

fn all<K: Ord + Copy, V: Copy>(table: &[(K, V)], key: K) -> impl Iterator<Item = V> + '_ {
    let start = table.partition_point(|entry| entry.0 < key);
    (table.get(start..).unwrap_or_default())
        .iter()
        .take_while(move |entry| entry.0 == key)
        .map(|entry| entry.1)
}

impl<'p, 'a> Index<'p, 'a> {
    pub(crate) fn new(program: &'p Program<'a>) -> Self {
        let mut effect_preds: Vec<(OpId, OpId)> = program
            .edges
            .iter()
            .filter(|edge| edge.kind == EdgeKind::EffectOrder)
            .map(|edge| (edge.to, edge.from))
            .collect();
        effect_preds.sort_by_key(|entry| entry.0);
        let mut owned: Vec<(OpId, RegionId)> = program
            .regions
            .iter()
            .filter_map(|region| region.owner.map(|owner| (owner, region.id)))
            .collect();
        owned.sort_by_key(|entry| entry.0);
        Self {
            program,
            ops: positions(&program.ops, |op| op.id),
            regions: positions(&program.regions, |region| region.id),
            operands: positions(&program.operands, |operand| operand.op),
            effect_preds,
            owned,
        }
    }

    pub(crate) fn position(&self, id: OpId) -> Option<usize> {
        first(&self.ops, id).map(|position| position as usize)
    }

    pub(crate) fn op(&self, id: OpId) -> Option<&'p Op> {
        self.program.ops.get(self.position(id)?)
    }

    pub(crate) fn region(&self, id: RegionId) -> Option<&'p Region> {
        self.program.regions.get(first(&self.regions, id)? as usize)
    }

    pub(crate) fn region_position(&self, id: RegionId) -> Option<usize> {
        first(&self.regions, id).map(|position| position as usize)
    }

    pub(crate) fn operands(&self, id: OpId) -> impl Iterator<Item = &'p Operand<'a>> + '_ {
        let operands = &self.program.operands;
        all(&self.operands, id).filter_map(move |position| operands.get(position as usize))
    }

    pub(crate) fn owned_regions(&self, owner: OpId) -> impl Iterator<Item = RegionId> + '_ {
        all(&self.owned, owner)
    }

    /// Kind of the op owning `region`: `Some(None)` for an ownerless region.
    fn owner_kind(&self, region: &Region) -> Option<Option<OpKind>> {
        match region.owner {
            None => Some(None),
            Some(owner) => self.op(owner).map(|op| Some(op.kind)),
        }
    }

    /// The nearest region at or above `region` that is not owned by an
    /// element. Element children render whenever the element does; every
    /// other owner can create, destroy, or re-render its regions on its own.
    pub(crate) fn effect_region(&self, region: RegionId) -> Option<&'p Region> {
        let mut current = self.region(region)?;
        for _ in 0..=self.program.regions.len() {
            if self.owner_kind(current)? != Some(OpKind::InsertNode) {
                return Some(current);
            }
            current = self.region(current.parent?)?;
        }
        None
    }

    /// Kind of the control op owning the effect region, if one does.
    pub(crate) fn controller(&self, region: RegionId) -> Option<OpKind> {
        let effect = self.effect_region(region)?;
        self.owner_kind(effect)?.filter(|kind| is_control(*kind))
    }

    /// The effect region every member of one group shares. Only the root,
    /// `if` branches, and `for` items qualify: each is one lexical scope, so
    /// an identical direct reference names the identical binding there. Slot
    /// and component content can mix scopes and never groups.
    pub(crate) fn group_scope(&self, region: RegionId) -> Option<RegionId> {
        let effect = self.effect_region(region)?;
        matches!(
            self.owner_kind(effect)?,
            None | Some(OpKind::If | OpKind::For)
        )
        .then_some(effect.id)
    }

    /// No region between `region` and the root can bind template scope.
    pub(crate) fn scope_free(&self, region: RegionId) -> bool {
        let Some(mut current) = self.region(region) else {
            return false;
        };
        for _ in 0..=self.program.regions.len() {
            match self.owner_kind(current) {
                Some(None) => return true,
                Some(Some(OpKind::InsertNode | OpKind::If)) => {}
                _ => return false,
            }
            let Some(parent) = current.parent.and_then(|parent| self.region(parent)) else {
                return false;
            };
            current = parent;
        }
        false
    }

    /// `region` is owned by `owner` or nested inside a region it owns.
    pub(crate) fn under(&self, region: RegionId, owner: OpId) -> bool {
        let mut current = self.region(region);
        for _ in 0..=self.program.regions.len() {
            let Some(item) = current else {
                return false;
            };
            if item.owner == Some(owner) {
                return true;
            }
            current = item.parent.and_then(|parent| self.region(parent));
        }
        false
    }

    /// The only effect-order predecessor of `op`.
    pub(crate) fn effect_pred(&self, op: OpId) -> Option<OpId> {
        let mut preds = all(&self.effect_preds, op);
        let pred = preds.next()?;
        preds.next().is_none().then_some(pred)
    }

    /// The nearest effect-order predecessor that reads reactive state.
    /// Keyless dynamic ops re-run only when their controlling region
    /// re-renders, where op order is unchanged by any placement.
    pub(crate) fn keyed_pred(&self, op: OpId) -> Option<OpId> {
        let mut current = self.effect_pred(op)?;
        for _ in 0..self.program.ops.len() {
            if self.operands(current).any(reads) {
                return Some(current);
            }
            current = self.effect_pred(current)?;
        }
        None
    }

    /// Static element, comment, or literal text with no instance identity.
    pub(crate) fn shape_static(&self, op: &Op) -> bool {
        let mut content = 0u32;
        for operand in self.operands(op.id) {
            if !is_literal(operand.value.kind)
                || operand.target.is_some()
                || operand.region.is_some()
            {
                return false;
            }
            match (op.kind, operand.role) {
                (OpKind::InsertNode, Role::Tag | Role::Comment) | (OpKind::SetText, Role::Text) => {
                    content += 1;
                }
                (OpKind::InsertNode, Role::Namespace) => {}
                (OpKind::InsertNode, Role::Attribute)
                    if !operand.name.is_some_and(is_instance_attribute) => {}
                _ => return false,
            }
        }
        content == 1
    }

    /// The single direct reference read by a groupable leaf update.
    pub(crate) fn reference(&self, op: &Op) -> Option<&'a str> {
        if !matches!(
            op.kind,
            OpKind::SetProp | OpKind::SetDynamicProps | OpKind::SetText | OpKind::SetHtml
        ) || op.effect.is_none()
        {
            return None;
        }
        let mut read = None;
        for operand in self.operands(op.id) {
            if let Some(target) = operand.target
                && self.op(target).map(|target| target.kind) != Some(OpKind::InsertNode)
            {
                return None;
            }
            let value = operand.value;
            match operand.role {
                Role::Value | Role::Text
                    if read.is_none()
                        && value.kind == ValueKind::Js
                        && is_direct_reference(value.text) =>
                {
                    read = Some(value.text);
                }
                Role::Name | Role::Modifier if is_literal(value.kind) => {}
                Role::BindingKind
                    if value.kind == ValueKind::Literal
                        && matches!(value.text, "bind" | "vue.text" | "vue.html") => {}
                _ => return None,
            }
        }
        read
    }

    /// A handler created once and reused behaves exactly like one re-bound on
    /// every update when it names a static event and reads no template scope.
    pub(crate) fn plain_handler(&self, op: &Op) -> bool {
        let mut handlers = 0u32;
        for operand in self.operands(op.id) {
            let value = operand.value;
            match operand.role {
                Role::Value if value.kind == ValueKind::Js && !value.text.is_empty() => {
                    handlers += 1;
                }
                Role::Name | Role::Modifier if value.kind == ValueKind::Literal => {}
                Role::BindingKind if value.kind == ValueKind::Literal && value.text == "on" => {}
                _ => return false,
            }
        }
        handlers == 1
    }
}

/// Owners that can create, destroy, or re-render their regions on their own.
pub(crate) const fn is_control(kind: OpKind) -> bool {
    matches!(
        kind,
        OpKind::If | OpKind::For | OpKind::SlotOutlet | OpKind::CreateComponent
    )
}

pub(crate) const fn is_literal(kind: ValueKind) -> bool {
    matches!(kind, ValueKind::Literal | ValueKind::Absent)
}

/// Whether `operand` reads reactive state when its op renders.
pub(crate) fn reads(operand: &Operand<'_>) -> bool {
    matches!(
        operand.role,
        Role::Value | Role::Text | Role::Condition | Role::ForSource | Role::ModelRead | Role::Name
    ) && !is_literal(operand.value.kind)
}

/// Attributes whose meaning is bound to one component instance.
fn is_instance_attribute(name: &str) -> bool {
    ["ref", "ref_for", "ref_key", "key", "is"]
        .iter()
        .any(|special| name.eq_ignore_ascii_case(special))
}

/// `name` or `name.member.member`: ASCII identifier segments only, so the
/// read has no call, operator, or assignment that could order effects.
pub(crate) fn is_direct_reference(text: &str) -> bool {
    !text.is_empty()
        && text.split('.').all(|segment| {
            let mut bytes = segment.bytes();
            bytes
                .next()
                .is_some_and(|byte| byte.is_ascii_alphabetic() || matches!(byte, b'_' | b'$'))
                && bytes.all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
        })
}

pub(crate) const fn contains(owner: vize_s0::Span, child: vize_s0::Span) -> bool {
    owner.start <= child.start && child.end <= owner.end
}
