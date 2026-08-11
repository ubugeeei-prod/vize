//! Private package imports must advance to a graph fixpoint, not a depth cap.
#![allow(
    clippy::disallowed_macros,
    clippy::disallowed_methods,
    clippy::disallowed_types
)]

use std::path::Path;

use super::vue_document::{CorsaVueVirtualDocumentOptions, build_vue_virtual_project};

#[test]
fn editor_discovers_private_package_chain_beyond_the_old_eight_pass_cap() {
    let root = tempfile::tempdir().unwrap();
    let app = root.path().join("app");
    let host = app.join("src/Host.vue");
    write(
        &app.join("tsconfig.json"),
        r#"{"compilerOptions":{"module":"ESNext","moduleResolution":"Bundler","allowArbitraryExtensions":true}}"#,
    );
    write(
        &host,
        "<script setup lang=\"ts\">\nimport Root from '@scope/root'\nvoid Root\n</script>\n",
    );
    install_package(&app, "root", Some("level-1"));
    for level in 1..=10 {
        let next = (level < 10).then(|| format!("level-{}", level + 1));
        install_package(&app, &format!("level-{level}"), next.as_deref());
    }
    let source = std::fs::read_to_string(&host).unwrap();
    let project =
        build_vue_virtual_project(&host, &source, CorsaVueVirtualDocumentOptions::default())
            .unwrap();
    let host = crate::file_uri::file_uri_to_path(&project.host.request_uri).unwrap();
    let mut nested = host.parent().unwrap().join("node_modules/@scope/root");
    for level in 1..=10 {
        nested = nested
            .join("node_modules/@scope")
            .join(format!("level-{level}"));
    }
    assert!(
        nested.join("src/Entry.d.vue.ts").is_file(),
        "the final private package was not materialized: {}",
        nested.display()
    );
}

fn install_package(app: &Path, name: &str, next: Option<&str>) {
    let package = app.join("node_modules/@scope").join(name);
    let imports = next.map_or_else(
        || String::from("{}"),
        |next| format!(r##"{{"#next":"@scope/{next}"}}"##),
    );
    write(
        &package.join("package.json"),
        &format!(
            "{{\"name\":\"@scope/{name}\",\"exports\":\"./src/Entry.vue\",\"imports\":{imports}}}\n"
        ),
    );
    let import = next.map_or_else(String::new, |_| {
        String::from("import Next from '#next'\nvoid Next\n")
    });
    let prop = name.replace('-', "_");
    write(
        &package.join("src/Entry.vue"),
        &format!(
            "<script setup lang=\"ts\">\n{import}defineProps<{{ {prop}: string }}>()\n</script>\n"
        ),
    );
}

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}
