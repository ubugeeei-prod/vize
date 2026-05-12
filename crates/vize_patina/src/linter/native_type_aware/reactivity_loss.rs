use super::{
    markers::marker_insert_offset, push_warning, LintResult, Linter, RULE_NO_REACTIVITY_LOSS,
};
use crate::diagnostic::LintDiagnostic;
use vize_carton::{cstr, CompactString, FxHashSet, String, ToCompactString};
use vize_croquis::{
    reactivity::{ReactivityLoss, ReactivityLossKind},
    script_parser::ScriptParseResult,
    virtual_ts::VirtualTsOutput,
};

#[derive(Clone)]
pub(super) struct ReactivityLossQuery {
    pub generated_offset: u32,
    pub source_start: u32,
    pub source_end: u32,
    message: CompactString,
    help: &'static str,
}

impl ReactivityLossQuery {
    #[inline]
    pub fn owner_key(&self) -> u64 {
        ((self.source_start as u64) << 32) | self.source_end as u64
    }

    pub fn diagnostic(&self, script_offset: u32) -> LintDiagnostic {
        LintDiagnostic::warn(
            RULE_NO_REACTIVITY_LOSS,
            self.message.clone(),
            script_offset + self.source_start,
            script_offset + self.source_end,
        )
        .with_help(self.help)
    }
}

pub(super) fn collect_reactivity_loss_queries(
    linter: &Linter,
    result: &mut LintResult,
    parse_result: &ScriptParseResult,
    script_content: &str,
    script_offset: u32,
    virtual_ts: &mut VirtualTsOutput,
) -> Vec<ReactivityLossQuery> {
    if !(linter.registry.has_rule(RULE_NO_REACTIVITY_LOSS)
        && linter.is_rule_enabled(RULE_NO_REACTIVITY_LOSS))
    {
        return Vec::new();
    }
    if !parse_result.reactivity.has_losses() {
        return Vec::new();
    }

    let mut queries = Vec::with_capacity(parse_result.reactivity.losses().len());
    let mut immediate = FxHashSet::default();

    for loss in parse_result.reactivity.losses() {
        let diagnostic = reactivity_loss_diagnostic(loss);
        let expressions = query_expressions_for_loss(loss, parse_result, script_content);

        if expressions.is_empty() {
            let key = diagnostic_key(loss.start, loss.end);
            if immediate.insert(key) {
                push_warning(result, diagnostic.diagnostic(script_offset));
            }
            continue;
        }

        for expression in expressions {
            if let Some(query) =
                push_reactivity_loss_marker(virtual_ts, expression.as_str(), &diagnostic)
            {
                queries.push(query);
            }
        }
    }

    queries
}

fn push_reactivity_loss_marker(
    virtual_ts: &mut VirtualTsOutput,
    expression_source: &str,
    diagnostic: &ReactivityLossQuery,
) -> Option<ReactivityLossQuery> {
    let expression_source = expression_source.trim();
    if expression_source.is_empty() {
        return None;
    }
    let insert_offset = marker_insert_offset(&virtual_ts.content)?;

    let mut marker_name = String::with_capacity(32);
    marker_name.push_str("__vize_patina_reactivity_");
    marker_name.push_str(diagnostic.source_start.to_compact_string().as_str());
    marker_name.push('_');
    marker_name.push_str(diagnostic.source_end.to_compact_string().as_str());
    marker_name.push('_');
    marker_name.push_str(virtual_ts.content.len().to_compact_string().as_str());

    let mut line = String::with_capacity(marker_name.len() + expression_source.len() + 24);
    line.push_str("    const ");
    let name_offset = line.len() as u32;
    line.push_str(&marker_name);
    line.push_str(" = (");
    line.push_str(expression_source);
    line.push_str(");\n");

    let mut query = diagnostic.clone();
    query.generated_offset = insert_offset as u32 + name_offset;
    virtual_ts.content.insert_str(insert_offset, &line);
    Some(query)
}

fn query_expressions_for_loss(
    loss: &ReactivityLoss,
    parse_result: &ScriptParseResult,
    script_content: &str,
) -> Vec<CompactString> {
    match &loss.kind {
        ReactivityLossKind::PropsDestructure { destructured_props } => {
            props_destructure_locals(parse_result, destructured_props)
        }
        ReactivityLossKind::RefValueExtract { .. }
        | ReactivityLossKind::ReactivePropertyExtract { .. }
        | ReactivityLossKind::FunctionArgumentExtract { .. }
        | ReactivityLossKind::GetterCallExtract { .. }
        | ReactivityLossKind::PlainValueAlias { .. } => script_content
            .get(loss.start as usize..loss.end as usize)
            .map(str::trim)
            .filter(|source| !source.is_empty())
            .map(|source| vec![CompactString::new(source)])
            .unwrap_or_default(),
        ReactivityLossKind::ReactiveDestructure { .. }
        | ReactivityLossKind::RefValueDestructure { .. }
        | ReactivityLossKind::ReactiveSpread { .. }
        | ReactivityLossKind::ReactiveReassign { .. } => Vec::new(),
    }
}

fn props_destructure_locals(
    parse_result: &ScriptParseResult,
    destructured_props: &[CompactString],
) -> Vec<CompactString> {
    let Some(destructure) = parse_result.macros.props_destructure() else {
        return destructured_props.to_vec();
    };

    let mut locals =
        Vec::with_capacity(destructured_props.len() + usize::from(destructure.rest_id.is_some()));
    for prop in destructured_props {
        if prop.as_str() == "(rest)" {
            if let Some(rest_id) = &destructure.rest_id {
                locals.push(rest_id.clone());
            }
            continue;
        }
        if let Some(binding) = destructure.get(prop.as_str()) {
            locals.push(binding.local.clone());
        } else {
            locals.push(prop.clone());
        }
    }
    locals
}

fn reactivity_loss_diagnostic(loss: &ReactivityLoss) -> ReactivityLossQuery {
    let (message, help) = match &loss.kind {
        ReactivityLossKind::ReactiveDestructure {
            source_name,
            destructured_props,
        } => (
            cstr!(
                "Destructuring reactive value '{}' creates plain snapshots for: {}",
                source_name,
                destructured_props.join(", ")
            ),
            "Use `toRefs(...)`, `toRef(...)`, or access the property through the reactive object.",
        ),
        ReactivityLossKind::RefValueDestructure {
            source_name,
            destructured_props,
        } => (
            cstr!(
                "Destructuring '{}.value' creates plain snapshots for: {}",
                source_name,
                destructured_props.join(", ")
            ),
            "Keep the ref boundary and derive values through `computed(...)` or `toRef(...)`.",
        ),
        ReactivityLossKind::RefValueExtract {
            source_name,
            target_name,
        } => (
            cstr!(
                "Assigning '{}.value' to '{}' stores a plain snapshot",
                source_name,
                target_name
            ),
            "Pass the ref itself, use a getter `() => ref.value`, or wrap the derived value in `computed(...)`.",
        ),
        ReactivityLossKind::ReactivePropertyExtract {
            source_name,
            prop_name,
            target_name,
        } => (
            cstr!(
                "Assigning '{}.{}' to '{}' stores a plain snapshot",
                source_name,
                prop_name,
                target_name
            ),
            "Use `toRef(source, 'key')`, `toRefs(source)`, or access the property on the reactive object.",
        ),
        ReactivityLossKind::PropsDestructure { destructured_props } => (
            cstr!(
                "Destructuring props creates plain snapshots for: {}",
                destructured_props.join(", ")
            ),
            "Use `toRefs(props)`, `toRef(props, 'key')`, or pass a getter `() => prop` across call boundaries.",
        ),
        ReactivityLossKind::FunctionArgumentExtract {
            source_name,
            argument_name,
            callee_name,
        } => (
            cstr!(
                "Passing '{}' to '{}' crosses a plain-value boundary from '{}'",
                argument_name,
                callee_name,
                source_name
            ),
            "Pass a ref, `toRef(...)`, `computed(...)`, or a getter so the callee can observe later updates.",
        ),
        ReactivityLossKind::GetterCallExtract {
            context_name,
            getter_name,
            target_name,
            callee_name,
            source_name,
        } => (
            cstr!(
                "Assigning '{}.{}()' to '{}' stores a plain snapshot from '{}' returned by '{}'",
                context_name,
                getter_name,
                target_name,
                source_name,
                callee_name
            ),
            "Keep the getter lazy, wrap it in `computed(...)`, or have the composable return a ref-like value.",
        ),
        ReactivityLossKind::PlainValueAlias {
            source_name,
            alias_name,
            target_name,
        } => (
            cstr!(
                "Assigning plain snapshot '{}' to '{}' keeps reactivity lost from '{}'",
                alias_name,
                target_name,
                source_name
            ),
            "Pass the reactive source itself, a getter, `toRef(...)`, or `computed(...)` instead of aliasing the snapshot.",
        ),
        ReactivityLossKind::ReactiveSpread { source_name } => (
            cstr!("Spreading '{}' creates a non-reactive copy", source_name),
            "Keep the reactive object intact, or copy through refs with `toRefs(...)` when destructuring is intentional.",
        ),
        ReactivityLossKind::ReactiveReassign { source_name } => (
            cstr!("Reassigning reactive binding '{}' breaks tracked identity", source_name),
            "Mutate the reactive object in place or store replaceable state in a ref.",
        ),
    };

    ReactivityLossQuery {
        generated_offset: 0,
        source_start: loss.start,
        source_end: loss.end.max(loss.start.saturating_add(1)),
        message,
        help,
    }
}

#[inline]
fn diagnostic_key(start: u32, end: u32) -> u64 {
    ((start as u64) << 32) | end as u64
}
