//! Dev-only AST comparison supplements the independent native scope goldens.

#![cfg(test)]

use vize_armature::{ExpressionNode, PropNode, TemplateChildNode, parse};
use vize_l0::{Allocator, String, cstr};
use vize_l1::markup::parse_component;
use vize_l1::{SurfaceChild, check_fidelity};

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

fn assert_ast_policy(source: &str) {
    let allocator = Allocator::new();
    let parsed = parse_component(&allocator, source).unwrap();
    assert!(parsed.unsupported.is_empty(), "{source}");
    let legacy_allocator = vize_armature::Allocator::new();
    let (legacy, _) = parse(&legacy_allocator, source);
    let mut native_values = Vec::new();
    let mut legacy_values = Vec::new();
    native_interpolations(&parsed.tree.children, &mut native_values);
    legacy_interpolations(&legacy.children, &mut legacy_values);
    assert_eq!(native_values, legacy_values, "{source}");
    assert_eq!(check_fidelity(&parsed.tree), Ok(()), "{source}");
}

#[test]
fn native_mode_uses_the_actual_legacy_parsed_name_policy() {
    for head in [
        "v-pre",
        "v-pre.foo",
        "v-pre:arg",
        "v-pre:[keys['a.b']]",
        "v-pre:[broken",
        "v-pre[broken",
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
fn native_recovered_scopes_match_live_legacy_ast_interpolation_admission() {
    for source in [
        "<div v-pre><span v-pre>{{inner}}</span>{{outer}}</div>{{normal}}",
        "<div v-pre><span>{{inner}}</div>{{normal}}",
        "<div v-pre></stray>{{inner}}</div>{{normal}}",
        "<Comp v-pre/>{{normal}}",
        "<input v-pre>{{normal}}",
        "<div v-pre><input v-pre>{{inner}}<Comp v-pre/>{{more}}</div>{{normal}}",
        "<a v-pre><span>{{before}}<a>{{after}}</a>{{tail}}</span></a>{{end}}",
        "<button v-pre><button>{{after}}</button>{{tail}}</button>{{end}}",
        "<svg><a v-pre><a>{{inner}}</a>{{outer}}</a></svg>{{tail}}",
        "<math><button v-pre><button>{{inner}}</button></button></math>{{tail}}",
        "<svg><foreignObject><button v-pre><button>{{after}}</button></button></foreignObject></svg>{{tail}}",
        "<section><a><span><a></a><span v-pre>{{inside}}</span>{{tail}}</span></a></section>",
    ] {
        assert_ast_policy(source);
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
fn native_recovered_independent_owner_honors_its_own_pre_after_implicit_close() {
    // Native policy is resolved after shared structural recovery. Legacy still
    // classifies the second control head under the implicitly closed owner and
    // loses it. Production routes retain that legacy behavior unchanged.
    let source = "<a v-pre><a v-pre>{{inner}}</a>{{tail}}</a>{{end}}";
    let allocator = Allocator::new();
    let parsed = parse_component(&allocator, source).unwrap();
    let legacy_allocator = vize_armature::Allocator::new();
    let (legacy, _) = parse(&legacy_allocator, source);
    let mut native_values = Vec::new();
    let mut legacy_values = Vec::new();
    native_interpolations(&parsed.tree.children, &mut native_values);
    legacy_interpolations(&legacy.children, &mut legacy_values);
    assert_eq!(native_values, ["tail", "end"]);
    assert_eq!(legacy_values, ["inner", "tail", "end"]);
    assert!(parsed.unsupported.is_empty());
    assert_eq!(check_fidelity(&parsed.tree), Ok(()));
}
