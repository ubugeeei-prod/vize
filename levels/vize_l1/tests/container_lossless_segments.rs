use vize_l0::Allocator;
use vize_l1::container::{ContainerFormat, ContainerSegment, Vue};

const SOURCE: &str = include_str!("fixtures/container/lossless-blocks.vue");

#[test]
fn ordered_segments_reconstruct_the_complete_authored_sfc() {
    let allocator = Allocator::default();
    let container = Vue.split(&allocator, SOURCE);
    assert!(container.errors.is_empty(), "{:?}", container.errors);
    assert_eq!(container.blocks.len(), 4);

    let segments = container.segments().collect::<Vec<_>>();
    assert_eq!(segments.len(), 9);
    assert!(matches!(
        segments.first(),
        Some(ContainerSegment::Interstitial(_))
    ));
    assert!(matches!(
        segments.last(),
        Some(ContainerSegment::Interstitial(_))
    ));

    let reconstructed = segments
        .iter()
        .map(|segment| segment.span().slice(container.source))
        .collect::<String>();
    assert_eq!(reconstructed.as_bytes(), SOURCE.as_bytes());
}

#[test]
fn block_segments_retain_original_kinds_attributes_and_bytes() {
    let allocator = Allocator::default();
    let container = Vue.split(&allocator, SOURCE);
    assert!(container.errors.is_empty(), "{:?}", container.errors);

    let blocks = container
        .segments()
        .filter_map(|segment| match segment {
            ContainerSegment::Block(block) => Some(block),
            ContainerSegment::Interstitial(_) => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        blocks.iter().map(|block| block.name).collect::<Vec<_>>(),
        ["template", "i18n", "script", "style"]
    );

    assert_eq!(
        blocks[0].attr("lang").and_then(|attr| attr.value),
        Some("pug")
    );
    assert_eq!(
        blocks[1].attr("locale").and_then(|attr| attr.value),
        Some("ja")
    );
    assert!(blocks[2].attr("setup").is_some());
    assert_eq!(blocks[2].attr("setup").and_then(|attr| attr.value), None);
    assert_eq!(
        blocks[2].attr("lang").and_then(|attr| attr.value),
        Some("ts")
    );
    assert!(blocks[3].attr("scoped").is_some());
    assert!(blocks[3].attr("module").is_some());

    assert_eq!(
        blocks[1].authored_span().slice(SOURCE),
        "<i18n locale='ja'>\n{\"hello\":\"こんにちは\"}\n</i18n>"
    );
    assert_eq!(blocks[2].content.slice(SOURCE), "\nconst answer = 42\n");
    assert_eq!(
        blocks[3].close_tag.expect("closed style").slice(SOURCE),
        "</style>"
    );
}
