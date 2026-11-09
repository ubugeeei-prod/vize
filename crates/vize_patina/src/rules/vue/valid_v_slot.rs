//! vue/valid-v-slot
//!
//! Enforce valid `v-slot` directives.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <div v-slot:header></div>                 <!-- not on component -->
//! <MyComponent v-slot v-slot:header />      <!-- duplicate -->
//! <template v-slot:header v-slot:footer />  <!-- multiple named slots -->
//! ```
//!
//! ### Valid
//! ```vue
//! <MyComponent v-slot="{ item }">{{ item }}</MyComponent>
//! <MyComponent><template #header>Header</template></MyComponent>
//! <MyComponent><template v-slot:header>Header</template></MyComponent>
//! ```

use crate::context::{ElementContext, LintContext};
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_croquis::drawer::{extract_identifier_refs_retained_only, parse_v_for_scope_expression};
use vize_l0::{String, ToCompactString, is_native_tag};
use vize_relief::{
    DirectiveNode, ElementNode, ElementType, ExpressionNode, PropNode, TemplateChildNode,
};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-slot",
    description: "Enforce valid `v-slot` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

mod checks;
mod dynamic_binding;

/// Valid v-slot rule
#[derive(Default)]
pub struct ValidVSlot;

impl ValidVSlot {
    fn is_custom_component(element: &ElementNode) -> bool {
        Self::is_custom_component_tag(element.tag, Some(element.tag_type))
    }

    fn is_custom_component_tag(tag: &str, tag_type: Option<ElementType>) -> bool {
        if matches!(tag_type, Some(ElementType::Slot | ElementType::Template))
            || matches!(tag, "slot" | "template")
        {
            return false;
        }

        // Vue treats unknown non-native tags as components, including
        // lowercase single-word registered components such as <draggable>.
        matches!(tag_type, Some(ElementType::Component))
            || tag == "component"
            || !is_native_tag(tag)
    }

    fn count_slot_directives(element: &ElementNode) -> (usize, usize) {
        let mut default_count = 0;
        let mut named_count = 0;

        for prop in &element.props {
            if let PropNode::Directive(dir) = prop
                && dir.name == "slot"
            {
                if dir.arg.is_some() {
                    named_count += 1;
                } else {
                    default_count += 1;
                }
            }
        }

        (default_count, named_count)
    }

    fn is_named_slot(directive: &DirectiveNode) -> bool {
        match &directive.arg {
            None => false,
            Some(ExpressionNode::Simple(arg)) => arg.content != "default",
            Some(ExpressionNode::Compound(_)) => true,
        }
    }
}

impl Rule for ValidVSlot {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        if Self::is_custom_component(element) {
            check_child_slot_templates(ctx, element);
        }
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        self.check_slot_directive(ctx, element, directive, false);
    }
}

struct ChildSlot {
    loop_scope: Option<LoopScopedSlot>,
    name: String,
}

struct LoopScopedSlot {
    source: String,
    value_pattern: String,
    key_alias: Option<String>,
    index_alias: Option<String>,
    uses_value: bool,
    uses_key: bool,
    uses_index: bool,
}

fn check_child_slot_templates(ctx: &mut LintContext, owner: &ElementNode) {
    let owner_default_slot = owner_default_slot_directive(owner);
    let mut has_child_slot_template = false;
    let mut seen_slots = Vec::new();
    let mut current_group_slots = Vec::new();
    let mut in_if_chain = false;

    for child in owner.children.iter() {
        let TemplateChildNode::Element(child) = child else {
            continue;
        };

        let Some(slot) = child_slot_directive(child) else {
            finish_slot_group(&mut seen_slots, &mut current_group_slots);
            in_if_chain = false;
            continue;
        };
        has_child_slot_template = true;

        let starts_chain = has_directive(child, "if");
        let continues_chain =
            in_if_chain && (has_directive(child, "else-if") || has_directive(child, "else"));

        if starts_chain || !continues_chain {
            finish_slot_group(&mut seen_slots, &mut current_group_slots);
            in_if_chain = starts_chain;
        }

        if let Some(name) = static_slot_name(slot) {
            let occurrence = ChildSlot {
                loop_scope: loop_scoped_slot(child, slot),
                name,
            };
            if !current_group_slots
                .iter()
                .any(|existing| same_distributed_slot(existing, &occurrence))
                && seen_slots
                    .iter()
                    .any(|existing| same_distributed_slot(existing, &occurrence))
            {
                ctx.error_with_help(
                    ctx.t("vue/valid-v-slot.invalid_location"),
                    &slot.loc,
                    ctx.t("vue/valid-v-slot.help"),
                );
            }
            current_group_slots.push(occurrence);
        }

        if !starts_chain && !continues_chain {
            finish_slot_group(&mut seen_slots, &mut current_group_slots);
            in_if_chain = false;
        }
    }

    finish_slot_group(&mut seen_slots, &mut current_group_slots);

    if let Some(default_slot) = owner_default_slot
        && has_child_slot_template
    {
        ctx.error_with_help(
            ctx.t("vue/valid-v-slot.mixed_default_and_child_slots"),
            &default_slot.loc,
            ctx.t("vue/valid-v-slot.help"),
        );
    }
}

fn finish_slot_group(seen_slots: &mut Vec<ChildSlot>, group_slots: &mut Vec<ChildSlot>) {
    seen_slots.append(group_slots);
}

fn same_distributed_slot(left: &ChildSlot, right: &ChildSlot) -> bool {
    if left.name != right.name {
        return false;
    }

    match (&left.loop_scope, &right.loop_scope) {
        (Some(left), Some(right)) => same_loop_scope(left, right),
        // Suppress an existing duplicate only when both slot names are proven
        // to depend on their own loops. An incomplete parse, a property name,
        // or an outer binding must retain the conservative duplicate finding.
        _ => true,
    }
}

fn loop_scoped_slot(element: &ElementNode<'_>, slot: &DirectiveNode<'_>) -> Option<LoopScopedSlot> {
    let ExpressionNode::Simple(argument) = slot.arg.as_ref()? else {
        return None;
    };
    if argument.is_static {
        return None;
    }
    let argument_ast = argument.js_ast?.as_expression()?;
    let reads = extract_identifier_refs_retained_only(argument.content, &argument_ast)?;
    let for_expression = element.props.iter().find_map(|prop| match prop {
        PropNode::Directive(directive) if directive.name == "for" => match directive.exp.as_ref() {
            Some(ExpressionNode::Simple(expression)) => Some(expression.content),
            _ => None,
        },
        _ => None,
    })?;
    let aliases = parse_v_for_scope_expression(for_expression)?;
    let uses = |name: &str| reads.iter().any(|read| read.name == name);
    let uses_value = aliases
        .value_bindings
        .iter()
        .any(|binding| uses(binding.as_str()));
    let uses_key = aliases
        .key_alias
        .as_ref()
        .is_some_and(|binding| uses(binding.as_str()));
    let uses_index = aliases
        .index_alias
        .as_ref()
        .is_some_and(|binding| uses(binding.as_str()));
    if !uses_value && !uses_key && !uses_index {
        return None;
    }

    Some(LoopScopedSlot {
        source: aliases.source,
        value_pattern: aliases.value_pattern,
        key_alias: aliases.key_alias,
        index_alias: aliases.index_alias,
        uses_value,
        uses_key,
        uses_index,
    })
}

fn same_loop_scope(left: &LoopScopedSlot, right: &LoopScopedSlot) -> bool {
    if left.uses_value != right.uses_value
        || left.uses_key != right.uses_key
        || left.uses_index != right.uses_index
        || (left.uses_key && left.key_alias != right.key_alias)
        || (left.uses_index && left.index_alias != right.index_alias)
        || left.source != right.source
    {
        return false;
    }

    // These strings are the centralized v-for parser's selected source and
    // binding-pattern projections. Exact equality is intentionally
    // conservative: uncertain spellings keep the pre-existing duplicate.
    !left.uses_value || left.value_pattern == right.value_pattern
}

fn owner_default_slot_directive<'a>(element: &'a ElementNode<'a>) -> Option<&'a DirectiveNode<'a>> {
    element.props.iter().find_map(|prop| match prop {
        PropNode::Directive(dir) if dir.name == "slot" && !ValidVSlot::is_named_slot(dir) => {
            Some(dir.as_ref())
        }
        _ => None,
    })
}

fn child_slot_directive<'a>(element: &'a ElementNode<'a>) -> Option<&'a DirectiveNode<'a>> {
    if element.tag != "template" {
        return None;
    }

    element.props.iter().find_map(|prop| match prop {
        PropNode::Directive(dir) if dir.name == "slot" => Some(dir.as_ref()),
        _ => None,
    })
}

fn has_directive(element: &ElementNode, name: &str) -> bool {
    element
        .props
        .iter()
        .any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == name))
}

fn has_component_parent(ctx: &LintContext) -> bool {
    ctx.parent_element()
        .is_some_and(is_component_parent_context)
}

fn is_component_parent_context(parent: &ElementContext) -> bool {
    let tag = parent.tag.as_str();
    !matches!(tag, "slot" | "template") && (tag == "component" || !is_native_tag(tag))
}

fn static_slot_name(directive: &DirectiveNode) -> Option<String> {
    match &directive.arg {
        None => Some("default".into()),
        Some(ExpressionNode::Simple(arg)) => {
            let mut name = arg.content.to_compact_string();
            for modifier in &directive.modifiers {
                name.push('.');
                name.push_str(modifier.content);
            }
            Some(name)
        }
        Some(ExpressionNode::Compound(_)) => None,
    }
}

#[cfg(test)]
mod location_tests;

#[cfg(test)]
mod mixed_default_tests;

#[cfg(test)]
mod tests;

struct ConfiguredValidVSlot;

impl ValidVSlot {
    pub(crate) fn allowing_modifiers() -> impl Rule {
        ConfiguredValidVSlot
    }
}

impl Rule for ConfiguredValidVSlot {
    fn meta(&self) -> &'static RuleMeta {
        ValidVSlot.meta()
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        ValidVSlot.enter_element(ctx, element);
    }

    fn check_directive<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        directive: &DirectiveNode<'a>,
    ) {
        ValidVSlot.check_slot_directive(ctx, element, directive, true);
    }
}
