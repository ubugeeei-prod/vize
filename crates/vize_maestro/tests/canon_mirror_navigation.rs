//! #12533: a CSS `v-bind()` query can move the editor's Canon project onto
//! its materialized mirror. Types that need the Canon root's shared helpers
//! or its packages must stay exact afterwards.
#![cfg(feature = "native")]
#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "tests assert by panicking"
)]
#![expect(
    clippy::disallowed_types,
    reason = "the Corsa bridge is shared as `Arc<CorsaBridge>`; hovers compare std strings"
)]

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use tower_lsp::lsp_types::{GotoDefinitionResponse, HoverContents, Url};
use vize_canon::{CorsaBridge, CorsaBridgeConfig};
use vize_maestro::ide::{HoverService, IdeContext, ImplementationService, TypeDefinitionService};
use vize_maestro::server::ServerState;

const TSCONFIG: &str = include_str!(
    "../../../tests/_fixtures/differential/typechecker/materialized-canon-root-12533/tsconfig.json.txt"
);
const SHAPES: &str = include_str!(
    "../../../tests/_fixtures/differential/typechecker/materialized-canon-root-12533/shapes.ts.txt"
);
const CHILD: &str = include_str!(
    "../../../tests/_fixtures/differential/typechecker/materialized-canon-root-12533/Child.vue.txt"
);
const CARD: &str = include_str!(
    "../../../tests/_fixtures/differential/typechecker/materialized-canon-root-12533/Card.vue.txt"
);
const PANEL: &str = include_str!(
    "../../../tests/_fixtures/differential/typechecker/materialized-canon-root-12533/Panel.vue.txt"
);

/// The installed tsgo, following the crate's unit-test selection: skip when
/// disabled or locally absent, fail when CI requires it.
fn tsgo() -> Option<PathBuf> {
    let set = |name: &str| {
        std::env::var_os(name).is_some_and(|value| !value.is_empty() && value != "false")
    };
    let required = std::env::var_os("VIZE_TEST_REQUIRE_TSGO").is_some_and(|value| value == "1");
    if set("VIZE_TEST_DISABLE_TSGO") {
        assert!(
            !required,
            "VIZE_TEST_REQUIRE_TSGO=1 conflicts with VIZE_TEST_DISABLE_TSGO"
        );
        return None;
    }
    if let Some(path) = std::env::var_os("VIZE_TEST_TSGO_PATH") {
        return Some(path.into());
    }
    let workspace = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let resolved = vize_carton::corsa_resolver::resolve_corsa_executable(
        vize_carton::corsa_resolver::CorsaResolveRequest {
            project_root: Some(&workspace),
            ..Default::default()
        },
    );
    match resolved {
        Ok(path) => Some(path),
        Err(error) => {
            assert!(
                !required && !set("CI"),
                "tsgo-backed test has no tsgo binary: {error:?}"
            );
            eprintln!("skipping tsgo-backed test: no tsgo binary ({error:?})");
            None
        }
    }
}

struct Project {
    src: PathBuf,
    card: Url,
    panel: Url,
    state: ServerState,
    bridge: Arc<CorsaBridge>,
    _root: tempfile::TempDir,
}

impl Project {
    async fn open(tsgo: PathBuf) -> Self {
        // The server compares physical paths; macOS's `/var` is a symlink.
        let temp = std::env::temp_dir();
        let root = tempfile::tempdir_in(temp.canonicalize().unwrap_or(temp)).expect("project");
        let src = root.path().join("src");
        fs::create_dir_all(&src).expect("src");
        fs::write(root.path().join("tsconfig.json"), TSCONFIG).expect("tsconfig");
        fs::write(src.join("shapes.ts"), SHAPES).expect("shapes");
        fs::write(src.join("Child.vue"), CHILD).expect("child");
        let state = ServerState::new();
        state.set_workspace_root(root.path().to_path_buf());
        let open = |name: &str, text: &str| {
            fs::write(src.join(name), text).expect("sfc");
            let uri = Url::from_file_path(src.join(name)).expect("sfc uri");
            state
                .documents
                .open(uri.clone(), text.to_owned(), 1, "vue".to_owned());
            state.update_virtual_docs(&uri, text);
            uri
        };
        let card = open("Card.vue", CARD);
        let panel = open("Panel.vue", PANEL);
        let bridge = Arc::new(CorsaBridge::with_config(CorsaBridgeConfig {
            corsa_path: Some(tsgo),
            working_dir: Some(root.path().to_path_buf()),
            timeout_ms: 30_000,
            ..Default::default()
        }));
        bridge.spawn().await.expect("tsgo session");
        Self {
            src,
            card,
            panel,
            state,
            bridge,
            _root: root,
        }
    }

    fn context<'a>(&'a self, uri: &'a Url, needle: &str, nth: usize) -> IdeContext<'a> {
        let text = if *uri == self.card { CARD } else { PANEL };
        let offset = text.find(needle).expect("needle") + nth;
        IdeContext::new(&self.state, uri, offset).expect("context")
    }

    /// The ref binding in a template, a component instance and one of its
    /// exposed members.
    async fn hovers(&self) -> Vec<String> {
        let mut hovers = Vec::new();
        for (uri, needle, nth) in [
            (&self.card, "count }}", 2),
            (&self.panel, "child?.focus", 2),
            (&self.panel, "focus()", 1),
        ] {
            let ctx = self.context(uri, needle, nth);
            match HoverService::hover_with_corsa(&ctx, Some(self.bridge.clone()))
                .await
                .map(|hover| hover.contents)
            {
                Some(HoverContents::Markup(markup)) => hovers.push(markup.value),
                other => panic!("expected a markup hover at {needle}: {other:?}"),
            }
        }
        hovers
    }

    /// Target file names, relative to the authored `src`.
    fn names(&self, response: Option<GotoDefinitionResponse>) -> Vec<String> {
        let uris = match response {
            None => Vec::new(),
            Some(GotoDefinitionResponse::Scalar(location)) => vec![location.uri],
            Some(GotoDefinitionResponse::Array(locations)) => {
                locations.into_iter().map(|location| location.uri).collect()
            }
            Some(GotoDefinitionResponse::Link(links)) => {
                links.into_iter().map(|link| link.target_uri).collect()
            }
        };
        uris.iter()
            .map(|uri| {
                let path = uri.to_file_path().expect("file target");
                path.strip_prefix(&self.src)
                    .unwrap_or_else(|_| panic!("target outside the project: {uri}"))
                    .to_string_lossy()
                    .into_owned()
            })
            .collect()
    }
}

#[test]
fn style_binding_navigation_keeps_canon_project_types() {
    vize_maestro::runtime::block_on(async {
        let Some(tsgo) = tsgo() else {
            return;
        };
        let project = Project::open(tsgo).await;
        let expected = [
            "```typescript\nconst count: number\n```",
            "```typescript\nconst child: $ComponentInstance | null\n```",
            "```typescript\n(property) focus: () => void | undefined\n```",
        ];
        assert_eq!(project.hovers().await, expected, "before navigation");

        let bridge = Some(project.bridge.clone());
        let ctx = project.context(&project.card, "measure(shape, 3)", 2);
        let implementation =
            ImplementationService::implementation_with_corsa(&ctx, bridge.clone()).await;
        assert_eq!(project.names(implementation), ["Card.vue"]);
        let ctx = project.context(&project.card, "shape, 2", 1);
        let type_definition = TypeDefinitionService::type_definition_with_corsa(&ctx, bridge).await;
        assert_eq!(project.names(type_definition), ["shapes.ts"]);

        assert_eq!(project.hovers().await, expected, "after navigation");
        project.bridge.shutdown().await.expect("shutdown");
    });
}
