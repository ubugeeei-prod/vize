use std::path::Path;

use super::generate_vue_content_mapper_transform;

const DEFINE_ART_STUB: &str =
    "declare function defineArt(source: string, options?: Record<string, any>): void;";
const SUBJECT_STUB: &str = r#"declare const MyCard: typeof import("./MyCard.vue").default;"#;

#[test]
fn musea_art_file_declares_define_art_and_types_its_implicit_component() {
    let source = r#"<script setup lang="ts">
defineArt("./MyCard.vue", { title: "MyCard" });
</script>
<art>
  <variant name="Default"><MyCard title="hello" /></variant>
</art>
"#;
    let art = generate_vue_content_mapper_transform(Path::new("MyCard.art.vue"), source)
        .expect("art transform");
    let ordinary = generate_vue_content_mapper_transform(Path::new("MyCard.vue"), source)
        .expect("ordinary SFC transform");

    assert_eq!(
        art.text
            .lines()
            .filter(|line| *line == DEFINE_ART_STUB)
            .count(),
        1,
        "the mapper must provide exactly one Musea file-kind global:\n{}",
        art.text,
    );
    assert_eq!(
        art.text
            .lines()
            .filter(|line| *line == SUBJECT_STUB)
            .count(),
        1,
        "the art subject must retain the component's real prop type:\n{}",
        art.text,
    );
    assert_eq!(
        ordinary
            .text
            .lines()
            .filter(|line| *line == DEFINE_ART_STUB)
            .count(),
        0,
        "ordinary Vue files must not gain Musea globals:\n{}",
        ordinary.text,
    );
}
