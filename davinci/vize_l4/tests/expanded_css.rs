use core::cell::Cell;
use vize_l0::{SourceBlock, SourceRoot, Span};
use vize_l2::style::{ExpandedRules, Property, RuleRun};
use vize_l4::{targets::css_rules, write::EmitDocument};

struct Sheet<'a> {
    block: SourceBlock<'a>,
    rules: Vec<RuleRun<'a>>,
    cursors: Cell<usize>,
    mapped_values: Cell<usize>,
}

impl<'a> ExpandedRules<'a> for Sheet<'a> {
    fn runs(&self) -> &[RuleRun<'a>] {
        &self.rules
    }

    fn value_links(&self) -> impl FnMut(&Property<'a>) -> Option<Span> {
        self.cursors.set(self.cursors.get() + 1);
        |property| {
            self.mapped_values.set(self.mapped_values.get() + 1);
            self.block
                .contains_block_span(property.value_span)
                .then_some(property.value_span)
        }
    }
}

#[test]
fn writes_plain_css_runs_and_maps_the_original_multiline_value() {
    let prefix = "😀\r\n";
    let source = "😀\r\nbutton::before { color:\r\nred; }\r\n.note { opacity: 1; }";
    let block = SourceRoot::new(source)
        .unwrap()
        .block(source.strip_prefix(prefix).unwrap(), prefix.len() as u32)
        .unwrap();
    let span = |word: &str| {
        let start = source.find(word).unwrap() as u32;
        Span::new(start, start + word.len() as u32)
    };
    let rule = |group, selector: &str, name: &'static str, value: &str| RuleRun {
        selector: selector.into(),
        selector_span: span(selector),
        group,
        declarations: vec![Property {
            name,
            name_span: span(name),
            value: value.into(),
            value_span: span(value),
        }],
    };
    let sheet = Sheet {
        block,
        rules: vec![
            rule(0, "button::before", "color", "red"),
            rule(1, ".note", "opacity", "1"),
        ],
        cursors: Cell::new(0),
        mapped_values: Cell::new(0),
    };
    let expected = "button::before {\n  color: red;\n}\n\n.note {\n  opacity: 1;\n}";
    assert_eq!(css_rules::emit(&sheet), expected);
    let mut plain = EmitDocument::new(false);
    css_rules::append(&mut plain, &sheet);
    assert_eq!(plain.as_str(), expected);
    assert!(plain.links().is_empty());
    assert_eq!((sheet.cursors.get(), sheet.mapped_values.get()), (0, 0));

    let mut mapped = EmitDocument::new(true);
    mapped.push_str("/* prefix */\n");
    css_rules::append(&mut mapped, &sheet);
    assert_eq!(
        mapped.as_str().strip_prefix("/* prefix */\n"),
        Some(expected)
    );
    assert_eq!((sheet.cursors.get(), sheet.mapped_values.get()), (1, 2));
    let links: Vec<_> = mapped
        .links()
        .iter()
        .map(|link| {
            (
                link.generated.slice(mapped.as_str()),
                link.authored,
                link.segment,
            )
        })
        .collect();
    assert_eq!(
        links,
        [
            ("button::before", span("button::before"), true),
            ("color", span("color"), true),
            ("red", span("red"), true),
            (".note", span(".note"), true),
            ("opacity", span("opacity"), true),
            ("1", span("1"), true),
        ]
    );
}

#[test]
fn empty_rules_keep_their_selectors_and_group_boundaries() {
    let block = SourceRoot::new(".a.b").unwrap().whole_block();
    let rules = [(0, ".a", Span::new(0, 2)), (2, ".b", Span::new(2, 4))].map(
        |(group, selector, selector_span)| RuleRun {
            group,
            selector: selector.into(),
            selector_span,
            declarations: Vec::new(),
        },
    );
    let sheet = Sheet {
        block,
        rules: Vec::from(rules),
        cursors: Cell::new(0),
        mapped_values: Cell::new(0),
    };
    let mut css = EmitDocument::new(true);
    css_rules::append(&mut css, &sheet);
    assert_eq!(css.as_str(), ".a {\n}\n\n.b {\n}");
    assert_eq!(css_rules::emit(&sheet), css.as_str());
    assert_eq!(
        css.links()
            .iter()
            .map(|link| link.authored)
            .collect::<Vec<_>>(),
        [Span::new(0, 2), Span::new(2, 4)]
    );
    assert_eq!((sheet.cursors.get(), sheet.mapped_values.get()), (1, 0));
}

#[test]
fn a_sheet_without_rules_keeps_the_existing_document() {
    let sheet = Sheet {
        block: SourceRoot::new("").unwrap().whole_block(),
        rules: Vec::new(),
        cursors: Cell::new(0),
        mapped_values: Cell::new(0),
    };
    let mut css = EmitDocument::new(true);
    css.push_linked("a", Span::new(0, 1));
    let before = css.clone();
    css_rules::append(&mut css, &sheet);
    assert_eq!(css, before);
    assert!(css_rules::emit(&sheet).is_empty());
    assert_eq!((sheet.cursors.get(), sheet.mapped_values.get()), (0, 0));
}
