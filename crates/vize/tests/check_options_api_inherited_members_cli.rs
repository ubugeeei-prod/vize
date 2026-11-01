//! Original imported Options API template members through the default-on CLI.
#![expect(
    clippy::disallowed_macros,
    reason = "complete fixture process observations"
)]
#![expect(clippy::disallowed_methods, reason = "fixture uses std strings")]
#![expect(clippy::disallowed_types, reason = "fixture uses owned std strings")]
use crate::support::{corsa_path, corsa_requirement};
use std::{path::Path, process::Command};

const SHARED: &str = include_str!(
    "../../../tests/_fixtures/differential/typechecker/options-api-inherited-template-members/src/shared.ts.txt"
);
const BASE: &str = include_str!(
    "../../../tests/_fixtures/differential/typechecker/options-api-inherited-template-members/src/base.ts.txt"
);
const INVALID: &str = include_str!(
    "../../../tests/_fixtures/differential/typechecker/options-api-inherited-template-members/src/Invalid.vue.txt"
);
const WRAPPED: &str = include_str!(
    "../../../tests/_fixtures/differential/typechecker/options-api-inherited-template-members/src/Wrapped.vue.txt"
);
const CONFIG: &str = include_str!(
    "../../../tests/_fixtures/differential/typechecker/options-api-inherited-template-members/tsconfig.json.txt"
);
const SHADOW: &str = r#"<template>
  <p>{{ (payload as $AuthoredInstanceMember).valid.toUpperCase() }} {{ isPC ? 'pc' : 'sp' }}</p>
</template>
<script lang="ts">
import { defineComponent } from 'vue'
import { shared } from './shared'
type $AuthoredInstanceMember = { valid: string }
export default defineComponent({ mixins: [shared], data() { return { payload: { valid: 'ok' } } } })
</script>
"#;

#[derive(Clone, Copy, Debug)]
enum Case {
    OriginalText,
    Original,
    Repair,
    Restore,
    AuthoredType,
    UnicodeCrLf,
    Wrapped,
    ExplicitThis,
}

fn sources(case: Case) -> Vec<(&'static str, String)> {
    let mut invalid = INVALID.to_owned();
    if matches!(case, Case::Repair | Case::ExplicitThis) {
        invalid = invalid
            .replace("greet(1)", "greet('x')")
            .replace("isPC.toFixed()", "isPC ? 'pc' : 'sp'")
            .replace("currentDevice.toFixed()", "currentDevice.toUpperCase()")
            .replace("fromBase.toUpperCase()", "fromBase.toFixed()")
            .replace("{{ missing }}", "");
    } else if matches!(case, Case::AuthoredType) {
        invalid = SHADOW.to_owned();
    } else if matches!(case, Case::UnicodeCrLf) {
        invalid = invalid
            .replace("  <p", "<!--雪🌸-->  <p")
            .replace('\n', "\r\n");
    }
    let mut files = vec![
        (
            "src/shared.ts",
            if matches!(case, Case::ExplicitThis) {
                SHARED.replace(
                    "greet(name: string)",
                    "greet(this: { currentDevice: string }, name: string)",
                )
            } else {
                SHARED.to_owned()
            },
        ),
        ("src/base.ts", BASE.to_owned()),
        ("src/Invalid.vue", invalid),
        ("tsconfig.json", CONFIG.to_owned()),
    ];
    if matches!(case, Case::Wrapped) {
        files.push(("src/Wrapped.vue", WRAPPED.to_owned()));
    }
    files
}

fn original_errors(column_shift: usize, source_line: &str) -> Vec<String> {
    [
        (
            20,
            2345,
            "Argument of type 'number' is not assignable to parameter of type 'string'.",
        ),
        (
            32,
            2339,
            "Property 'toFixed' does not exist on type 'boolean'.",
        ),
        (
            62,
            2551,
            "Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'?",
        ),
        (
            87,
            2339,
            "Property 'toUpperCase' does not exist on type 'number'.",
        ),
        (
            107,
            2339,
            "Property 'missing' does not exist on type '<authored instance>'.",
        ),
    ]
    .into_iter()
    .map(|(column, code, message)| {
        // SourceContextCache renders this complete, <160-character authored
        // line. Only the original title expression is inside the start tag;
        // the Unicode comment ends before every diagnostic in that control.
        let binding = if column == 20 && column_shift == 0 {
            "; binding: 'title'"
        } else {
            ""
        };
        format!(
            "error:2:{} [TS{code}] {message} (source: {source_line}{binding})",
            column + column_shift
        )
    })
    .collect()
}

/// Only the existing native fixture's explicitly portable Vue receiver wording
/// is adapted. Every original packet/byte and the adapted packet are retained.
fn portable_packet(packet: &serde_json::Value) -> serde_json::Value {
    let mut value = packet.clone();
    if let Some(files) = value["files"].as_array_mut() {
        for file in files {
            if let Some(diagnostics) = file["diagnostics"].as_array_mut() {
                for diagnostic in diagnostics {
                    if let Some(text) = diagnostic.as_str() {
                        if let Some((prefix, receiver)) =
                            text.split_once("Property 'missing' does not exist on type '")
                        {
                            if receiver.starts_with("CreateComponentPublicInstance") {
                                if let Some((_, source)) = receiver.rsplit_once("'. (source: ") {
                                    *diagnostic = format!("{prefix}Property 'missing' does not exist on type '<authored instance>'. (source: {source}").into();
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    value
}

fn expected(case: Case) -> serde_json::Value {
    let repaired = matches!(case, Case::Repair | Case::AuthoredType | Case::ExplicitThis);
    let mut paths = vec!["src/Invalid.vue", "src/base.ts", "src/shared.ts"];
    if matches!(case, Case::Wrapped) {
        paths.push("src/Wrapped.vue");
    }
    paths.sort();
    let errors = if repaired {
        vec![]
    } else {
        let files = sources(case);
        let (_, invalid) = files
            .iter()
            .find(|(path, _)| *path == "src/Invalid.vue")
            .unwrap();
        let line = invalid.lines().nth(1).unwrap().trim();
        assert!(
            line.chars().count() < 160,
            "fixture must retain its whole source context"
        );
        original_errors(
            if matches!(case, Case::UnicodeCrLf) {
                10
            } else {
                0
            },
            line,
        )
    };
    let files = paths.iter().map(|name|serde_json::json!({"file":name,"diagnostics":
        if *name=="src/Invalid.vue" {errors.clone()} else if *name=="src/Wrapped.vue" {
            vec![format!("error:2:34 [TS2339] Property 'toFixed' does not exist on type 'boolean'. (source: {})", WRAPPED.lines().nth(1).unwrap().trim())]
        } else {vec![]}})).collect::<Vec<_>>();
    let config: serde_json::Value = serde_json::from_str(CONFIG).expect("physical config JSON");
    serde_json::json!({"files":files,"programs":[{"root":".","tsconfig":"tsconfig.json",
        "compilerOptions":config["compilerOptions"],"files":paths}],
        "errorCount":errors.len()+usize::from(matches!(case,Case::Wrapped)),"warningCount":0,"fileCount":paths.len()})
}

#[test]
fn original_inherited_options_cli_keeps_errors_repair_restore_and_authored_types() {
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let Some(native) = corsa_requirement::required_or_skip(corsa_path::resolve(workspace)) else {
        return;
    };
    // Check the edit before launching any case, so fixture drift cannot leave
    // earlier command observations unpersisted or silently remove this control.
    assert_eq!(SHARED.matches("greet(name: string)").count(), 1);
    assert_eq!(
        SHARED
            .replace(
                "greet(name: string)",
                "greet(this: { currentDevice: string }, name: string)"
            )
            .matches("greet(this: { currentDevice: string }, name: string)")
            .count(),
        1
    );
    let modules = workspace.join("tests/node_modules");
    let vue_path = modules
        .join("vue/package.json")
        .canonicalize()
        .expect("actual test Vue; no facade");
    let vue = std::fs::read(&vue_path).expect("whole physical Vue manifest");
    let root = tempfile::tempdir().unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(&modules, root.path().join("node_modules")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(&modules, root.path().join("node_modules")).unwrap();
    let mut observations = Vec::new();
    for case in [
        Case::OriginalText,
        Case::Original,
        Case::Repair,
        Case::Restore,
        Case::AuthoredType,
        Case::UnicodeCrLf,
        Case::Wrapped,
        Case::ExplicitThis,
    ] {
        let extra = root.path().join("src/Wrapped.vue");
        if extra.exists() {
            std::fs::remove_file(extra).unwrap();
        }
        let files = sources(case);
        for (name, body) in &files {
            let path = root.path().join(name);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, body).unwrap();
        }
        // The report's original command and a declared JSON companion, with
        // default Options API behavior. No quiet/no-config/feature override.
        let args = if matches!(case, Case::OriginalText) {
            vec!["check"]
        } else {
            vec!["check", "--format", "json"]
        };
        let mut record = serde_json::json!({"case":format!("{case:?}"),"cwd":root.path(),"args":args,
            "inputs":files,"expected":if matches!(case,Case::OriginalText){serde_json::Value::Null}else{expected(case)},
            "expectedStatus":if matches!(case,Case::Repair|Case::AuthoredType|Case::ExplicitThis){0}else{1},
            "expectedStderr":format!("Building Corsa virtual project for {} files under {}...\nRunning Corsa diagnostics for {} files...\n",
                files.len()-1, root.path().display(), files.len()-1),
            "status":null,"signal":null,"stdoutBytes":null,"stderrBytes":null,"parsed":null,
            "originalSelectedBackendPid":null});
        match Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(root.path())
            .env("CORSA_PATH", &native)
            .args(&args)
            .output()
        {
            Ok(output) => {
                record["status"] = output.status.code().into();
                record["osStatus"] = output.status.to_string().into();
                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    record["signal"] = output.status.signal().into();
                }
                let stdout = String::from_utf8(output.stdout.clone());
                let stderr = String::from_utf8(output.stderr.clone());
                record["stdoutBytes"] = output.stdout.into();
                record["stderrBytes"] = output.stderr.into();
                record["parsed"] = stdout
                    .as_ref()
                    .ok()
                    .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                    .into();
                record["portableAssertionPacket"] = portable_packet(&record["parsed"]);
                record["stdout"] = stdout.as_ref().ok().cloned().into();
                record["stderr"] = stderr.as_ref().ok().cloned().into();
                record["stdoutUtf8Error"] = stdout.err().map(|e| e.to_string()).into();
                record["stderrUtf8Error"] = stderr.err().map(|e| e.to_string()).into();
            }
            Err(error) => record["spawnError"] = error.to_string().into(),
        }
        observations.push(record);
    }
    let capture = serde_json::json!({"schema":"vize.inherited-options-cli-source-observations.v1",
        "scope":"Source CLI and real test Vue. Portable missing-instance wording is explicit; not public/macOS acceptance",
        "cli":env!("CARGO_BIN_EXE_vize"),"requestedNativePath":native,
        "vueManifestPath":vue_path,"vueManifestBytes":vue,"observations":observations});
    let whole = serde_json::to_string_pretty(&capture).unwrap();
    eprintln!("11339 complete CLI observations: {whole}");
    if let Some(out) = std::env::var_os("VIZE_SNAPSHOT_SOURCE_CAPTURE_DIR") {
        let dir = std::path::PathBuf::from(out).join("options-api-inherited-template-members");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("cli-observations.json"), &whole).unwrap();
    }
    // Every original/negative/repair/restored process is durable before verdict.
    for record in capture["observations"].as_array().unwrap() {
        assert_eq!(record["status"], record["expectedStatus"], "{record:#}");
        assert_eq!(record["signal"], serde_json::Value::Null, "{record:#}");
        assert_eq!(record["stderr"], record["expectedStderr"], "{record:#}");
        assert_eq!(
            record["stderrUtf8Error"],
            serde_json::Value::Null,
            "{record:#}"
        );
        assert_eq!(
            record["stdoutUtf8Error"],
            serde_json::Value::Null,
            "{record:#}"
        );
        if record["case"] != "OriginalText" {
            assert_eq!(
                record["portableAssertionPacket"], record["expected"],
                "{record:#}"
            );
        } else {
            // Full renderer output is retained; exact text diagnostic parity is
            // reviewed separately from the declared JSON packet assertions.
            assert_eq!(
                record["stdout"].as_str().map(str::is_empty),
                Some(false),
                "{record:#}"
            );
        }
    }
}
