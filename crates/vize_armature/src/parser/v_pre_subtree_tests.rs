//! Every parse keeps a v-pre subtree plain, as Vue does (#13314).

use super::Parser;
use vize_l0::Allocator;
use vize_relief::options::ParserOptions;
use vize_relief::{ElementType, PropNode, RootNode, TemplateChildNode};

fn dom_options() -> ParserOptions {
    ParserOptions {
        is_native_tag: Some(vize_l0::is_native_tag),
        ..Default::default()
    }
}

fn types<'a>(children: &[TemplateChildNode<'a>], out: &mut Vec<(&'a str, ElementType)>) {
    for child in children {
        if let TemplateChildNode::Element(element) = child {
            out.push((element.tag, element.tag_type));
            types(&element.children, out);
        }
    }
}

fn shape(children: &[TemplateChildNode<'_>], out: &mut String) {
    for child in children {
        if let TemplateChildNode::Element(element) = child {
            out.push_str(element.tag);
            out.push('(');
            shape(&element.children, out);
            out.push(')');
        }
    }
}

fn attribute_names<'a>(children: &[TemplateChildNode<'a>], out: &mut Vec<&'a str>) {
    for child in children {
        if let TemplateChildNode::Element(element) = child {
            for prop in &element.props {
                match prop {
                    PropNode::Attribute(attribute) => out.push(attribute.name),
                    PropNode::Directive(_) => panic!("a v-pre directive stayed semantic"),
                }
            }
            attribute_names(&element.children, out);
        }
    }
}

/// The ordinary parse, then both compiler custody entries.
fn each_entry(source: &str, mut check: impl FnMut(usize, &RootNode<'_>, &[vize_l0::Span])) {
    for entry in 0..3 {
        let arena = Allocator::new();
        let parser = Parser::with_options(&arena, source, dom_options());
        let (root, errors, spans) = match entry {
            0 => {
                let (root, errors) = parser.parse();
                (root, errors, None)
            }
            1 => {
                let (root, errors, frozen) = parser.parse_with_frozen_elements();
                let spans = frozen.spans_for(root.source, source).map(<[_]>::to_vec);
                (root, errors, spans)
            }
            _ => {
                let (root, errors, frozen) = parser.parse_with_frozen_element_spans();
                let spans = frozen.spans_for(root.source, source).map(<[_]>::to_vec);
                (root, errors, spans)
            }
        };
        assert!(errors.is_empty(), "{source} entry {entry}: {errors:?}");
        check(entry, &root, spans.as_deref().unwrap_or_default());
    }
}

#[test]
fn every_parse_entry_keeps_the_opener_and_its_subtree_plain() {
    use ElementType::{Component, Element, Slot, Template};
    let cases: &[(&str, &[(&str, ElementType)])] = &[
        (
            "<div v-pre><div><Child>{{ x }}</Child></div></div>",
            &[("div", Element), ("div", Element), ("Child", Element)],
        ),
        ("<Child v-pre>{{ x }}</Child>", &[("Child", Element)]),
        (
            "<div v-pre><component is=\"x\"/></div>",
            &[("div", Element), ("component", Element)],
        ),
        (
            "<div v-pre><my-comp/></div>",
            &[("div", Element), ("my-comp", Element)],
        ),
        (
            "<div v-pre><slot/></div>",
            &[("div", Element), ("slot", Element)],
        ),
        (
            "<div v-pre><template v-if=\"a\">x</template></div>",
            &[("div", Element), ("template", Element)],
        ),
        (
            "<slot v-pre></slot><template v-pre v-if=\"a\">x</template><KeepAlive v-pre></KeepAlive>",
            &[
                ("slot", Element),
                ("template", Element),
                ("KeepAlive", Element),
            ],
        ),
        (
            "<div><Child/><slot/><template v-if=\"a\">x</template></div>",
            &[
                ("div", Element),
                ("Child", Component),
                ("slot", Slot),
                ("template", Template),
            ],
        ),
    ];
    for (source, expected) in cases {
        each_entry(source, |entry, root, _| {
            let mut actual = Vec::new();
            types(&root.children, &mut actual);
            assert_eq!(actual, *expected, "{source} entry {entry}");
        });
    }
}

#[test]
fn a_frozen_component_tag_keeps_its_html_tree_construction() {
    // Fostering and scope boundaries read the type outside v-pre: a literal
    // component neither leaves its table nor lets a `<div>` close its `<p>`.
    for (frozen, control) in [
        (
            "<table v-pre><tr><Row></Row></tr></table>",
            "<table><tr><Row></Row></tr></table>",
        ),
        (
            "<p v-pre><Child><div></div></Child></p>",
            "<p><Child><div></div></Child></p>",
        ),
    ] {
        let mut shapes = Vec::new();
        for source in [frozen, control] {
            each_entry(source, |_, root, _| {
                let mut actual = String::new();
                shape(&root.children, &mut actual);
                shapes.push(actual);
            });
        }
        // Three parse entries for each source; the unfrozen control sets the shape.
        let expected = vec![shapes[5].clone(); 6];
        assert_eq!(shapes, expected, "{frozen}");
    }
}

#[test]
fn span_custody_alone_keeps_the_default_directive_names() {
    let source = "<div v-pre v-bind:x=\"1\"><span v-bind:y=\"2\" v-my:a.m=\"v\"></span><Child></Child></div>";
    let mut spans = Vec::new();
    each_entry(source, |entry, root, frozen| {
        let mut names = Vec::new();
        attribute_names(&root.children, &mut names);
        let expected: &[&str] = if entry == 1 {
            &["v-bindx", "v-bindy", "v-mya.m"]
        } else {
            &["v-bindx", "v-bind:y", "v-my:a.m"]
        };
        assert_eq!(names, expected, "entry {entry}");
        let frozen: Vec<_> = frozen.iter().map(|span| span.slice(source)).collect();
        spans.push(frozen);
    });
    assert_eq!(
        spans,
        [
            vec![],
            vec![
                "<div v-pre v-bind:x=\"1\">",
                "<span v-bind:y=\"2\" v-my:a.m=\"v\">",
                "<Child>"
            ],
            vec![
                "<div v-pre v-bind:x=\"1\">",
                "<span v-bind:y=\"2\" v-my:a.m=\"v\">",
                "<Child>"
            ],
        ]
    );
}
