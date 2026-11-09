//! Dev-only AST comparison supplements the independent native scope goldens.

#![cfg(test)]

use vize_armature::{ExpressionNode, PropNode, TemplateChildNode, parse};
use vize_l0::{Allocator, String, cstr};
use vize_l1::markup::parse_component;
use vize_l1::markup::{SurfaceChild, check_fidelity};

fn native_interpolations<'a>(children: &[SurfaceChild<'a>], values: &mut Vec<&'a str>) {
    for child in children {
        match child {
            SurfaceChild::Interpolation(node) => values.push(node.content.text.trim()),
            SurfaceChild::Element(node) => native_interpolations(&node.children, values),
            _ => {}
        }
    }
}

fn legacy_interpolations<'a>(children: &[TemplateChildNode<'a>], values: &mut Vec<&'a str>) {
    for child in children {
        match child {
            TemplateChildNode::Interpolation(node) => match &node.content {
                ExpressionNode::Simple(expression) => values.push(expression.content),
                ExpressionNode::Compound(_) => panic!("ordinary fixture interpolation"),
            },
            TemplateChildNode::Element(node) => legacy_interpolations(&node.children, values),
            _ => {}
        }
    }
}

fn interpolation_policy(source: &str) -> (Vec<std::string::String>, Vec<std::string::String>) {
    let allocator = Allocator::new();
    let parsed = parse_component(&allocator, source).unwrap();
    assert!(parsed.unsupported.is_empty(), "{source}");
    let legacy_allocator = vize_armature::Allocator::new();
    let (legacy, _) = parse(&legacy_allocator, source);
    let mut native_values = Vec::new();
    let mut legacy_values = Vec::new();
    native_interpolations(&parsed.tree.children, &mut native_values);
    legacy_interpolations(&legacy.children, &mut legacy_values);
    assert_eq!(check_fidelity(&parsed.tree), Ok(()), "{source}");
    (
        native_values.into_iter().map(str::to_owned).collect(),
        legacy_values.into_iter().map(str::to_owned).collect(),
    )
}

fn assert_ast_policy(source: &str) {
    let (native, legacy) = interpolation_policy(source);
    assert_eq!(native, legacy, "{source}");
}

#[test]
fn native_mode_uses_the_actual_legacy_parsed_name_policy() {
    for head in [
        "v-pre",
        "v-pre.foo",
        "v-pre:arg",
        "v-pre:[keys['a.b']]",
        "v-pre:[broken",
        "v-pre:.",
        "v-pre..",
        "v-pre:[",
        "v-pre:",
        "v-pretty",
        "v-prefix",
        "v-PRE",
        ":pre",
        "@pre",
        "#pre",
        ".pre",
        "class",
    ] {
        let source = cstr!(
            "<!--中🍣--><div :prior='old' {head} :later='value'>{{{{ msg }}}}<span @click='handler'>{{{{ nested }}}}</span></div>{{{{ tail }}}}"
        );
        assert_ast_policy(&source);
    }
}

#[test]
fn no_colon_bracket_name_keeps_native_interpolations_while_legacy_retains_its_old_policy() {
    let source = "<div v-pre[broken>{{msg}}<span>{{nested}}</span></div>{{tail}}";
    let allocator = Allocator::new();
    let parsed = parse_component(&allocator, source).unwrap();
    assert!(parsed.unsupported.is_empty());
    assert_eq!(check_fidelity(&parsed.tree), Ok(()));
    let mut native_values = Vec::new();
    native_interpolations(&parsed.tree.children, &mut native_values);
    assert_eq!(native_values, ["msg", "nested", "tail"]);

    // The preserved legacy tokenizer accepts '[' without ':' as an argument.
    // The pinned upstream Vue 3.5.35 oracle proves the native full-name rule.
    let legacy_allocator = vize_armature::Allocator::new();
    let (legacy, _) = parse(&legacy_allocator, source);
    let mut legacy_values = Vec::new();
    legacy_interpolations(&legacy.children, &mut legacy_values);
    assert_eq!(legacy_values, ["tail"]);
}

#[test]
fn native_recovered_scopes_match_live_legacy_ast_interpolation_admission() {
    for source in [
        "<div v-pre><span v-pre>{{inner}}</span>{{outer}}</div>{{normal}}",
        "<div v-pre><span>{{inner}}</div>{{normal}}",
        "<div v-pre></stray>{{inner}}</div>{{normal}}",
        "<Comp v-pre/>{{normal}}",
        "<input v-pre>{{normal}}",
        "<div v-pre><input v-pre>{{inner}}<Comp v-pre/>{{more}}</div>{{normal}}",
        "<svg><a v-pre><a>{{inner}}</a>{{outer}}</a></svg>{{tail}}",
        "<math><button v-pre><button>{{inner}}</button></button></math>{{tail}}",
    ] {
        assert_ast_policy(source);
    }
}

#[test]
fn authored_vue_scopes_classify_known_legacy_recovery_differences() {
    for (source, native_expected, legacy_expected) in [
        (
            "<a v-pre><span>{{before}}<a>{{after}}</a>{{tail}}</span></a>{{end}}",
            vec!["end"],
            vec!["after", "tail", "end"],
        ),
        (
            "<button v-pre><button>{{after}}</button>{{tail}}</button>{{end}}",
            vec!["end"],
            vec!["after", "tail", "end"],
        ),
        (
            "<svg><foreignObject><button v-pre><button>{{after}}</button></button></foreignObject></svg>{{tail}}",
            vec!["tail"],
            vec!["after", "tail"],
        ),
        (
            "<section><a><span><a></a><span v-pre>{{inside}}</span>{{tail}}</span></a></section>",
            vec!["tail"],
            vec![],
        ),
    ] {
        let (native, legacy) = interpolation_policy(source);
        assert_eq!(native, native_expected, "Vue authored scope: {source}");
        assert_eq!(
            legacy, legacy_expected,
            "retained legacy behavior: {source}"
        );
    }
}

#[test]
fn same_tag_pre_proves_unrelated_directive_heads_are_raw_regardless_of_order() {
    let opens: String = (0..64)
        .map(|i| if i % 2 == 0 { '(' } else { '[' })
        .collect();
    let closes: String = opens
        .chars()
        .rev()
        .map(|c| if c == '(' { ')' } else { ']' })
        .collect();
    let head = cstr!("v-bind:[{opens}key{closes}]");
    for attrs in [cstr!("{head} v-pre"), cstr!("v-pre {head}")] {
        let source =
            cstr!("<div {attrs}>{{{{ raw }}}}<span>{{{{ nested }}}}</span></div>{{{{ tail }}}}");
        let allocator = vize_armature::Allocator::new();
        let (legacy, _) = parse(&allocator, &source);
        let TemplateChildNode::Element(div) = &legacy.children[0] else {
            panic!("div")
        };
        assert_eq!(div.props.len(), 1, "control is consumed: {source}");
        assert!(
            matches!(&div.props[0], PropNode::Attribute(_)),
            "unrelated directive is raw: {source}"
        );
        let mut values = Vec::new();
        legacy_interpolations(&legacy.children, &mut values);
        assert_eq!(values, ["tail"], "{source}");
        assert_ast_policy(&source);
    }
}

#[test]
fn native_keeps_authored_outer_pre_where_legacy_loses_its_owner() {
    // Vue's outer lexical scope survives HTML repair. The legacy parser still
    // loses that owner; production routes retain that behavior until migrated.
    let source = "<a v-pre><a v-pre>{{inner}}</a>{{tail}}</a>{{end}}";
    let allocator = Allocator::new();
    let parsed = parse_component(&allocator, source).unwrap();
    let legacy_allocator = vize_armature::Allocator::new();
    let (legacy, _) = parse(&legacy_allocator, source);
    let mut native_values = Vec::new();
    let mut legacy_values = Vec::new();
    native_interpolations(&parsed.tree.children, &mut native_values);
    legacy_interpolations(&legacy.children, &mut legacy_values);
    assert_eq!(native_values, ["end"]);
    assert_eq!(legacy_values, ["inner", "tail", "end"]);
    assert!(parsed.unsupported.is_empty());
    assert_eq!(check_fidelity(&parsed.tree), Ok(()));
}
