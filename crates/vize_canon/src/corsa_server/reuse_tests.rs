//! P5-8: a second check reuses the project session; the manifest still separates them.

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

use super::{CorsaServer, ServerConfig};

const SOURCE: &str = "<script setup lang=\"ts\">\nconst count: string = 0;\n</script>\n";
const TSCONFIG: &str = r#"{
  "compilerOptions": {
    "strict": true,
    "target": "ES2022",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "noEmit": true
  },
  "include": ["src/**/*"]
}
"#;

#[test]
fn session_stats_start_at_zero() {
    let mut server = CorsaServer::new();
    let response = server.handle_request(r#"{"jsonrpc":"2.0","id":1,"method":"session-stats"}"#);
    assert!(response.error.is_none());
    assert_eq!(
        response.result.unwrap(),
        json!({"projectInits": 0, "liveSessions": 0})
    );
}

#[test]
fn the_second_check_does_not_initialize_and_a_new_tsconfig_does() {
    let Some((mut server, root)) = server(Duration::from_secs(60)) else {
        return;
    };
    let host = root.path().join("src/App.vue");

    let first = check(&mut server, &host, "");
    assert_eq!(first["diagnostics"], expected_diagnostic());
    let inits = server.project_inits();
    assert!(inits >= 1, "the first check initializes, got {inits}");
    assert_eq!(server.live_sessions(), 1);

    let second = check(&mut server, &host, "");
    assert_eq!(second["diagnostics"], first["diagnostics"]);
    assert_eq!(
        server.project_inits(),
        inits,
        "the second check must not initialize"
    );
    assert_eq!(server.live_sessions(), 1);

    std::fs::write(
        root.path().join("tsconfig.json"),
        TSCONFIG.replace("true", "false"),
    )
    .unwrap();
    let edited = check(&mut server, &host, "");
    assert_eq!(edited["errorCount"], 1);
    assert!(
        server.project_inits() > inits,
        "tsconfig content is part of the key"
    );
    assert_eq!(server.live_sessions(), 2);
    let edited_inits = server.project_inits();

    let reused = check(&mut server, &host, "");
    assert_eq!(reused["diagnostics"], edited["diagnostics"]);
    assert_eq!(server.project_inits(), edited_inits);
    assert_eq!(server.live_sessions(), 2);
}

#[test]
fn feature_flags_do_not_share_a_session_and_each_flag_set_is_reused() {
    let Some((mut server, root)) = server(Duration::from_secs(60)) else {
        return;
    };
    let host = root.path().join("src/App.vue");

    let _ = check(&mut server, &host, "");
    let first = server.project_inits();
    assert!(first >= 1);
    assert_eq!(server.live_sessions(), 1);

    let _ = check(&mut server, &host, "options-api=0");
    assert!(server.project_inits() > first);
    assert_eq!(server.live_sessions(), 2);
    let both = server.project_inits();

    let _ = check(&mut server, &host, "");
    let _ = check(&mut server, &host, "options-api=0");
    assert_eq!(server.project_inits(), both);
    assert_eq!(server.live_sessions(), 2);
}

#[test]
fn an_idle_session_is_torn_down_and_the_next_check_initializes() {
    let Some((mut server, root)) = server(Duration::from_millis(1)) else {
        return;
    };
    let host = root.path().join("src/App.vue");
    let _ = check(&mut server, &host, "");
    let inits = server.project_inits();
    assert!(inits >= 1);
    assert_eq!(server.live_sessions(), 1);

    server.reap_idle_at(Instant::now() + Duration::from_secs(1));
    assert_eq!(server.live_sessions(), 0);

    let _ = check(&mut server, &host, "");
    assert!(server.project_inits() > inits);
    assert_eq!(server.live_sessions(), 1);
}

fn server(idle: Duration) -> Option<(CorsaServer, tempfile::TempDir)> {
    let corsa = corsa_path()?;
    let root = tempfile::tempdir().unwrap();
    write_project(root.path());
    let server = CorsaServer::with_config(ServerConfig {
        corsa_path: Some(corsa.to_string_lossy().into_owned().into()),
        working_dir: Some(root.path().to_string_lossy().into_owned().into()),
        idle_timeout: idle,
    });
    Some((server, root))
}

fn check(server: &mut CorsaServer, host: &Path, flags: &str) -> Value {
    let uri = vize_carton::cstr!("{}", host.display());
    let request = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": "check",
        "params": {"uri": uri.as_str(), "content": SOURCE, "flags": flags}
    });
    let response = server.handle_request(&serde_json::to_string(&request).unwrap());
    assert!(
        response.error.is_none(),
        "check failed: {:?}",
        response.error.as_ref().map(|error| error.message.as_str())
    );
    response.result.unwrap()
}

fn expected_diagnostic() -> Value {
    json!([{
        "message": "Type 'number' is not assignable to type 'string'.",
        "severity": "error",
        "line": 2,
        "column": 7,
        "code": "TS2322"
    }])
}

fn corsa_path() -> Option<PathBuf> {
    if std::env::var_os("VIZE_TEST_DISABLE_TSGO").is_some() {
        return None;
    }
    let explicit = std::env::var_os("CORSA_PATH").map(PathBuf::from);
    let request = vize_carton::corsa_resolver::CorsaResolveRequest {
        explicit_path: explicit.as_deref(),
        project_root: Some(Path::new(env!("CARGO_MANIFEST_DIR"))),
    };
    vize_carton::corsa_resolver::resolve_corsa_executable(request).ok()
}

fn write_project(root: &Path) {
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(root.join("tsconfig.json"), TSCONFIG).unwrap();
    std::fs::write(root.join("src/App.vue"), SOURCE).unwrap();
    let node_modules = root.join("node_modules");
    crate::batch::write_vue_facade(&node_modules).unwrap();
    let runtime_dom = node_modules.join("@vue/runtime-dom");
    std::fs::create_dir_all(&runtime_dom).unwrap();
    std::fs::write(
        runtime_dom.join("package.json"),
        r#"{"name":"@vue/runtime-dom","types":"index.d.ts"}"#,
    )
    .unwrap();
    std::fs::write(
        runtime_dom.join("index.d.ts"),
        crate::batch::VUE_RUNTIME_DOM_STUB_TYPES,
    )
    .unwrap();
}
