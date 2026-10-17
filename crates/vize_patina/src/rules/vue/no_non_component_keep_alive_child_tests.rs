use super::NoNonComponentKeepAliveChild;
use crate::linter::Linter;
use crate::rule::RuleRegistry;

fn create_linter() -> Linter {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoNonComponentKeepAliveChild));
    Linter::with_registry(registry)
}

#[test]
fn reports_plain_keep_alive_child() {
    let result = create_linter().lint_template(
        r#"<KeepAlive><div><SomeComponent /></div></KeepAlive>"#,
        "test.vue",
    );

    assert_eq!(result.warning_count, 1);
    assert_eq!(
        result.diagnostics[0].rule_name,
        "vue/no-non-component-keep-alive-child"
    );
}

#[test]
fn reports_plain_branch_wrappers() {
    let result = create_linter().lint_template(
        r#"<keep-alive><div v-if="a"><One /></div><section v-else><Two /></section></keep-alive>"#,
        "test.vue",
    );

    assert_eq!(result.warning_count, 2);
}

#[test]
fn reports_plain_child_inside_transition_below_keep_alive() {
    let result = create_linter().lint_template(
        r#"<KeepAlive><Transition><main v-if="ready"><Screen /></main></Transition></KeepAlive>"#,
        "test.vue",
    );

    assert_eq!(result.warning_count, 1);
}

#[test]
fn accepts_component_children() {
    let result = create_linter().lint_template(
        r#"<KeepAlive><One v-if="a" /><Two v-else /></KeepAlive>"#,
        "test.vue",
    );

    assert_eq!(result.warning_count, 0);
}

#[test]
fn accepts_kebab_case_component_children() {
    let result = create_linter().lint_template(
        r#"<keep-alive><n-layout v-if="a" /><n-empty v-else /><router-view /></keep-alive>"#,
        "test.vue",
    );

    assert_eq!(result.warning_count, 0);
}

#[test]
fn accepts_dynamic_component_and_builtins() {
    let result = create_linter().lint_template(
        r#"
        <KeepAlive><component :is="view" /></KeepAlive>
        <KeepAlive><Suspense><Screen /></Suspense></KeepAlive>
        <KeepAlive><Teleport to="body"><Screen /></Teleport></KeepAlive>
        <KeepAlive><TransitionGroup><Screen /></TransitionGroup></KeepAlive>
        "#,
        "test.vue",
    );

    assert_eq!(result.warning_count, 0);
}

#[test]
fn ignores_v_show_only_wrapper() {
    let result = create_linter().lint_template(
        r#"<KeepAlive><div v-show="opened"><SomeComponent /></div></KeepAlive>"#,
        "test.vue",
    );

    assert_eq!(result.warning_count, 0);
}

#[test]
fn reports_v_show_wrapper_when_it_also_controls_identity() {
    let result = create_linter().lint_template(
        r#"<KeepAlive><div v-show="opened" :key="view"><SomeComponent /></div></KeepAlive>"#,
        "test.vue",
    );

    assert_eq!(result.warning_count, 1);
}

#[test]
fn ignores_standalone_transition_native_children() {
    let result = create_linter().lint_template(
        r#"<Transition><div v-if="open">content</div></Transition>"#,
        "test.vue",
    );

    assert_eq!(result.warning_count, 0);
}

fn whole(result: &crate::LintResult) -> serde_json::Value {
    serde_json::json!({
        "filename": result.filename,
        "error_count": result.error_count, "warning_count": result.warning_count,
        "diagnostics": result.diagnostics.iter().map(|d| serde_json::json!({
            "rule_name": d.rule_name, "severity": d.severity, "message": d.message,
            "start": d.start, "end": d.end, "help": d.help, "fix": d.fix,
            "labels": d.labels.iter().map(|l| serde_json::json!({
                "message": l.message, "start": l.start, "end": l.end
            })).collect::<Vec<_>>()
        })).collect::<Vec<_>>()
    })
}

fn warning_packet() -> serde_json::Value {
    serde_json::json!({
        "filename": "test.vue", "error_count": 0, "warning_count": 1,
        "diagnostics": [{
            "rule_name": "vue/no-non-component-keep-alive-child", "severity": "warning",
            "message": "<KeepAlive> can only cache component children; this direct child is a plain element",
            "start": 11, "end": 32,
            "help": "Move the condition or key onto the component child, or render a dynamic component directly with <component :is=\"...\">",
            "fix": null, "labels": []
        }]
    })
}

#[test]
fn changing_descendant_identity_reports_the_whole_wrapper_packet() {
    // Each frontier is below the same native VNode that KeepAlive cannot cache.
    for body in [
        r#"<component :is="view" />"#,
        r#"<Panel v-if="visible" />"#,
        r#"<PanelA v-if="visible" /><PanelB v-else />"#,
        r#"<section v-if="visible"><Panel /></section>"#,
        r#"<Panel :key="identity" />"#,
        r#"<section :key="identity"><Panel /></section>"#,
        r#"<template :key="identity"><Panel /></template>"#,
        r#"<Panel v-for="item in items" :key="item.id" />"#,
        r#"<template v-for="item in items" :key="item.id"><Panel /></template>"#,
        r#"<section v-for="item in items" :key="item.id"><Panel /></section>"#,
        r#"<my-panel :key="identity" />"#,
        r#"<component :is="`Panel${suffix}`" />"#,
        r#"<component v-if="!visible" :is="view" />"#,
        r#"<component v-if="Number(view) === 0" :is="view" />"#,
        r#"<component v-if="settings.enabled === 0" :is="view" />"#,
        r#"<component v-if="NaN === 0" :is="view" />"#,
    ] {
        let source = vize_l0::cstr!(r#"<KeepAlive><div v-show="opened">{body}</div></KeepAlive>"#);
        assert_eq!(
            whole(&create_linter().lint_template(&source, "test.vue")),
            warning_packet(),
            "{source}"
        );
    }
}

#[test]
fn static_identity_and_component_owned_visibility_keep_whole_clean_packets() {
    for body in [
        r#"<Panel />"#,
        r#"<Panel key="fixed" />"#,
        r#"<Panel :key="('fixed')" />"#,
        r#"<Panel :key="-1" />"#,
        r#"<section :key="1 + 2"><Panel /></section>"#,
        r#"<component is="Panel" />"#,
        r#"<component :is="('Panel' as const)" />"#,
        r#"<component :is="`Panel`" />"#,
        r#"<component :is="true ? 'Panel' : 'Other'" />"#,
        r#"<Panel v-if="true" />"#,
        r#"<component v-if="false" :is="view" /><Panel v-else />"#,
        r#"<Panel v-if="true" /><component v-else :is="view" />"#,
        r#"<Panel v-if="'fixed'" /><component v-else-if="visible" :is="view" />"#,
        r#"<section v-if="visible">text only</section><Panel />"#,
        r#"<slot :name="view" />"#,
        r#"<Unknown><component :is="view" /></Unknown>"#,
        r#"<KeepAlive><component :is="view" /></KeepAlive>"#,
        r#"<div v-for="item in items">{{ item }}</div>"#,
    ] {
        let source = vize_l0::cstr!(r#"<KeepAlive><div v-show="opened">{body}</div></KeepAlive>"#);
        assert_eq!(
            whole(&create_linter().lint_template(&source, "test.vue")),
            serde_json::json!({"filename":"test.vue", "error_count":0,
                "warning_count":0,"diagnostics":[]}),
            "{source}"
        );
    }
    for source in [
        r#"<KeepAlive><Panel v-show="opened" :key="identity" /></KeepAlive>"#,
        r#"<KeepAlive><component v-show="opened" :is="view" /></KeepAlive>"#,
        r#"<KeepAlive><PanelA v-if="visible" /><PanelB v-else /></KeepAlive>"#,
    ] {
        assert_eq!(
            whole(&create_linter().lint_template(source, "test.vue")),
            serde_json::json!({"filename":"test.vue", "error_count":0,
                "warning_count":0,"diagnostics":[]}),
            "{source}"
        );
    }
}

#[test]
fn normalized_if_branch_keys_and_for_nodes_preserve_wrapper_ownership() {
    use vize_l0::{Allocator, Box};
    use vize_relief::{
        ExpressionNode, ForNode, ForParseResult, IfBranchNode, IfNode, PropNode,
        SimpleExpressionNode, TemplateChildNode,
    };
    for kind in [
        "if",
        "branch",
        "branch-key",
        "for",
        "match-scope",
        "if-false",
        "branch-false",
        "if-unary-false",
        "branch-unary-false",
        "if-binary-false",
        "branch-binary-false",
        "if-logical-false",
        "branch-logical-false",
        "if-or-true",
        "branch-or-true",
        "if-array-true",
        "branch-array-true",
    ] {
        let allocator = Allocator::default();
        let allocator = &allocator;
        let source = if kind.contains("or-true") {
            r#"<KeepAlive><div v-show="opened"><Panel :key="true || visible" /></div></KeepAlive>"#
        } else if kind.contains("array-true") {
            r#"<KeepAlive><div v-show="opened"><Panel :key="[]" /></div></KeepAlive>"#
        } else if kind.contains("logical-false") {
            r#"<KeepAlive><div v-show="opened"><component :is="view" :key="visible && false" /></div></KeepAlive>"#
        } else if kind.contains("unary") {
            r#"<KeepAlive><div v-show="opened"><component :is="view" :key="!true" /></div></KeepAlive>"#
        } else if kind.contains("binary") {
            r#"<KeepAlive><div v-show="opened"><component :is="view" :key="0 === 1" /></div></KeepAlive>"#
        } else if kind.ends_with("false") {
            r#"<KeepAlive><div v-show="opened"><component :is="view" :key="false" /></div></KeepAlive>"#
        } else {
            r#"<KeepAlive><div v-show="opened"><Panel :key="identity" /></div></KeepAlive>"#
        };
        let (mut root, errors) = vize_armature::Parser::new(allocator, source).parse();
        assert!(errors.is_empty());
        let TemplateChildNode::Element(keep_alive) = root.children.first_mut().unwrap() else {
            panic!()
        };
        let TemplateChildNode::Element(wrapper) = keep_alive.children.first_mut().unwrap() else {
            panic!()
        };
        let mut children =
            std::mem::replace(&mut wrapper.children, vize_l0::Vec::new_in(&allocator));
        let TemplateChildNode::Element(panel) = children.first_mut().unwrap() else {
            panic!()
        };
        let key = panel.props.pop().unwrap();
        let loc = panel.loc.clone();
        let PropNode::Directive(key_dir) = &key else {
            panic!()
        };
        let Some(ExpressionNode::Simple(condition_exp)) = key_dir.exp.as_ref() else {
            panic!()
        };
        let condition = || {
            ExpressionNode::Simple(Box::new_in(
                SimpleExpressionNode::from_node(condition_exp),
                &allocator,
            ))
        };
        let node = if matches!(kind, "for" | "match-scope") {
            let mut parse_result = ForParseResult::new(condition(), None, None, None);
            parse_result.match_scope = kind == "match-scope";
            TemplateChildNode::For(Box::new_in(
                ForNode {
                    source: condition(),
                    value_alias: None,
                    key_alias: None,
                    object_index_alias: None,
                    parse_result,
                    children,
                    loc: loc.clone(),
                },
                &allocator,
            ))
        } else {
            let mut branch = IfBranchNode::new(
                allocator,
                (kind != "branch-key").then(condition),
                loc.clone(),
            );
            branch.children = children;
            if kind == "branch-key" {
                branch.user_key = Some(key);
            }
            if kind.starts_with("branch") && kind != "branch-key" {
                TemplateChildNode::IfBranch(Box::new_in(branch, &allocator))
            } else {
                let mut node = IfNode::new(allocator, loc.clone());
                node.branches.push(branch);
                TemplateChildNode::If(Box::new_in(node, &allocator))
            }
        };
        wrapper.children.push(node);
        let mut ctx = crate::context::LintContext::new(allocator, source, "test.vue");
        ctx.current_rule = "vue/no-non-component-keep-alive-child";
        super::check_children(&mut ctx, &keep_alive.children);
        assert_eq!(
            whole(&crate::LintResult {
                filename: "test.vue".into(),
                diagnostics: ctx.diagnostics,
                error_count: ctx.error_count,
                warning_count: ctx.warning_count,
            }),
            if kind == "match-scope" || kind.ends_with("false") || kind.ends_with("true") {
                serde_json::json!({"filename":"test.vue", "error_count":0,
                    "warning_count":0,"diagnostics":[]})
            } else {
                warning_packet()
            },
            "normalized {kind}"
        );
    }
}

#[test]
fn shared_vue_lifecycle_sources_preserve_complete_warning_and_repair_packets() {
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../tests/_fixtures/keepalive-vshow-identity.json"
    ))
    .unwrap();
    let clean = serde_json::json!({"filename":"test.vue", "error_count":0,
        "warning_count":0,"diagnostics":[]});
    for case in corpus["cases"].as_array().unwrap() {
        let expected = if case["warning"].as_bool().unwrap() {
            warning_packet()
        } else {
            clean.clone()
        };
        assert_eq!(
            whole(&create_linter().lint_template(case["source"].as_str().unwrap(), "test.vue")),
            expected,
            "{} / authored",
            case["id"]
        );
        assert_eq!(
            whole(&create_linter().lint_template(case["repair"].as_str().unwrap(), "test.vue")),
            clean,
            "{} / component-owned repair",
            case["id"]
        );
    }
}

#[test]
fn normalized_conditional_chains_keep_constant_truth_and_prior_unknown_ownership() {
    use vize_l0::{Allocator, Box};
    use vize_relief::{IfBranchNode, IfNode, PropNode, TemplateChildNode};
    let corpus: serde_json::Value = serde_json::from_str(include_str!(
        "../../../../../tests/_fixtures/keepalive-vshow-identity.json"
    ))
    .unwrap();
    for case in corpus["cases"].as_array().unwrap().iter().filter(|case| {
        matches!(
            case["id"].as_str().unwrap(),
            "constant-or-true"
                | "constant-array-true"
                | "constant-and-false-else"
                | "prior-unknown-before-constant-true"
                | "component-key-with-constant-true"
                | "ancestor-key-before-constant-true"
                | "unknown-logical-condition"
        )
    }) {
        let source = case["source"].as_str().unwrap();
        let allocator = Allocator::default();
        let allocator = &allocator;
        let (mut root, errors) = vize_armature::Parser::new(allocator, source).parse();
        assert!(errors.is_empty());
        let TemplateChildNode::Element(keep_alive) = root.children.first_mut().unwrap() else {
            panic!()
        };
        let TemplateChildNode::Element(wrapper) = keep_alive.children.first_mut().unwrap() else {
            panic!()
        };
        let mut node = IfNode::new(allocator, wrapper.loc.clone());
        let conditional_children = if case["id"] == "ancestor-key-before-constant-true" {
            let TemplateChildNode::Element(section) = wrapper.children.first_mut().unwrap() else {
                panic!()
            };
            &mut section.children
        } else {
            &mut wrapper.children
        };
        let children = std::mem::replace(conditional_children, vize_l0::Vec::new_in(&allocator));
        for child in children {
            let TemplateChildNode::Element(mut element) = child else {
                panic!()
            };
            let index = element
                .props
                .iter()
                .position(|prop| {
                    matches!(prop,
                PropNode::Directive(dir) if matches!(dir.name, "if" | "else-if" | "else"))
                })
                .unwrap();
            let PropNode::Directive(mut dir) = element.props.remove(index) else {
                panic!()
            };
            let mut branch = IfBranchNode::new(allocator, dir.exp.take(), element.loc.clone());
            if case["id"] == "component-key-with-constant-true" {
                branch.user_key = Some(element.props.pop().unwrap());
            }
            branch.children.push(TemplateChildNode::Element(element));
            node.branches.push(branch);
        }
        conditional_children.push(TemplateChildNode::If(Box::new_in(node, &allocator)));
        let mut ctx = crate::context::LintContext::new(allocator, source, "test.vue");
        ctx.current_rule = "vue/no-non-component-keep-alive-child";
        super::check_children(&mut ctx, &keep_alive.children);
        let expected = if case["warning"].as_bool().unwrap() {
            warning_packet()
        } else {
            serde_json::json!({"filename":"test.vue", "error_count":0,
                "warning_count":0,"diagnostics":[]})
        };
        assert_eq!(
            whole(&crate::LintResult {
                filename: "test.vue".into(),
                diagnostics: ctx.diagnostics,
                error_count: ctx.error_count,
                warning_count: ctx.warning_count,
            }),
            expected,
            "{} / normalized",
            case["id"]
        );
    }
}
