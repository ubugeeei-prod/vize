use std::path::{Path, PathBuf};

use super::{
    component_tag_from_path, derive_component_path, is_story_file, output_path, story_basename,
};

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
