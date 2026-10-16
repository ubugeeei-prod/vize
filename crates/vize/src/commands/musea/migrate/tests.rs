use std::path::{Path, PathBuf};

use super::{
    collect_story_files, component_tag_from_path, derive_component_path,
    is_default_ignored_story_path, is_story_file, output_path, story_basename,
};

#[test]
fn story_collection_is_available_without_formatter_features() {
    let directory = tempfile::tempdir().unwrap();
    let root = directory.path();
    let stories = [
        "src/Button.stories.tsx",
        "src/Card.stories.ts",
        "src/Link.stories.jsx",
        "src/Panel.stories.js",
    ];
    for relative in stories.into_iter().chain([
        "src/Component.vue",
        "src/plain.ts",
        "src/unsupported.stories.vue",
        "apps/web/node_modules/pkg/Hidden.stories.tsx",
        "dist/Hidden.stories.tsx",
        ".nuxt/Hidden.stories.tsx",
        ".output/Hidden.stories.tsx",
        "storybook-static/Hidden.stories.tsx",
        ".git/worktrees/cache/Hidden.stories.tsx",
    ]) {
        let path = root.join(relative);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, "export default {};").unwrap();
    }
    let patterns = [
        root.join("**/*").to_string_lossy().into_owned(),
        root.join("src/Button.stories.tsx")
            .to_string_lossy()
            .into_owned(),
    ];
    let actual = collect_story_files(&patterns);
    let expected = stories.map(|relative| root.join(relative));

    assert_eq!(actual, expected);
}

#[test]
fn is_story_file_matches_only_story_extensions() {
    assert!(is_story_file(Path::new("Button.stories.tsx")));
    assert!(is_story_file(Path::new("a/b/Button.stories.ts")));
    assert!(is_story_file(Path::new("Button.stories.jsx")));
    assert!(is_story_file(Path::new("Button.stories.js")));
    assert!(!is_story_file(Path::new("Button.tsx")));
    assert!(!is_story_file(Path::new("Button.stories.vue")));
    assert!(!is_story_file(Path::new("stories.ts")));
}

#[test]
fn default_ignored_story_paths_skip_dependencies_and_outputs() {
    assert!(is_default_ignored_story_path(Path::new(
        "node_modules/pkg/Button.stories.tsx"
    )));
    assert!(is_default_ignored_story_path(Path::new(
        ".output/public/Button.stories.tsx"
    )));
    assert!(!is_default_ignored_story_path(Path::new(
        "src/Button.stories.tsx"
    )));
}

#[test]
fn story_basename_strips_stories_suffix() {
    assert_eq!(
        story_basename(Path::new("a/AfButton.stories.tsx")).as_str(),
        "AfButton"
    );
    assert_eq!(
        story_basename(Path::new("Card.stories.ts")).as_str(),
        "Card"
    );
}

#[test]
fn output_path_alongside_source_by_default() {
    assert_eq!(
        output_path(Path::new("src/AfButton.stories.tsx"), None),
        PathBuf::from("src/AfButton.art.vue")
    );
}

#[test]
fn output_path_uses_out_dir_when_given() {
    assert_eq!(
        output_path(
            Path::new("src/AfButton.stories.tsx"),
            Some(Path::new("out/art"))
        ),
        PathBuf::from("out/art/AfButton.art.vue")
    );
}

#[test]
fn derive_component_path_falls_back_to_basename() {
    assert_eq!(
        derive_component_path(Path::new("src/AfButton.stories.tsx")).as_str(),
        "./AfButton.vue"
    );
}

#[test]
fn component_tag_from_path_uses_file_stem() {
    assert_eq!(
        component_tag_from_path("./AfButton.vue").as_str(),
        "AfButton"
    );
    assert_eq!(
        component_tag_from_path("../components/Card.vue").as_str(),
        "Card"
    );
    assert_eq!(
        component_tag_from_path("./Widget.vue?raw").as_str(),
        "Widget"
    );
}
