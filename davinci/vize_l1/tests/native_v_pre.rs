//! Independent native scope goldens, not equality between two event recorders.

#![cfg(test)]

use vize_l0::{Allocator, Span, String, cstr};
use vize_l1::markup::{DirectiveNameError, parse_component, parse_component_with_authored};
use vize_l1::{SurfaceChild, check_fidelity};

fn interpolations<'a>(children: &[SurfaceChild<'a>]) -> Vec<&'a str> {
    fn collect<'a>(children: &[SurfaceChild<'a>], values: &mut Vec<&'a str>) {
        for child in children {
            match child {
                SurfaceChild::Interpolation(node) => values.push(node.content.text),
                SurfaceChild::Element(node) => collect(&node.children, values),
                _ => {}
            }
        }
    }
    let mut values = Vec::new();
    collect(children, &mut values);
    values
}

#[test]
fn full_parsed_pre_heads_suppress_nested_content_and_keep_every_authored_attribute() {
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
    ] {
        let source = cstr!(
            "<!--中🍣--><div :prior='old' {head} :later='value'>{{{{ msg }}}}<span @click='handler'>{{{{ nested }}}}</span></div>{{{{ tail }}}}"
        );
        let allocator = Allocator::new();
        let parsed = parse_component(&allocator, &source).unwrap();
        assert!(parsed.unsupported.is_empty(), "{source}");
        assert_eq!(check_fidelity(&parsed.tree), Ok(()), "{source}");
        assert_eq!(
            interpolations(&parsed.tree.children),
            [" tail "],
            "{source}"
        );
        let SurfaceChild::Element(div) = &parsed.tree.children[1] else {
            panic!("{source}")
        };
        assert_eq!(
            div.open
                .attrs
                .iter()
                .map(|attr| attr.name.text)
                .collect::<Vec<_>>(),
            [":prior", head, ":later"],
            "{source}"
        );
        assert!(matches!(&div.children[0], SurfaceChild::Text(text) if text.text == "{{ msg }}"));
        let SurfaceChild::Element(span) = &div.children[1] else {
            panic!("{source}")
        };
        assert_eq!(span.open.attrs[0].name.text, "@click");
        assert!(
            matches!(&span.children[0], SurfaceChild::Text(text) if text.text == "{{ nested }}")
        );
    }
}

#[test]
fn shorthand_and_similar_names_do_not_start_verbatim_scopes() {
    for head in [
        "v-pretty", "v-prefix", "v-PRE", ":pre", "@pre", "#pre", ".pre", "class",
    ] {
        let source =
            cstr!("<div {head}>{{{{ msg }}}}<span>{{{{ nested }}}}</span></div>{{{{ tail }}}}");
        let allocator = Allocator::new();
        let parsed = parse_component(&allocator, &source).unwrap();
        assert!(parsed.unsupported.is_empty());
        assert_eq!(
            interpolations(&parsed.tree.children),
            [" msg ", " nested ", " tail "],
            "{source}"
        );
    }
}

#[test]
fn matching_ancestors_stray_closes_void_and_self_closing_tags_restore_the_owner_mode() {
    for (source, expected) in [
        (
            "<div v-pre><span v-pre>{{inner}}</span>{{outer}}</div>{{normal}}",
            vec!["normal"],
        ),
        ("<div v-pre><span>{{inner}}</div>{{normal}}", vec!["normal"]),
        (
            "<div v-pre></stray>{{inner}}</div>{{normal}}",
            vec!["normal"],
        ),
        ("<Comp v-pre/>{{normal}}", vec!["normal"]),
        ("<input v-pre>{{normal}}", vec!["normal"]),
        (
            "<div v-pre><input v-pre>{{inner}}<Comp v-pre/>{{more}}</div>{{normal}}",
            vec!["normal"],
        ),
        ("<DIV v-pre>{{inner}}</div>{{normal}}", vec!["normal"]),
    ] {
        let allocator = Allocator::new();
        let parsed = parse_component(&allocator, source).unwrap();
        assert!(parsed.unsupported.is_empty());
        assert_eq!(interpolations(&parsed.tree.children), expected, "{source}");
        assert_eq!(check_fidelity(&parsed.tree), Ok(()), "{source}");
    }
}

#[test]
fn implicit_interactive_recovery_and_foreign_namespaces_share_builder_scope() {
    for (source, expected, authored) in [
        (
            "<a v-pre><span>{{before}}<a>{{after}}</a>{{tail}}</span></a>{{end}}",
            vec!["after", "tail", "end"],
            true,
        ),
        (
            "<button v-pre><button>{{after}}</button>{{tail}}</button>{{end}}",
            vec!["after", "tail", "end"],
            true,
        ),
        (
            "<a v-pre><a v-pre>{{inner}}</a>{{tail}}</a>{{end}}",
            vec!["tail", "end"],
            true,
        ),
        (
            "<svg><a v-pre><a>{{inner}}</a>{{outer}}</a></svg>{{tail}}",
            vec!["tail"],
            false,
        ),
        (
            "<math><button v-pre><button>{{inner}}</button></button></math>{{tail}}",
            vec!["tail"],
            false,
        ),
        (
            "<svg><foreignObject><button v-pre><button>{{after}}</button></button></foreignObject></svg>{{tail}}",
            vec!["after", "tail"],
            true,
        ),
        (
            "<section><a><span><a></a><span v-pre>{{inside}}</span>{{tail}}</span></a></section>",
            // This close claims the old, implicitly closed span. The new
            // shallower span remains open until </section>, including tail.
            vec![],
            true,
        ),
    ] {
        let allocator = Allocator::new();
        let parsed = parse_component_with_authored(&allocator, source).unwrap();
        assert!(parsed.unsupported.is_empty());
        assert_eq!(interpolations(&parsed.tree.children), expected, "{source}");
        assert_eq!(parsed.authored.is_some(), authored, "{source}");
        assert_eq!(check_fidelity(&parsed.tree), Ok(()), "{source}");
        if let Some(tree) = parsed.authored {
            assert_eq!(
                interpolations(&tree.children),
                expected,
                "shared event stream: {source}"
            );
            assert_eq!(check_fidelity(&tree), Ok(()));
        }
    }
}

#[test]
fn over_budget_heads_retain_typed_unsupported_facts_and_subsequent_nodes() {
    let opens: String = (0..64)
        .map(|i| if i % 2 == 0 { '(' } else { '[' })
        .collect();
    let closes: String = opens
        .chars()
        .rev()
        .map(|c| if c == '(' { ')' } else { ']' })
        .collect();
    let head = cstr!("v-pre:[{opens}key{closes}]");
    let source = cstr!(
        "<!--中🍣--><div {head}>{{{{ unresolved }}}}</div><p v-pre>{{{{ raw }}}}</p>{{{{ tail }}}}"
    );
    let allocator = Allocator::new();
    let parsed = parse_component(&allocator, &source).unwrap();
    let start = source.find(head.as_str()).unwrap() as u32;
    assert_eq!(parsed.unsupported.len(), 1);
    assert_eq!(
        parsed.unsupported[0].span,
        Span::new(start, start + head.len() as u32)
    );
    assert_eq!(
        parsed.unsupported[0].error,
        DirectiveNameError::NestingLimit
    );
    assert_eq!(
        interpolations(&parsed.tree.children),
        [" unresolved ", " tail "]
    );
    assert_eq!(check_fidelity(&parsed.tree), Ok(()));
    let SurfaceChild::Element(div) = &parsed.tree.children[1] else {
        panic!("div")
    };
    assert_eq!(div.open.attrs[0].name.text, head.as_str());

    // An inherited verbatim subtree ignores directive semantics altogether.
    // Its raw authored names must not be rejected by the unused syntax hook.
    let source = cstr!("<div v-pre><span {head}>{{{{ raw }}}}</span></div>{{{{ tail }}}}");
    let parsed = parse_component(&allocator, &source).unwrap();
    assert!(parsed.unsupported.is_empty());
    assert_eq!(interpolations(&parsed.tree.children), [" tail "]);
    assert_eq!(check_fidelity(&parsed.tree), Ok(()));
}

#[test]
fn same_tag_pre_ignores_unrelated_over_budget_heads_in_both_orders_and_keeps_prior_facts() {
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
        let source = cstr!(
            "<!--中🍣--><div {attrs}>{{{{ raw }}}}<span>{{{{ nested }}}}</span></div>{{{{ tail }}}}"
        );
        let allocator = Allocator::new();
        let parsed = parse_component(&allocator, &source).unwrap();
        assert!(parsed.unsupported.is_empty(), "{source}");
        assert_eq!(
            interpolations(&parsed.tree.children),
            [" tail "],
            "{source}"
        );
        assert_eq!(check_fidelity(&parsed.tree), Ok(()));
        let SurfaceChild::Element(div) = &parsed.tree.children[1] else {
            panic!("div")
        };
        assert_eq!(div.open.attrs.len(), 2);
        assert!(
            div.open
                .attrs
                .iter()
                .any(|attr| attr.name.text == head.as_str())
        );

        let source = cstr!(
            "<!--中🍣--><aside {head}>{{{{ prior }}}}</aside><div {attrs}>{{{{ raw }}}}</div>{{{{ tail }}}}"
        );
        let parsed = parse_component(&allocator, &source).unwrap();
        let start = source.find(head.as_str()).unwrap() as u32;
        assert_eq!(parsed.unsupported.len(), 1, "{source}");
        assert_eq!(
            parsed.unsupported[0].span,
            Span::new(start, start + head.len() as u32)
        );
        assert_eq!(
            parsed.unsupported[0].error,
            DirectiveNameError::NestingLimit
        );
        assert_eq!(
            interpolations(&parsed.tree.children),
            [" prior ", " tail "],
            "{source}"
        );
        assert_eq!(check_fidelity(&parsed.tree), Ok(()));

        let source =
            cstr!("<a v-pre><a {attrs}>{{{{ inner }}}}</a>{{{{ tail }}}}</a>{{{{ end }}}}");
        let parsed = parse_component_with_authored(&allocator, &source).unwrap();
        assert!(
            parsed.unsupported.is_empty(),
            "recovered independent owner: {source}"
        );
        assert_eq!(
            interpolations(&parsed.tree.children),
            [" tail ", " end "],
            "{source}"
        );
        assert_eq!(check_fidelity(&parsed.tree), Ok(()));
        let authored = parsed
            .authored
            .expect("interactive recovery retains authored projection");
        assert_eq!(interpolations(&authored.children), [" tail ", " end "]);
        assert_eq!(check_fidelity(&authored), Ok(()));
    }
}

#[test]
fn all_utf8_cuts_of_malformed_scopes_keep_recoverable_source_bytes() {
    for source in [
        "<!--中🍣--><div v-pre:[鍵[索引['値.']]] title='&fjlig;'>{{ raw }}<span @click='handler'>{{ nested }}</div>{{normal}}",
        "<button v-pre><span><button broken= >{{normal}}</span></button>{{tail}}",
        "<div v-pre:[broken :title='unfinished",
    ] {
        for (cut, _) in source.char_indices() {
            for fragment in [source.get(..cut).unwrap(), source.get(cut..).unwrap()] {
                let allocator = Allocator::new();
                let parsed = parse_component_with_authored(&allocator, fragment).unwrap();
                assert_eq!(check_fidelity(&parsed.tree), Ok(()), "{fragment}");
                if let Some(tree) = parsed.authored {
                    assert_eq!(check_fidelity(&tree), Ok(()));
                }
            }
        }
    }
}
