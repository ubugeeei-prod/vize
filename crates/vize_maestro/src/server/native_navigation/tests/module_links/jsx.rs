//! Real JSX/TSX elements preserve original operands through actual target IO.
use super::{error, files, initialized, link, links, open_at, send};
use serde_json::json;
use tower_lsp::lsp_types::Url;

#[test]
fn module_link_rpc_original_profile_inputs_keep_exact_positive_frames() {
    let (_dir, root, uri) = files();
    let mut service = initialized(Some(&Url::from_directory_path(&root).unwrap()));
    for (id, source, language) in [
        (
            5,
            "import './child.ts'; const value=1;value;",
            "javascriptreact",
        ),
        (
            6,
            "import './child.ts'; const value:number=1;value;",
            "typescriptreact",
        ),
    ] {
        open_at(&mut service, &uri, source, language);
        assert_eq!(
            send(&mut service, links(id, &uri)),
            Some(json!({"jsonrpc":"2.0","id":id,"result":[
                link(0, 7, 19, Url::from_file_path(root.join("child.ts")).unwrap())
            ]}))
        );
    }
}

#[test]
fn module_link_rpc_original_jsx_and_tsx_keep_complete_frames_and_standard_route() {
    let source = "/*😀*/import Widget from './child.ts';\r\nconst local=1;\r\nconst tree=<Widget value={local}/>;\r\nexport {alpha,beta} from './second.vue';";
    for language in ["javascriptreact", "typescriptreact"] {
        let (_dir, root, uri) = files();
        let mut service = initialized(Some(&Url::from_directory_path(&root).unwrap()));
        open_at(&mut service, &uri, source, language);
        let expected = json!([
            link(
                0,
                25,
                37,
                Url::from_file_path(root.join("child.ts")).unwrap()
            ),
            link(
                3,
                25,
                39,
                Url::from_file_path(root.join("second.vue")).unwrap()
            )
        ]);
        for id in [2, 3] {
            assert_eq!(
                send(&mut service, links(id, &uri)),
                Some(json!({"jsonrpc":"2.0","id":id,"result":expected}))
            );
        }
        assert_eq!(
            send(
                &mut service,
                json!({"jsonrpc":"2.0","id":4,"method":"textDocument/documentLink",
                    "params":{"textDocument":{"uri":uri}}})
            ),
            Some(json!({"jsonrpc":"2.0","id":4,"result":null}))
        );
        for (id, suffix, code, message) in [
            (
                5,
                "import '\\uD800';",
                -32014,
                "Native original module sources refused",
            ),
            (
                6,
                "import './missing.ts';",
                -32018,
                "Native module target unavailable",
            ),
        ] {
            let complete = vize_l0::cstr!("{source}\r\n{suffix}");
            open_at(&mut service, &uri, complete.as_str(), language);
            assert_eq!(
                send(&mut service, links(id, &uri)),
                Some(error(id, code, message))
            );
        }
    }
}
