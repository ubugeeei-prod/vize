//! Reused build dumps preserve user files and fail closed on uncertain capture.
#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use davinci_test_support::schema;
use serde_json::Value;

fn project(name: &str, source: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "vize-build-dump-safety-{}-{name}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/a.vue"), source).unwrap();
    root
}

fn build(root: &Path, extra: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(["build", "src", "--output", "dist", "--dump-dir", "dumps"])
        .args(extra)
        .output()
        .unwrap()
}

fn dump_dir(root: &Path) -> PathBuf {
    root.join("dumps/a.vue")
}

fn feed(root: &Path) -> Value {
    serde_json::from_slice(&fs::read(dump_dir(root).join("stages.json")).unwrap()).unwrap()
}

fn first_page(root: &Path) -> PathBuf {
    fs::read_dir(dump_dir(root))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .find(|path| path.extension().is_some_and(|ext| ext == "dump"))
        .unwrap()
}

#[test]
fn reused_build_dump_clears_only_feed_owned_pages() {
    let root = project("reuse", "<template><div>hello</div></template>");
    assert!(build(&root, &[]).status.success());
    let user_page = dump_dir(&root).join("999-user.dump");
    fs::write(&user_page, "user data").unwrap();
    assert!(build(&root, &[]).status.success());
    assert_eq!(fs::read_to_string(user_page).unwrap(), "user data");
    assert_eq!(feed(&root)["outcome"]["kind"], "accepted");
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn modified_owned_page_refuses_cleanup_without_deleting_user_data() {
    let root = project("modified", "<template><div>hello</div></template>");
    assert!(build(&root, &[]).status.success());
    let page = first_page(&root);
    fs::write(&page, "user modification").unwrap();
    let old_feed = fs::read(dump_dir(&root).join("stages.json")).unwrap();
    let output = build(&root, &["--continue-on-error"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(fs::read_to_string(page).unwrap(), "user modification");
    assert_eq!(
        fs::read(dump_dir(&root).join("stages.json")).unwrap(),
        old_feed
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn unowned_page_name_collision_refuses_overwrite() {
    let root = project("collision", "<template><div>hello</div></template>");
    assert!(build(&root, &[]).status.success());
    let page = first_page(&root);
    let name = page.file_name().unwrap().to_owned();
    fs::remove_file(dump_dir(&root).join("stages.json")).unwrap();
    fs::write(&page, "user data").unwrap();
    let output = build(&root, &[]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        fs::read_to_string(dump_dir(&root).join(name)).unwrap(),
        "user data"
    );
    fs::remove_dir_all(root).unwrap();
}

#[cfg(unix)]
#[test]
fn symlinked_source_dump_directory_cannot_redirect_cleanup() {
    use std::os::unix::fs::symlink;

    let root = project("symlink", "<template><div>hello</div></template>");
    let outside = root.join("outside");
    fs::create_dir_all(&outside).unwrap();
    fs::write(outside.join("sentinel"), "safe").unwrap();
    fs::create_dir_all(root.join("dumps")).unwrap();
    symlink(&outside, dump_dir(&root)).unwrap();
    let output = build(&root, &["--continue-on-error"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(
        fs::read_to_string(outside.join("sentinel")).unwrap(),
        "safe"
    );
    assert!(!outside.join("stages.json").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn failed_rebuild_does_not_keep_an_old_accepted_feed() {
    let root = project("failed", "<template><div>hello</div></template>");
    assert!(build(&root, &[]).status.success());
    assert_eq!(feed(&root)["outcome"]["kind"], "accepted");
    fs::remove_file(root.join("dist/a.js")).unwrap();
    fs::write(
        root.join("src/a.vue"),
        "<template lang=\"pug\">include missing</template>",
    )
    .unwrap();
    let output = build(&root, &["--continue-on-error"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(!dump_dir(&root).join("stages.json").exists());
    assert!(fs::read_dir(dump_dir(&root)).unwrap().next().is_none());
    assert!(root.join("dist/a.js").exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn syntax_labels_match_the_vue_template_parser_route() {
    let root = project("syntax", "<template lang=\"PUG\">p Hello</template>");
    assert!(build(&root, &[]).status.success());
    let pug = feed(&root);
    assert_eq!(pug["source"]["authored_syntax"], "pug");
    assert_eq!(pug["source"]["compiled_syntax"], "vue-template");
    let schema_value: Value = serde_json::from_str(include_str!(
        "../../../docs/davinci/plan/product-stage-feed.schema.json"
    ))
    .unwrap();
    assert_eq!(schema::validate(&schema_value, &pug, "$"), Ok(()));
    fs::write(
        root.join("src/a.vue"),
        "<template lang=\"jade\">p Hello</template>",
    )
    .unwrap();
    assert!(build(&root, &[]).status.success());
    let jade = feed(&root);
    assert_eq!(jade["source"]["authored_syntax"], "jade");
    assert_eq!(jade["source"]["compiled_syntax"], "vue-template");
    assert_eq!(schema::validate(&schema_value, &jade, "$"), Ok(()));
    assert!(
        fs::read_to_string(root.join("dist/a.js"))
            .unwrap()
            .contains("p Hello")
    );
    fs::write(
        root.join("src/a.vue"),
        "<template><div>Hello</div></template>",
    )
    .unwrap();
    assert!(build(&root, &[]).status.success());
    let standard = feed(&root);
    assert_eq!(standard["source"]["authored_syntax"], "vue-template");
    assert_eq!(standard["source"]["compiled_syntax"], "vue-template");
    assert_eq!(schema::validate(&schema_value, &standard, "$"), Ok(()));
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn external_template_source_cannot_claim_accepted_native_pages() {
    let root = project("external", "<template src=\"./other.html\" />");
    let output = build(&root, &["--continue-on-error"]);
    if output.status.success() {
        let value = feed(&root);
        assert_eq!(value["outcome"]["kind"], "unavailable");
        assert!(value["pages"].as_array().unwrap().is_empty());
    } else {
        assert!(!dump_dir(&root).join("stages.json").exists());
    }
    fs::remove_dir_all(root).unwrap();
}
