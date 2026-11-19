use crate::linter::Linter;
use crate::rule::RuleRegistry;
use crate::rules::vue::NoMultiSpaces;

fn gaps(source: &str) -> Vec<&str> {
    let mut registry = RuleRegistry::new();
    registry.register(Box::new(NoMultiSpaces::default()));
    let result = Linter::with_registry(registry).lint_sfc(source, "test.vue");
    result
        .diagnostics
        .iter()
        .map(|diagnostic| {
            let start = (diagnostic.start as usize).saturating_sub(6);
            let end = (diagnostic.end as usize + 6).min(source.len());
            source.get(start..end).unwrap_or_default()
        })
        .collect()
}

#[test]
fn reports_gaps_between_block_tag_tokens() {
    assert_eq!(
        gaps(concat!(
            "<script setup  lang=\"ts\" generic=\"T\">\n</script  >\n",
            "<template  lang=\"html\"><div /></template>\n",
            "<style  scoped>\n</style>\n",
            "<i18n lang = \"json\"   >{}</i18n>\n",
        )),
        [
            " setup  lang=\"",
            "script  >\n<tem",
            "mplate  lang=\"",
            "<style  scoped",
            "\"json\"   >{}</i",
        ]
    );
}

#[test]
fn ignores_line_breaks_and_quoted_values() {
    assert_eq!(
        gaps(concat!(
            "<script\n  setup\n  lang=\"ts\"\n>\n</script>\n",
            "<template><div /></template>\n",
            "<style lang=\"scss\"   \n  scoped>\n</style>\n",
            "<i18n locale=\"en  US\" src=\"a\n.json\"  lang=\"json\">{}</i18n>\n",
        )),
        Vec::<&str>::new()
    );
}
