//! Composition fixtures: the verdicts a parent's context proves in a child
//! template, and the ones it must leave alone.

#[cfg(test)]
mod composition_tests {
    use crate::html_content_model::{NodeKind, Skeleton, authored_skeleton, compose};
    use vize_s0::Allocator;

    /// Compose `files` (name, template) where a component tag resolves to the
    /// file of the same name; render each finding as
    /// `usage-path | child-file node class <- evidence-file node`.
    fn composed(files: &[(&str, &str)]) -> Vec<String> {
        let skeletons: Vec<Skeleton> = files
            .iter()
            .map(|(_, source)| {
                let allocator = Allocator::with_capacity(source.len() * 4 + 1024);
                authored_skeleton(&allocator, source)
            })
            .collect();
        let resolve = |_: u32, tag: &str| {
            files
                .iter()
                .position(|(name, _)| *name == tag)
                .map(|index| index as u32)
        };
        let label = |file: u32, node: u32| {
            let (name, source) = files[file as usize];
            let entry = skeletons[file as usize].node(node);
            let span = match &entry.kind {
                NodeKind::Element(element) => element.name_span,
                _ => entry.span,
            };
            format!("{name}:{}", &source[span.start as usize..span.end as usize])
        };
        compose(&skeletons, &resolve)
            .into_iter()
            .map(|finding| {
                let path: Vec<String> = finding
                    .usages
                    .iter()
                    .map(|(file, node)| label(*file, *node))
                    .collect();
                let evidence = finding
                    .evidence
                    .map_or_else(|| "-".to_string(), |(file, node)| label(file, node));
                format!(
                    "{} | {} {} <- {}",
                    path.join(" > "),
                    label(finding.file, finding.node),
                    finding.class.id(),
                    evidence
                )
            })
            .collect()
    }

    #[test]
    fn child_root_collides_with_the_parent_chain() {
        assert_eq!(
            composed(&[("App", "<p><Card /></p>"), ("Card", "<div>body</div>")]),
            ["App:<Card /> | Card:div paragraph-auto-closed <- App:p"]
        );
        // The lint parser does not mark `<my-card>` as a component. It still
        // renders the imported component, whose `<div>` root closes the `<p>`.
        assert_eq!(
            composed(&[
                ("App", "<p><my-card /></p>"),
                ("my-card", "<div>body</div>")
            ]),
            ["App:my-card | my-card:div paragraph-auto-closed <- App:p"]
        );
        assert_eq!(
            composed(&[
                ("App", "<table><Row /></table>"),
                ("Row", "<tr><td /></tr>")
            ]),
            ["App:<Row /> | Row:tr table-wrapper-inserted <- App:table"]
        );
        assert_eq!(
            composed(&[
                ("App", "<a href=\"#\"><Btn /></a>"),
                ("Btn", "<button>x</button>")
            ]),
            ["App:<Btn /> | Btn:button interactive-content-nested <- App:a"]
        );
    }

    #[test]
    fn conforming_compositions_stay_silent() {
        for files in [
            [("App", "<p><Badge /></p>"), ("Badge", "<span>ok</span>")],
            [
                ("App", "<table><tbody><Row /></tbody></table>"),
                ("Row", "<tr><td /></tr>"),
            ],
            [("App", "<div><Card /></div>"), ("Card", "<div>body</div>")],
            [
                ("App", "<p><my-badge /></p>"),
                ("my-badge", "<span>ok</span>"),
            ],
            // Hyphenated native tags are not component usages.
            [
                ("App", "<svg><color-profile /></svg>"),
                ("color-profile", "<div>x</div>"),
            ],
            [("App", "<ul><Item /></ul>"), ("Item", "<li>item</li>")],
        ] {
            assert_eq!(composed(&files), Vec::<String>::new());
        }
    }

    #[test]
    fn every_alternative_root_and_fragment_root_is_checked() {
        assert_eq!(
            composed(&[
                ("App", "<p><Card /></p>"),
                ("Card", "<span v-if=\"a\">a</span><div v-else>b</div>"),
            ]),
            ["App:<Card /> | Card:div paragraph-auto-closed <- App:p"]
        );
        assert_eq!(
            composed(&[
                ("App", "<p><Pair /></p>"),
                ("Pair", "<em>a</em><ul><li /></ul>")
            ]),
            ["App:<Pair /> | Pair:ul paragraph-auto-closed <- App:p"]
        );
    }

    #[test]
    fn chains_follow_nested_roots_and_stop_at_unknowns() {
        assert_eq!(
            composed(&[
                ("App", "<p><Outer /></p>"),
                ("Outer", "<Inner />"),
                ("Inner", "<section>deep</section>"),
            ]),
            ["App:<Outer /> > Outer:<Inner /> | Inner:section paragraph-auto-closed <- App:p"]
        );
        // Unresolved components, slot pass-through and recursion stay unknown.
        for files in [
            vec![("App", "<p><Missing /></p>")],
            vec![("App", "<p><Wrap><div /></Wrap></p>"), ("Wrap", "<slot />")],
            vec![("App", "<p><Loop /></p>"), ("Loop", "<Loop />")],
        ] {
            assert_eq!(composed(&files), Vec::<String>::new());
        }
    }
}
