//! Musea Art files retain their project-level subject types while the content
//! mapper reuses the same file-kind contract (#10186).

use super::super::{create_project_case, resolve_test_tsgo_binary, snapshot_project_diagnostics};

#[test]
fn content_mapper_musea_art_project_preserves_bound_component_diagnostics() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let project_root = create_project_case(
        "content-mapper-musea-art",
        &[
            (
                "src/MyCard.vue",
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../tests/_fixtures/differential/typechecker/content-mapper-musea-art-10186/src/MyCard.vue.txt"
                )),
            ),
            (
                "src/MyCard.art.vue",
                include_str!(concat!(
                    env!("CARGO_MANIFEST_DIR"),
                    "/../../tests/_fixtures/differential/typechecker/content-mapper-musea-art-10186/src/MyCard.art.vue.txt"
                )),
            ),
        ],
    );
    std::fs::write(
        project_root.join("tsconfig.json"),
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/typechecker/content-mapper-musea-art-10186/tsconfig.json.txt"
        )),
    )
    .unwrap();
    let snapshot = snapshot_project_diagnostics(&project_root);
    let _ = std::fs::remove_dir_all(&project_root);
    let Some(snapshot) = snapshot else {
        return;
    };

    assert_eq!(
        snapshot,
        vec![
            (
                "src/MyCard.art.vue".into(),
                Some(2322),
                "5:34:error Type 'number' is not assignable to type 'string'.".into(),
            ),
            (
                "src/MyCard.art.vue".into(),
                Some(2345),
                concat!(
                    "6:28:error Argument of type '{ elevated: true; }' is not assignable to parameter of type ",
                    "'__VizeComponentCheckProps<Props, __VizePublicComponentAttrs & { [x: string]: unknown; } & ",
                    "{ [x: `data${string}`]: unknown; } & Record<string, unknown>>'.\n",
                    "Property 'title' is missing in type '{ elevated: true; }' but required in type ",
                    "'{ readonly title: string; readonly elevated?: boolean | undefined; }'.",
                )
                .into(),
            ),
        ]
    );
}
