//! Narrow receiver ownership, independent of a callback's contextual contract.
//!
//! Both typed callback bodies remain emitted. Only an unchanged static receiver
//! in a single-return, parameterless block can have a second diagnostic owner.
//! Expression bodies are excluded: contextual TS2322 can start at their receiver.

use super::{DiagnosticOwnership, InputOwner, InputRole, Projection};
use crate::virtual_ts::script_facts::ScriptBindings;
use oxc_ast::ast::{Expression, ObjectPropertyKind, PropertyKind, Statement};
use oxc_span::GetSpan;
use std::ops::Range;
use vize_carton::{FxHashSet, String};
use vize_croquis::{
    BindingType, Croquis, ScopeKind,
    croquis::{ComponentUsage, PassedProp},
};

pub(crate) struct ReadCandidate {
    range: Range<usize>,
    root: String,
}

pub(crate) struct ReadValue {
    pub(crate) raw: String,
    reads: Vec<ReadCandidate>,
}

#[derive(Clone, Debug)]
pub(super) struct PreparedReads {
    raw: String,
    value_start: u32,
    reads: Vec<Range<usize>>,
}

pub(super) fn collect_candidates(expression: &Expression<'_>, raw: &str) -> Option<ReadValue> {
    if !matches!(
        expression,
        Expression::ObjectExpression(_) | Expression::ArrayExpression(_)
    ) {
        return None;
    }
    let mut reads = Vec::new();
    if !collect(expression, raw, &mut reads) {
        return None;
    }
    (!reads.is_empty()).then(|| ReadValue {
        raw: String::from(raw),
        reads,
    })
}

fn collect(expression: &Expression<'_>, raw: &str, reads: &mut Vec<ReadCandidate>) -> bool {
    match expression {
        Expression::ObjectExpression(object) => object.properties.iter().all(|property| {
            let ObjectPropertyKind::ObjectProperty(property) = property else {
                return false;
            };
            property.kind == PropertyKind::Init
                && !property.computed
                && !property.method
                && collect(&property.value, raw, reads)
        }),
        Expression::ArrayExpression(array) => array.elements.iter().all(|element| {
            element
                .as_expression()
                .is_some_and(|expression| collect(expression, raw, reads))
        }),
        Expression::ArrowFunctionExpression(arrow)
            if !arrow.expression
                && !arrow.r#async
                && arrow.params.items.is_empty()
                && arrow.params.rest.is_none()
                && arrow.type_parameters.is_none()
                && arrow.return_type.is_none()
                && arrow.body.directives.is_empty() =>
        {
            let [Statement::ReturnStatement(statement)] = arrow.body.statements.as_slice() else {
                return false;
            };
            let Some(Expression::StaticMemberExpression(member)) = &statement.argument else {
                return false;
            };
            let Some((root, root_span)) = static_root(&member.object) else {
                return false;
            };
            // Retained coordinates must describe these exact display bytes.
            if raw.get(root_span) != Some(root) {
                return false;
            }
            let span = member.object.span();
            let range = span.start as usize..span.end as usize;
            if raw.get(range.clone()).is_none() {
                return false;
            }
            reads.push(ReadCandidate {
                range,
                root: String::from(root),
            });
            true
        }
        Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::StringLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::Identifier(_) => true,
        // No calls, spreads, methods, mutation or additional function scopes.
        _ => false,
    }
}

fn static_root<'a>(expression: &'a Expression<'_>) -> Option<(&'a str, Range<usize>)> {
    match expression {
        Expression::Identifier(identifier) => Some((
            identifier.name.as_str(),
            identifier.span.start as usize..identifier.span.end as usize,
        )),
        Expression::StaticMemberExpression(member) if !member.optional => {
            static_root(&member.object)
        }
        Expression::ComputedMemberExpression(member)
            if !member.optional
                && matches!(
                    member.expression,
                    Expression::NumericLiteral(_) | Expression::StringLiteral(_)
                ) =>
        {
            static_root(&member.object)
        }
        Expression::TSNonNullExpression(value) => static_root(&value.expression),
        _ => None,
    }
}

/// Require the actual immutable script declaration and reject every accessible
/// lexical shadow, mutation or replayed closure. This is a binding proof, not
/// an assumption made from an identifier's spelling or two equal inferred types.
fn script_declaration(
    summary: &Croquis,
    bindings: ScriptBindings<'_>,
    usage: &ComponentUsage,
    name: &str,
) -> Option<(u32, u32)> {
    if usage.vif_guard.is_some()
        || !bindings.is_script_setup()
        || bindings.binding_type(name) != Some(BindingType::SetupConst)
        || name.starts_with("__")
        || name.starts_with('$')
    {
        return None;
    }
    let (start, end) = bindings.span(name)?;
    if end.checked_sub(start)? as usize != name.len() {
        return None;
    }
    let mut pending = vec![usage.scope_id];
    let mut seen = FxHashSet::default();
    let mut declaration = None;
    while let Some(id) = pending.pop() {
        if !seen.insert(id) {
            continue;
        }
        let scope = summary.scopes.get_scope(id)?;
        if scope.id != id
            || matches!(
                scope.kind,
                ScopeKind::VFor
                    | ScopeKind::VSlot
                    | ScopeKind::VMatch
                    | ScopeKind::VWhen
                    | ScopeKind::Callback
                    | ScopeKind::Closure
                    | ScopeKind::Function
                    | ScopeKind::EventHandler
                    | ScopeKind::Block
            )
        {
            return None;
        }
        if let Some(binding) = scope.get_binding(name) {
            if scope.kind != ScopeKind::ScriptSetup
                || binding.binding_type != BindingType::SetupConst
                || binding.is_mutated()
                || binding.declaration_offset != start
            {
                return None;
            }
            let actual = (id.as_u32(), binding.declaration_offset);
            if declaration.is_some_and(|previous| previous != actual) {
                return None;
            }
            declaration = Some(actual);
        }
        pending.extend(scope.parents.iter().copied());
    }
    declaration
}

fn independent_owner(
    owner: InputOwner,
    value_start: u32,
    range: &Range<usize>,
) -> Option<InputOwner> {
    Some(InputOwner {
        start: value_start.checked_add(u32::try_from(range.start).ok()?)?,
        end: value_start.checked_add(u32::try_from(range.end).ok()?)?,
        role: InputRole::IndependentRead,
        ..owner
    })
}

impl DiagnosticOwnership {
    /// A proven root callback buffer keeps only its own usage's ephemeral facts.
    pub(crate) fn inherit_independent_reads(&mut self, from: &Self, usage: &ComponentUsage) {
        for prop in &usage.props {
            if let Some(owner) = InputOwner::for_expected_projection(usage, prop)
                && !from.unproven.contains(&owner)
                && let Some(value) = from.independent_reads.get(&owner)
            {
                self.independent_reads.insert(owner, value.clone());
            }
        }
    }
    // Keep the base owner compact for every ordinary scalar projection.
    // A late refusal of a whole prop must also revoke its narrow read proofs.
    pub(super) fn refuse_independent_reads(&mut self, owner: InputOwner) {
        if let Some(value) = self
            .independent_reads
            .get(&owner.with_role(InputRole::Value))
        {
            for range in &value.reads {
                if let Some(read_owner) = independent_owner(owner, value.value_start, range) {
                    self.unproven.insert(read_owner.with_role(InputRole::Value));
                }
            }
        }
    }
    pub(in crate::virtual_ts) fn prepare_independent_reads(
        &mut self,
        summary: &Croquis,
        bindings: ScriptBindings<'_>,
        usage: &ComponentUsage,
        prop: &PassedProp,
        value: &ReadValue,
        value_start: u32,
    ) {
        let Some(owner) = InputOwner::for_expected_projection(usage, prop) else {
            return;
        };
        if prop
            .value
            .as_ref()
            .is_none_or(|raw| raw.as_str() != value.raw.as_str())
        {
            return;
        }
        let reads = value
            .reads
            .iter()
            .map(|read| {
                script_declaration(summary, bindings, usage, read.root.as_str())
                    .map(|_| read.range.clone())
            })
            .collect::<Option<Vec<_>>>();
        let Some(reads) = reads else {
            return;
        };
        if !reads.is_empty() {
            self.independent_reads.insert(
                owner,
                PreparedReads {
                    raw: value.raw.clone(),
                    value_start,
                    reads,
                },
            );
        }
    }

    pub(crate) fn record_independent_receivers(
        &mut self,
        usage: &ComponentUsage,
        prop: &PassedProp,
        emitted: &str,
        generated: Range<usize>,
        authored: Range<usize>,
        inference: bool,
    ) {
        let Some(owner) = InputOwner::for_expected_projection(usage, prop) else {
            return;
        };
        if self.unproven.contains(&owner) {
            return;
        }
        let Some(value) = self.independent_reads.get(&owner) else {
            return;
        };
        // No searching, token normalization or offset guessing after a rewrite.
        if emitted != value.raw.as_str()
            || generated.len() != emitted.len()
            || authored.len() != emitted.len()
        {
            return;
        }
        for range in &value.reads {
            let Some(read_owner) = independent_owner(owner, value.value_start, range) else {
                continue;
            };
            self.rows.push(Projection {
                owner: read_owner,
                generated: generated.start + range.start..generated.start + range.end,
                authored: authored.start + range.start..authored.start + range.end,
                inference,
                suppress: true,
                inert: false,
                valid: true,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    const SCRIPT: &str = "import Child from './Child.vue'; const items: {label:string}[]=[]; const enabled=true; const rows=[];";
    const VALUE: &str = "{run:()=>{return items[0].label;}}";

    fn echoes(script: &str, template: &str, rewrite: bool) -> Vec<super::super::DiagnosticEcho> {
        let allocator = vize_carton::Allocator::new();
        let (root, errors) = vize_armature::parse(&allocator, template);
        assert!(errors.is_empty(), "{errors:?}");
        let mut analyzer =
            vize_croquis::Analyzer::with_options(vize_croquis::AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();
        let facts = super::super::contextual_prop_facts(&root);
        let mut ledger = DiagnosticOwnership::default();
        crate::virtual_ts::script_facts::with_bindings(&summary, |bindings| {
            for usage in vize_croquis::facts::component_usage_list(&summary) {
                for prop in &usage.props {
                    if let Some(value) = facts.reads.get(&(prop.start, prop.end)) {
                        ledger
                            .prepare_independent_reads(&summary, bindings, &usage, prop, value, 0);
                        let text = if rewrite {
                            "{run:()=>{return other[0].label;}}"
                        } else {
                            value.raw.as_str()
                        };
                        let length = text.len();
                        ledger.record_independent_receivers(
                            &usage,
                            prop,
                            text,
                            0..length,
                            0..length,
                            false,
                        );
                        ledger.record_independent_receivers(
                            &usage,
                            prop,
                            text,
                            length + 1..2 * length + 1,
                            0..length,
                            true,
                        );
                    }
                }
            }
        });
        ledger.into_echoes()
    }

    #[test]
    fn actual_script_declaration_owns_only_the_repeated_static_receiver() {
        let template = format!("<Child :options=\"{VALUE}\" />");
        let result = echoes(SCRIPT, &template, false);
        assert_eq!(
            result.len(),
            1,
            "real ScriptBindings and ScopeChain must agree"
        );
        let start = VALUE.find("items[0]").unwrap();
        assert_eq!(result[0].authored, start..start + "items[0]".len());
        assert_eq!(
            result[0].generated,
            VALUE.len() + 1 + start..VALUE.len() + 1 + start + "items[0]".len()
        );
        assert!(result[0].suppress);
        assert!(
            echoes(SCRIPT, &template, true).is_empty(),
            "rewritten display bytes are not the same owner"
        );
    }

    #[test]
    fn contextual_and_lexical_boundaries_refuse_receiver_ownership() {
        for value in [
            "{run:()=>items[0].label}",
            "{run:():string=>{return items[0].label;}}",
            "{run:(items)=>{return items[0].label;}}",
            "{run:(items)=>()=>{return items[0].label;}}",
            "{run:()=>{const items=[];return items[0].label;}}",
            "{run:()=>{return call(items[0].label);}}",
            "{run:()=>{return this.items[0].label;}}",
            "{run:()=>{return items[next()].label;}}",
            "{run:async()=>{return items[0].label;}}",
            "{run:()=>{return absent[0].label;}}",
            "{run:()=>{return items[0].label;}, other:call()}",
            "{run:()=>{return items[0].label;}, other:(items)=>items[0]}",
        ] {
            assert!(
                echoes(SCRIPT, &format!("<Child :options=\"{value}\" />"), false).is_empty(),
                "{value}"
            );
        }
        for template in [
            format!("<div v-for=\"items in rows\"><Child :options=\"{VALUE}\" /></div>"),
            format!("<Child v-if=\"enabled\" :options=\"{VALUE}\" />"),
            format!("<Child v-slot=\"{{ items }}\"><Child :options=\"{VALUE}\" /></Child>"),
        ] {
            assert!(echoes(SCRIPT, &template, false).is_empty(), "{template}");
        }
        assert!(
            echoes(
                &SCRIPT.replace("const items", "let items"),
                &format!("<Child :options=\"{VALUE}\" />"),
                false
            )
            .is_empty()
        );
    }
    #[test]
    fn a_late_whole_prop_refusal_also_revokes_receiver_proofs() {
        let owner = InputOwner {
            usage: 1,
            scope: 0,
            start: 2,
            end: 20,
            role: InputRole::Value,
        };
        let range = 0..1;
        let read = independent_owner(owner, 4, &range).unwrap();
        let mut ledger = DiagnosticOwnership::default();
        ledger.independent_reads.insert(
            owner,
            PreparedReads {
                raw: String::from("x"),
                value_start: 4,
                reads: vec![range],
            },
        );
        ledger.record(read, 0..1, 4..5, false);
        ledger.record(read, 2..3, 4..5, true);
        ledger.refuse_owner(owner);
        assert!(ledger.into_echoes().is_empty());
    }
}
