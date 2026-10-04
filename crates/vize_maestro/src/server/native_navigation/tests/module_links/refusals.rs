//! Typed whole errors discard all original and physical prefixes.
use super::{error, files, initialized, link, links, open_at, send};
use serde_json::json;
use tower_lsp::lsp_types::Url;

#[test]
fn module_link_rpc_original_program_file_and_profile_refusals_keep_complete_errors() {
    let (_dir, root, uri) = files();
    let mut service = initialized(Some(&Url::from_directory_path(&root).unwrap()));
    for (index, source, language, code, message) in [
        (
            2,
            "import './child.ts'; const value=/x/uv;",
            "javascript",
            -32002,
            "Native original Program refused",
        ),
        (
            3,
            "import './child.ts'; const value:number|string=1;value;",
            "typescript",
            -32003,
            "Native File observation refused",
        ),
        (
            4,
            "import './child.ts'; missing;",
            "javascript",
            -32003,
            "Native File observation refused",
        ),
        (
            7,
            "import './child.ts';",
            "html",
            -32001,
            "Native navigation language unsupported",
        ),
        (
            8,
            "plain source",
            "plaintext",
            -32001,
            "Native navigation language unsupported",
        ),
    ] {
        open_at(&mut service, &uri, source, language);
        assert_eq!(
            send(&mut service, links(index, &uri)),
            Some(error(index, code, message))
        );
    }
}

#[test]
fn module_link_rpc_actual_module_read_errors_discard_complete_prior_occurrences() {
    let (_dir, root, uri) = files();
    let mut service = initialized(Some(&Url::from_directory_path(&root).unwrap()));
    for (id, source) in [
        (2, "import './child.ts'; import('./child.ts');"),
        (
            3,
            "import './child.ts'; function f(){return import('./second.vue');}",
        ),
        (4, "import './child.ts'; export {} from './second.vue';"),
        (5, "import './child.ts'; import '\\uD800';"),
        (6, "/* original comment */"),
        (7, "\"use strict\";"),
    ] {
        open_at(&mut service, &uri, source, "javascript");
        assert_eq!(
            send(&mut service, links(id, &uri)),
            Some(error(id, -32014, "Native original module sources refused"))
        );
    }
}

#[test]
fn module_link_rpc_decoded_candidate_policy_refuses_without_alias_or_suffix_guessing() {
    let (_dir, root, uri) = files();
    let mut service = initialized(Some(&Url::from_directory_path(&root).unwrap()));
    for (id, request) in [
        (2, "../child.ts"),
        (3, "package"),
        (4, "@alias/child.ts"),
        (5, "./child"),
        (6, "./child.ts?raw"),
        (7, "./child.ts#fragment"),
        (8, "./%63hild.ts"),
        (9, "./child.TS"),
        (10, "././child.ts"),
        (11, "./child.ts/"),
        (12, "https://example.test/child.ts"),
        (13, "/absolute/child.ts"),
    ] {
        let source = vize_l0::cstr!("import './child.ts'; import '{request}';");
        open_at(&mut service, &uri, &source, "javascript");
        assert_eq!(
            send(&mut service, links(id, &uri)),
            Some(error(id, -32017, "Native module target policy refused"))
        );
    }
}

#[test]
fn module_link_rpc_nonregular_missing_and_symlink_targets_have_entire_failure_frames() {
    let (_dir, root, uri) = files();
    let mut service = initialized(Some(&Url::from_directory_path(&root).unwrap()));
    std::fs::create_dir(root.join("directory.ts")).unwrap();
    std::os::unix::fs::symlink(root.join("child.ts"), root.join("symbolic.ts")).unwrap();
    for (id, request, code, message) in [
        (
            2,
            "./missing.ts",
            -32018,
            "Native module target unavailable",
        ),
        (
            3,
            "./directory.ts",
            -32017,
            "Native module target policy refused",
        ),
        (
            4,
            "./symbolic.ts",
            -32018,
            "Native module target unavailable",
        ),
    ] {
        open_at(
            &mut service,
            &uri,
            &vize_l0::cstr!("import './child.ts'; import '{request}';"),
            "javascript",
        );
        assert_eq!(
            send(&mut service, links(id, &uri)),
            Some(error(id, code, message))
        );
    }
    let remote = Url::parse("https://example.test/original.ts").unwrap();
    open_at(&mut service, &remote, "import './child.ts';", "typescript");
    assert_eq!(
        send(&mut service, links(5, &remote)),
        Some(error(5, -32017, "Native module target policy refused"))
    );
}

#[test]
fn module_link_rpc_original_legacy_literals_refuse_with_complete_modern_control_frames() {
    let (_dir, root, uri) = files();
    let mut service = initialized(Some(&Url::from_directory_path(&root).unwrap()));
    for (index, source) in [
        (2, "import '\\056/child.ts';"),
        (3, "import './child.ts';const n=010;n;"),
        (4, "import './child.ts';const n=08;n;"),
        (5, "import './child.ts';const n=09.5;n;"),
        (6, "import './child.ts';const n='\\1';n;"),
        (7, "import './child.ts';const n='\\8';n;"),
        (8, "import './child.ts';const n='\\9';n;"),
        (9, "import './child.ts';const n='\\00';n;"),
    ] {
        open_at(&mut service, &uri, source, "javascript");
        assert_eq!(
            send(&mut service, links(index, &uri)),
            Some(error(index, -32002, "Native original Program refused"))
        );
    }
    for (index, source, start, end) in [
        (
            10,
            "/*\\056 010*/import '\\x2e/child.ts';const n=0o10;n;",
            19,
            34,
        ),
        (11, "import './child.ts';const n='\\\\1';n;", 7, 19),
        (12, "import '\\u002e/child.ts';const n=0x10;n;", 7, 24),
        (13, "import './child.ts';const n=0b10;n;", 7, 19),
    ] {
        open_at(&mut service, &uri, source, "javascript");
        assert_eq!(
            send(&mut service, links(index, &uri)),
            Some(json!({"jsonrpc":"2.0","id":index,"result":[
                link(0,start,end,Url::from_file_path(root.join("child.ts")).unwrap())
            ]}))
        );
    }
}
