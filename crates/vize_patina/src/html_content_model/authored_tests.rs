//! The linter's parse is reused only when it is the tree as authored.

#[cfg(test)]
mod reuse_tests {
    use crate::html_content_model::{authored_skeleton, built_as_authored, template_skeleton};
    use vize_armature::Parser;
    use vize_l0::Allocator;

    fn reused(source: &str) -> bool {
        let allocator = Allocator::with_capacity(source.len() * 8 + 4096);
        let (root, _) = Parser::new(&allocator, source).parse();
        built_as_authored(&root, source)
    }

    /// Whether a tree the HTML tree construction repaired would be reused.
    fn repaired_reused(source: &str) -> bool {
        let allocator = Allocator::with_capacity(source.len() * 8 + 4096);
        let options = vize_armature::ParserOptions {
            html_tree_construction: true,
            ..vize_armature::ParserOptions::default()
        };
        let (root, _) = Parser::with_options(&allocator, source, options).parse();
        built_as_authored(&root, source)
    }

    /// Whenever the linter's (`Standard`) parse is reused, its skeleton is the
    /// one the non-repairing re-read builds.
    fn assert_equivalent(source: &str) -> bool {
        let allocator = Allocator::with_capacity(source.len() * 8 + 4096);
        let (root, _) = Parser::new(&allocator, source).parse();
        let reused = built_as_authored(&root, source);
        assert_eq!(
            template_skeleton(&allocator, source, &root),
            authored_skeleton(&allocator, source),
            "{source}"
        );
        reused
    }

    #[test]
    fn ordinary_templates_reuse_the_linter_parse() {
        for source in [
            "<div><p>text <b>bold</b></p><img src=\"a.png\"><br><input /></div>",
            "<ul><li v-for=\"i in l\" :key=\"i\">{{ i }}</li></ul>",
            "<table><tbody><tr><td>cell</td></tr></tbody></table>",
            // Vue keeps a direct row; the component parse implies no `tbody`.
            "<table><tr><td>row</td></tr></table>",
            "<svg viewBox=\"0 0 1 1\"><path d=\"M0 0\" /><foreignObject><div>x</div></foreignObject></svg>",
            "<MyCard title=\"x\"><template #footer><button>ok</button></template></MyCard>",
            "<!-- comment --><p>a</p>\n<p>b</p >",
            "<textarea>{{ raw }}</textarea><select><option>A</option></select>",
        ] {
            assert!(reused(source), "{source}");
        }
    }

    #[test]
    fn repaired_trees_are_read_again() {
        for source in [
            "<p><div>block</div></p>",
            "<p>unclosed<p>second</p>",
            "<table><div>fostered</div></table>",
            "<table>text</table>",
            "<ul><li>a<li>b</li></li></ul>",
            "<a href=\"#\"><a href=\"#\">nested</a></a>",
            "<button><button>nested</button></button>",
            "<select><option>a<option>b</option></option></select>",
            "<b><p>formatting</b></p>",
        ] {
            assert!(!repaired_reused(source), "{source}");
        }
    }

    /// Every nesting of two elements (and three for a smaller set), each
    /// closed explicitly: reused or not, the skeleton is the authored one.
    #[test]
    fn reuse_is_equivalent_to_reading_again() {
        const TAGS: [&str; 24] = [
            "div",
            "p",
            "span",
            "a href=\"#\"",
            "button",
            "ul",
            "li",
            "dl",
            "dt",
            "table",
            "tbody",
            "tr",
            "td",
            "select",
            "option",
            "form",
            "h1",
            "svg",
            "math",
            "template",
            "textarea",
            "label",
            "MyCard",
            "b",
        ];
        const VOID: [&str; 3] = ["img", "input", "br"];
        let name = |tag: &str| tag.split(' ').next().unwrap_or(tag).to_owned();
        let open = |tag: &str| ["<", tag, ">"].concat();
        let close = |tag: &str| ["</", &name(tag), ">"].concat();
        let mut read_again = Vec::new();
        let mut total = 0;
        for outer in TAGS {
            for inner in TAGS.iter().chain(VOID.iter()) {
                let body = if VOID.contains(inner) {
                    open(inner)
                } else {
                    [open(inner), "t".to_owned(), close(inner)].concat()
                };
                for middle in ["", "span", "div", "tr"] {
                    let source = if middle.is_empty() {
                        [open(outer), body.clone(), close(outer)].concat()
                    } else {
                        [
                            open(outer),
                            open(middle),
                            body.clone(),
                            close(middle),
                            close(outer),
                        ]
                        .concat()
                    };
                    total += 1;
                    if !assert_equivalent(&source) {
                        read_again.push(source);
                    }
                }
            }
        }
        // The linter's parse builds a Vue template as written, so it is reused
        // for every nesting but a `<textarea>` in a `<textarea>`: the first end
        // tag closes the outer one and leaves the second stray.
        assert_eq!(total, 24 * 27 * 4);
        assert_eq!(
            read_again,
            [
                "<textarea><textarea>t</textarea></textarea>",
                "<textarea><span><textarea>t</textarea></span></textarea>",
                "<textarea><div><textarea>t</textarea></div></textarea>",
                "<textarea><tr><textarea>t</textarea></tr></textarea>",
            ]
        );
    }
}
