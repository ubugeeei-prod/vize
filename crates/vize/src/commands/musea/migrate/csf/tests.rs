use oxc_allocator::Allocator;
use oxc_parser::Parser;
use oxc_span::SourceType;

use super::extract_csf;

/// One story reduced to the fields the unit tests assert on.
#[derive(Debug, PartialEq, Eq)]
struct TestStory<'a> {
    name: &'a str,
    has_render: bool,
    has_args: bool,
}

#[test]
fn extracts_satisfies_meta_and_stories() {
    let source = r#"import AfButton from "./AfButton.vue";
export default { component: AfButton, title: "Base/AfButton" } satisfies Meta<typeof AfButton>;
export const Primary = { render: () => <AfButton color="primary">Primary</AfButton> };
export const Secondary: StoryObj = { args: { color: "secondary", label: "Hi" } };
"#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::tsx()).parse();
    assert!(!parsed.panicked);
    let module = extract_csf(&parsed.program);

    assert_eq!(module.title.as_deref(), Some("Base/AfButton"));
    assert_eq!(module.component_path.as_deref(), Some("./AfButton.vue"));
    let stories: Vec<TestStory> = module
        .stories
        .iter()
        .map(|story| TestStory {
            name: story.name.as_str(),
            has_render: story.render.is_some(),
            has_args: story.args.is_some(),
        })
        .collect();
    assert_eq!(
        stories,
        vec![
            TestStory {
                name: "Primary",
                has_render: true,
                has_args: false,
            },
            TestStory {
                name: "Secondary",
                has_render: false,
                has_args: true,
            },
        ]
    );
}

#[test]
fn extracts_const_meta_default_export() {
    let source = r#"import Card from "../components/Card.vue";
const meta = { component: Card, title: "Card" } satisfies Meta<typeof Card>;
export default meta;
export const Only = { args: {} };
"#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::tsx()).parse();
    assert!(!parsed.panicked);
    let module = extract_csf(&parsed.program);

    assert_eq!(module.title.as_deref(), Some("Card"));
    assert_eq!(
        module.component_path.as_deref(),
        Some("../components/Card.vue")
    );
    let stories: Vec<TestStory> = module
        .stories
        .iter()
        .map(|story| TestStory {
            name: story.name.as_str(),
            has_render: story.render.is_some(),
            has_args: story.args.is_some(),
        })
        .collect();
    assert_eq!(
        stories,
        vec![TestStory {
            name: "Only",
            has_render: false,
            has_args: true,
        }]
    );
}

#[test]
fn extracts_as_meta_and_name_override() {
    let source = r#"import Box from "./Box.vue";
export default { component: Box, title: "Box" } as Meta;
export const First = { name: "Custom Name", render: () => <Box /> };
"#;
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, source, SourceType::tsx()).parse();
    assert!(!parsed.panicked);
    let module = extract_csf(&parsed.program);

    assert_eq!(module.title.as_deref(), Some("Box"));
    assert_eq!(module.component_path.as_deref(), Some("./Box.vue"));
    let stories: Vec<TestStory> = module
        .stories
        .iter()
        .map(|story| TestStory {
            name: story.name.as_str(),
            has_render: story.render.is_some(),
            has_args: story.args.is_some(),
        })
        .collect();
    assert_eq!(
        stories,
        vec![TestStory {
            name: "Custom Name",
            has_render: true,
            has_args: false,
        }]
    );
}
