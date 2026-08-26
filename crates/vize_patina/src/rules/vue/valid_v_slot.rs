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
use vize_relief::{
    DirectiveNode, ElementNode, ElementType, ExpressionNode, PropNode, TemplateChildNode,
};
use vize_s0::{FxHashSet, String, ToCompactString, is_native_tag};

static META: RuleMeta = RuleMeta {
    name: "vue/valid-v-slot",
    description: "Enforce valid `v-slot` directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Error,
};

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
        if directive.name != "slot" {
            return;
        }

        let tag = element.tag;

        // v-slot can only be used on components or <template>
        if tag != "template" && !Self::is_custom_component(element) {
            ctx.error_with_help(
                ctx.t("vue/valid-v-slot.invalid_location"),
                &directive.loc,
                ctx.t("vue/valid-v-slot.help"),
            );
            return;
        }

        if tag != "template" && Self::is_named_slot(directive) {
            ctx.error_with_help(
                ctx.t("vue/valid-v-slot.invalid_location"),
                &directive.loc,
                ctx.t("vue/valid-v-slot.help"),
            );
        }

        if tag == "template" && !has_component_parent(ctx) {
            ctx.error_with_help(
                ctx.t("vue/valid-v-slot.invalid_location"),
                &directive.loc,
                ctx.t("vue/valid-v-slot.help"),
            );
        }

        // Check for duplicate v-slot directives
        let (default_count, named_count) = Self::count_slot_directives(element);

        if default_count > 1 {
            ctx.error_with_help(
                ctx.t("vue/valid-v-slot.invalid_location"),
                &directive.loc,
                ctx.t("vue/valid-v-slot.help"),
            );
        }

        // On <template>, can only have one named slot
        if tag == "template" && named_count > 1 {
            ctx.error_with_help(
                ctx.t("vue/valid-v-slot.invalid_location"),
                &directive.loc,
                ctx.t("vue/valid-v-slot.help"),
            );
        }

        // Mixing default and named on same element
        if default_count > 0 && named_count > 0 {
            ctx.error_with_help(
                ctx.t("vue/valid-v-slot.invalid_location"),
                &directive.loc,
                ctx.t("vue/valid-v-slot.help"),
            );
        }
    }
}

fn check_child_slot_templates(ctx: &mut LintContext, owner: &ElementNode) {
    let owner_default_slot = owner_default_slot_directive(owner);
    let mut has_child_slot_template = false;
    let mut seen_slots: FxHashSet<String> = FxHashSet::default();
    let mut current_group_slots: FxHashSet<String> = FxHashSet::default();
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
            if !current_group_slots.contains(&name) && seen_slots.contains(&name) {
                ctx.error_with_help(
                    ctx.t("vue/valid-v-slot.invalid_location"),
                    &slot.loc,
                    ctx.t("vue/valid-v-slot.help"),
                );
            }
            current_group_slots.insert(name);
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

fn finish_slot_group(seen_slots: &mut FxHashSet<String>, group_slots: &mut FxHashSet<String>) {
    seen_slots.extend(group_slots.drain());
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
