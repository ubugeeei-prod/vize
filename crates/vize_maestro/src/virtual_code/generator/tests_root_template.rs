use super::{
    ArtCursorPosition, BlockType, find_art_block_at_completion_offset, find_art_block_at_offset,
};

#[test]
fn art_root_template_keeps_ordinary_body_boundaries_and_variant_classification() {
    let original = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lsp/native-document-identity-11586/Parent.art.vue.txt"
    ));
    for suffix in [
        "",
        "\n<art title=\"Examples\"><variant name=\"First\"><p>example</p></variant></art>",
    ] {
        let source = [original, suffix].concat();
        let start = source.find("{{ format").unwrap();
        let end = source.find("</template>").unwrap();
        for offset in start..end {
            assert_eq!(
                find_art_block_at_offset(&source, offset),
                Some(BlockType::Template)
            );
            assert_eq!(
                find_art_block_at_completion_offset(&source, offset),
                Some(BlockType::Template)
            );
        }
        for offset in [start - 1, end, end + 1] {
            assert_ne!(
                find_art_block_at_offset(&source, offset),
                Some(BlockType::Template)
            );
            assert_ne!(
                find_art_block_at_completion_offset(&source, offset),
                Some(BlockType::Template)
            );
        }
        if !suffix.is_empty() {
            let offset = source.find("<p>example").unwrap();
            assert!(matches!(
                find_art_block_at_offset(&source, offset),
                Some(BlockType::Art(ArtCursorPosition::VariantTemplate(_)))
            ));
        }
    }
}

#[cfg(feature = "native")]
#[test]
fn art_root_template_import_definition_works_without_a_checker() {
    use crate::ide::{DefinitionService, IdeContext};
    use crate::server::ServerState;
    use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};

    let directory = tempfile::tempdir().unwrap();
    let source = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/lsp/native-document-identity-11586/Parent.art.vue.txt"
    ));
    let target = directory.path().join("format.ts");
    std::fs::write(
        &target,
        "export function format(value: string) { return value }\n",
    )
    .unwrap();
    let uri = Url::from_file_path(directory.path().join("Parent.art.vue")).unwrap();
    let state = ServerState::new();
    state.set_workspace_root(directory.path().to_path_buf());
    state
        .documents
        .open(uri.clone(), source.into(), 1, "art-vue".into());
    let ctx = IdeContext::new(&state, &uri, source.rfind("format('hello')").unwrap() + 1).unwrap();
    let actual = crate::runtime::block_on(DefinitionService::definition_with_corsa(&ctx, None));
    assert_eq!(
        actual,
        Some(GotoDefinitionResponse::Scalar(Location {
            uri: Url::from_file_path(&target).unwrap(),
            range: Range::new(Position::new(0, 16), Position::new(0, 22)),
        }))
    );
}
