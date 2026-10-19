//! Deferred checks for root owners with no mutable template dependencies.
//!
//! Reuse actual Croquis usage indices and event scope spans. These are emission
//! buffers, not another template/scope graph. Dynamic expressions deliberately
//! keep the old order: moving a closure past an assignment changes TS narrowing.

use vize_carton::{FxHashMap, FxHashSet, String};
use vize_croquis::{BindingType, Scope, ScopeData, analysis::ComponentUsage};
use vize_relief::{ExpressionNode, PropNode, TemplateChildNode};

use super::{component_prop_navigation, context::ComponentPropsContext};
use crate::template_diagnostic_directives::{DirectivePolicy, TemplateDiagnosticDirectives};
use crate::virtual_ts::{VizeSemanticLink, types::VizeMapping};

#[derive(Default)]
pub(super) struct RootOwnerPacket {
    pub(super) text: String,
    pub(super) mappings: Vec<VizeMapping>,
    pub(super) diagnostic_mappings: Vec<VizeMapping>,
    pub(super) semantic_links: Vec<VizeSemanticLink>,
}

impl RootOwnerPacket {
    pub(super) fn append_to(
        self,
        text: &mut String,
        mappings: &mut Vec<VizeMapping>,
        diagnostic_mappings: &mut Vec<VizeMapping>,
        links: &mut Vec<VizeSemanticLink>,
    ) {
        let offset = text.len();
        text.push_str(self.text.as_str());
        for (target, rows) in [
            (mappings, self.mappings),
            (diagnostic_mappings, self.diagnostic_mappings),
        ] {
            target.extend(rows.into_iter().map(|mut mapping| {
                mapping.gen_range.start += offset;
                mapping.gen_range.end += offset;
                for span in &mut mapping.sub_spans {
                    span.gen_range.start += offset;
                    span.gen_range.end += offset;
                }
                mapping
            }));
        }
        links.extend(self.semantic_links.into_iter().map(|mut link| {
            link.source_range.start += offset;
            link.source_range.end += offset;
            link.target_range.start += offset;
            link.target_range.end += offset;
            link
        }));
    }
}

#[derive(Default)]
pub(super) struct RootOwnerPackets {
    packets: FxHashMap<usize, RootOwnerPacket>,
    /// The exact directive spans used by Croquis when creating handler scopes.
    events: FxHashMap<(u32, u32), usize>,
}

impl RootOwnerPackets {
    pub(super) fn collect(
        ctx: &ComponentPropsContext<'_, '_>,
        usages: &[(usize, &ComponentUsage)],
    ) -> Self {
        let mut result = Self::default();
        let Some(root) = ctx.template_ast else {
            return result;
        };
        if !root.source.contains("@vue-expect-error")
            || ctx.legacy_vue2
            || ctx.options.strict_instance_globals
            || ctx.experimental_strict_slot_children
            || !crate::virtual_ts::script_facts::is_script_setup(ctx.summary)
        {
            return result;
        }
        // The source marker is only a cheap activation gate. Exact ownership,
        // precedence and orphan handling come from the shared canonical walk.
        let directives = TemplateDiagnosticDirectives::for_root(root, 0);
        let expected_owners: FxHashSet<_> = directives
            .directives()
            .iter()
            .filter(|directive| directive.policy == DirectivePolicy::Expect)
            .flat_map(|directive| {
                directive
                    .targets
                    .iter()
                    .map(|range| (range.start, range.end))
            })
            .collect();
        if expected_owners.is_empty() {
            return result;
        }
        let by_start: FxHashMap<_, _> = usages
            .iter()
            .map(|&(idx, usage)| (usage.start, (idx, usage)))
            .collect();
        crate::virtual_ts::script_facts::with_bindings(ctx.summary, |bindings| {
            for child in &root.children {
                let TemplateChildNode::Element(element) = child else {
                    continue;
                };
                let Some(&(idx, usage)) = by_start.get(&element.loc.span.start) else {
                    continue;
                };
                if !expected_owners.contains(&(usage.start as usize, usage.end as usize))
                    || usage.end != element.loc.span.end || usage.vif_guard.is_some()
                    || ctx.explicit_generics.alias(usage.start).is_some()
                    || bindings.binding_type(usage.name.as_str()) != Some(BindingType::SetupConst)
                    || usage.props.is_empty()
                    || usage.props.iter().all(|prop| matches!(prop.name.as_str(), "key" | "ref"))
                    || usage.props.iter().any(|prop| prop.is_dynamic || prop.name_is_dynamic)
                    || !usage.spread_props.is_empty()
                    || usage.events.is_empty()
                    || usage.events.iter().any(|event| event.name_is_dynamic
                        || event.handler.as_ref().is_none_or(|handler|
                            bindings.binding_type(handler.as_str().trim()) != Some(BindingType::SetupConst)))
                    || element.props.iter().any(|prop| match prop {
                        PropNode::Attribute(_) => false,
                        PropNode::Directive(directive) => directive.name != "on"
                            || !matches!(directive.arg.as_ref(), Some(ExpressionNode::Simple(arg)) if arg.is_static),
                    })
                    || usage.slots.iter().any(|slot|
                        component_prop_navigation::slot_navigation_source_range(ctx.template_source, slot)
                            .is_some_and(|range| range.start < usage.start as usize || range.end > usage.end as usize))
                {
                    continue;
                }
                result.packets.insert(idx, RootOwnerPacket::default());
                for event in &usage.events {
                    result.events.insert((event.start, event.end), idx);
                }
            }
        });
        result
    }

    pub(super) fn contains(&self, idx: usize) -> bool {
        self.packets.contains_key(&idx)
    }
    pub(super) fn is_empty(&self) -> bool {
        self.packets.is_empty()
    }
    pub(super) fn get_mut(&mut self, idx: usize) -> Option<&mut RootOwnerPacket> {
        self.packets.get_mut(&idx)
    }
    pub(super) fn for_scope(&mut self, scope: &Scope) -> Option<&mut RootOwnerPacket> {
        if !matches!(scope.data(), ScopeData::EventHandler(_)) {
            return None;
        }
        let idx = self.events.get(&(scope.span.start, scope.span.end))?;
        self.packets.get_mut(idx)
    }
    pub(super) fn remove(&mut self, idx: usize) -> Option<RootOwnerPacket> {
        self.packets.remove(&idx)
    }
}

#[cfg(test)]
mod tests {
    use super::super::{
        closures::emit_scope_closures,
        context::{ComponentBindingCheck, GlobalComponentCheck, ScopeGenerationOptions},
    };
    use crate::virtual_ts::{
        VirtualTsCheckOptions, VirtualTsOptions, VizeSemanticLink,
        template_binding_access::TemplateBindingAccess, types::VizeMapping,
    };
    use vize_carton::{FxHashSet, String, cstr};
    use vize_croquis::{Analyzer, AnalyzerOptions};

    const SCRIPT: &str = "import Comp from './Comp.vue'; const value = 1; const noop = () => {};";
    type Output = (String, Vec<VizeMapping>, Vec<VizeSemanticLink>);

    fn render(script: &str, template: &str, packets: bool, navigation: bool) -> Output {
        let allocator = vize_carton::Allocator::new();
        let (root, errors) = vize_armature::parse(&allocator, template);
        assert!(errors.is_empty(), "{errors:?}");
        let mut analyzer = Analyzer::with_options(AnalyzerOptions::full());
        analyzer.analyze_script_setup(script);
        analyzer.analyze_template(&root);
        let summary = analyzer.finish();
        let options = VirtualTsOptions::default();
        let names = FxHashSet::default();
        let access = TemplateBindingAccess::collect(&summary, false);
        let mut result = (String::default(), Vec::new(), Vec::new());
        let mut diagnostics = Vec::new();
        emit_scope_closures(
            &mut result.0,
            &mut super::super::context::ScopeMappings::new(&mut result.1, &mut diagnostics),
            &mut result.2,
            &summary,
            &access,
            0,
            (
                ScopeGenerationOptions {
                    check_options: VirtualTsCheckOptions {
                        check_unknown_events: true,
                        ..Default::default()
                    },
                    virtual_ts_options: &options,
                    setup_spread_bindings: &[],
                    keyed_prop_bindings: &FxHashSet::default(),
                    syntactic_type_only_imported_names: &names,
                    template_ast: Some(&root),
                    component_binding_check: ComponentBindingCheck {
                        globals: GlobalComponentCheck::None,
                        self_component_name: None,
                    },
                    legacy_vue2: false,
                    options_api: false,
                    preserve_event_navigation: navigation,
                    has_default_alias: false,
                    script_content: Some(script),
                    experimental_strict_slot_children: false,
                    forwarded_root_starts: &[],
                    instantiated_ref_starts: &[],
                },
                packets,
            ),
        );
        result.1.extend(diagnostics);
        result
    }

    // Preserve every full mapped payload and semantic edge under the relocation;
    // generated offsets change, authored offsets and generated bytes do not.
    fn mapping_payloads(output: &Output) -> Vec<Vec<u8>> {
        let mut result = output
            .1
            .iter()
            .map(|mapping| {
                let spans = mapping
                    .sub_spans
                    .iter()
                    .map(|span| {
                        (
                            span.src_range.clone(),
                            output.0.get(span.gen_range.clone()).unwrap(),
                        )
                    })
                    .collect::<Vec<_>>();
                serde_json::to_vec(&(
                    mapping.src_range.clone(),
                    output.0.get(mapping.gen_range.clone()).unwrap(),
                    spans,
                ))
                .unwrap()
            })
            .collect::<Vec<_>>();
        result.sort();
        result
    }

    fn link_payloads(output: &Output) -> Vec<Vec<u8>> {
        let mut result = output
            .2
            .iter()
            .map(|link| {
                let source_owners = output
                    .1
                    .iter()
                    .filter(|mapping| mapping.gen_range == link.source_range)
                    .map(|mapping| mapping.src_range.clone())
                    .collect::<Vec<_>>();
                let before = output.0.get(..link.target_range.start).unwrap();
                let after = output.0.get(link.target_range.end..).unwrap();
                let line_start = before.rfind('\n').map_or(0, |index| index + 1);
                let line_end = link.target_range.end + after.find('\n').unwrap_or(after.len());
                serde_json::to_vec(&(
                    link.kind as u8,
                    source_owners,
                    output.0.get(link.source_range.clone()).unwrap(),
                    output.0.get(link.target_range.clone()).unwrap(),
                    output.0.get(line_start..line_end).unwrap(),
                ))
                .unwrap()
            })
            .collect::<Vec<_>>();
        result.sort();
        result
    }

    fn owner_bounds(output: &Output, source: std::ops::Range<usize>) -> std::ops::Range<usize> {
        let rows = output
            .1
            .iter()
            .filter(|row| row.src_range.start < source.end && source.start < row.src_range.end)
            .collect::<Vec<_>>();
        rows.iter().map(|row| row.gen_range.start).min().unwrap()
            ..rows.iter().map(|row| row.gen_range.end).max().unwrap()
    }

    #[test]
    fn complete_root_packet_separates_child_and_retains_every_operation() {
        for newline in ["\n", "\r\n"] {
            for prefix in ["", "😀日本\n"] {
                for navigation in [false, true] {
                    for (id, event) in [
                        ("bad", "missing"),
                        ("ok", "missing"),
                        ("bad", "save"),
                        ("ok", "save"),
                    ] {
                        let template = cstr!(
                            "{prefix}<!-- @vue-expect-error -->{newline}<Comp{newline} id=\"{id}\" @{event}=\"noop\">{{{{ value.bad }}}}</Comp>"
                        );
                        let old = render(SCRIPT, &template, false, navigation);
                        let new = render(SCRIPT, &template, true, navigation);
                        let mut old_lines = old.0.lines().collect::<Vec<_>>();
                        let mut new_lines = new.0.lines().collect::<Vec<_>>();
                        old_lines.sort();
                        new_lines.sort();
                        assert_eq!(old_lines, new_lines, "one emission per old operation");
                        assert_eq!(mapping_payloads(&old), mapping_payloads(&new));
                        assert_eq!(link_payloads(&old), link_payloads(&new));
                        assert!(!old.2.is_empty(), "whole semantic links are exercised");
                        let start = template.find("<Comp").unwrap();
                        let end = start + template.get(start..).unwrap().find('>').unwrap() + 1;
                        let old_owner = owner_bounds(&old, start..end);
                        let new_owner = owner_bounds(&new, start..end);
                        let child = template.find("value.bad").unwrap();
                        let old_child = owner_bounds(&old, child..child + "value.bad".len());
                        let new_child = owner_bounds(&new, child..child + "value.bad".len());
                        assert!(
                            old_owner.start < old_child.start && old_child.end < old_owner.end,
                            "old exact layout is RED"
                        );
                        assert!(
                            new_child.end <= new_owner.start || new_owner.end <= new_child.start,
                            "parent must not claim a child check"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn unsupported_dependency_shapes_keep_whole_original_output() {
        let cases = [
            (
                SCRIPT,
                "<!-- prose @vue-expect-error --><Comp id=\"bad\" @missing=\"noop\">{{ value.bad }}</Comp>",
            ),
            (
                SCRIPT,
                "@vue-expect-error<Comp id=\"bad\" @missing=\"noop\">{{ value.bad }}</Comp>",
            ),
            (
                SCRIPT,
                "<Comp id=\"@vue-expect-error\" @missing=\"noop\">{{ value.bad }}</Comp>",
            ),
            (
                SCRIPT,
                "<Comp id=\"bad\" @missing=\"noop\">{{ value.bad }}</Comp><!-- @vue-expect-error -->",
            ),
            (
                SCRIPT,
                "<!-- @vue-expect-error --><!-- @vue-ignore --><Comp id=\"bad\" @missing=\"noop\">{{ value.bad }}</Comp>",
            ),
            (
                SCRIPT,
                "<!-- @vue-expect-error --><!-- @vue-skip --><Comp id=\"bad\" @missing=\"noop\">{{ value.bad }}</Comp>",
            ),
            (
                SCRIPT,
                "<Comp id=\"bad\" @missing=\"noop\">{{ value.bad }}</Comp>",
            ),
            (
                SCRIPT,
                "<!-- @vue-expect-error --><Comp :id=\"value\" @missing=\"noop\">{{ value.bad }}</Comp>",
            ),
            (
                "import Comp from './Comp.vue'; let value: string | number = '';",
                "<!-- @vue-expect-error --><Comp id=\"bad\" @save=\"value.toFixed()\">{{ value = 1 }}</Comp>",
            ),
            (
                "import Comp from './Comp.vue'; let noop = () => {}; const value = 1;",
                "<!-- @vue-expect-error --><Comp id=\"bad\" @save=\"noop\">{{ value.bad }}</Comp>",
            ),
            (
                SCRIPT,
                "<!-- @vue-expect-error --><Comp id=\"bad\" @save=\"newName\">{{ value.bad }}</Comp>",
            ),
            (
                SCRIPT,
                "<!-- @vue-expect-error --><Comp v-if=\"value\" id=\"bad\" @save=\"noop\">{{ value.bad }}</Comp>",
            ),
            (
                SCRIPT,
                "<!-- @vue-expect-error --><Comp v-for=\"item in [value]\" id=\"bad\" @save=\"noop\">{{ item.bad }}</Comp>",
            ),
            (
                SCRIPT,
                "<!-- @vue-expect-error --><Comp id=\"bad\" @save=\"noop\" v-slot=\"{ item }\">{{ item.bad }}</Comp>",
            ),
            (
                SCRIPT,
                "<!-- @vue-expect-error --><Comp id=\"bad\" @save=\"noop\"><template #missing>{{ value.bad }}</template></Comp>",
            ),
            (
                SCRIPT,
                "<!-- @vue-expect-error --><!-- @vue-generic {string} --><Comp id=\"bad\" @save=\"noop\">{{ value.bad }}</Comp>",
            ),
            (
                "",
                "<!-- @vue-expect-error --><Comp id=\"bad\" @save=\"newName\">{{ child }}</Comp>",
            ),
        ];
        for (script, template) in cases {
            for navigation in [false, true] {
                assert_eq!(
                    render(script, template, false, navigation),
                    render(script, template, true, navigation),
                    "{template}"
                );
            }
        }
    }

    #[test]
    fn separate_root_owners_never_share_a_packet() {
        let template = "<!-- @vue-expect-error --><Comp id=\"bad\" @missing=\"noop\">{{ value.bad }}</Comp>\n<!-- @vue-expect-error --><Comp id=\"ok\" @save=\"noop\">{{ value.other }}</Comp>";
        let old = render(SCRIPT, template, false, true);
        let new = render(SCRIPT, template, true, true);
        assert_eq!(mapping_payloads(&old), mapping_payloads(&new));
        assert_eq!(link_payloads(&old), link_payloads(&new));
        let first_start = template.find("<Comp").unwrap();
        let first_end = first_start + template.get(first_start..).unwrap().find('>').unwrap() + 1;
        let second_start = template.rfind("<Comp").unwrap();
        let second_end =
            second_start + template.get(second_start..).unwrap().find('>').unwrap() + 1;
        let first = owner_bounds(&new, first_start..first_end);
        let second = owner_bounds(&new, second_start..second_end);
        assert!(first.end <= second.start || second.end <= first.start);
        for child in ["value.bad", "value.other"] {
            let start = template.find(child).unwrap();
            let child = owner_bounds(&new, start..start + child.len());
            assert!(child.end <= first.start || first.end <= child.start);
            assert!(child.end <= second.start || second.end <= child.start);
        }
    }
}
