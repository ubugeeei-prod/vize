use super::{CONTRACT, record};
use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};
use vize_l0::ToCompactString;

pub(super) fn collect(dir: &Path, provider: &Path, variant: &str, source: &str) {
    let project = tempfile::tempdir().expect("isolated SFC project");
    let root = project.path();
    let workspace = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    fs::write(root.join("Card.vue"), source).unwrap();
    fs::write(root.join("contracts.ts"), CONTRACT).unwrap();
    fs::write(root.join("package.json"), r#"{"type":"module"}"#).unwrap();
    let tsconfig = r#"{"compilerOptions":{"strict":true,"target":"ESNext","module":"ESNext","moduleResolution":"bundler","noEmit":true},"include":["*.ts","*.vue"]}"#;
    fs::write(root.join("tsconfig.json"), tsconfig).unwrap();
    fs::create_dir(root.join("node_modules")).unwrap();
    let vue = workspace
        .join("playground/node_modules/vue")
        .canonicalize()
        .unwrap();
    std::os::unix::fs::symlink(&vue, root.join("node_modules/vue")).unwrap();
    let config = crate::support::vite_config::source(
        serde_json::to_vec(&json!({
            "typeChecker":{"corsaPath":provider},
            "linter":{"preset":"incremental","typeAware":true,"rules":{
                "type/strict-boolean-expressions":"error","type/no-unsafe-template-binding":"error"
            }}
        }))
        .unwrap(),
    );
    fs::write(root.join("vite.config.mjs"), &config).unwrap();
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(root)
        .args(["lint", "--format", "json", "Card.vue"]);
    for name in vize_carton::corsa_resolver::CORSA_ENV_VARS {
        command.env_remove(name);
    }
    let output = command.output().expect("source CLI execution");
    let parsed = serde_json::from_slice::<Value>(&output.stdout);
    record(
        dir,
        &format!("cli-{variant}.json"),
        &json!({
            "source":{"Card.vue":source,"contracts.ts":CONTRACT,"tsconfig.json":tsconfig,"vite.config.mjs":config},
            "provider":provider,"vuePackage":serde_json::from_slice::<Value>(&fs::read(vue.join("package.json")).unwrap()).unwrap(),
            "status":output.status.code(),"statusDebug":format!("{:?}",output.status),
            "stdoutBytes":output.stdout,"stderrBytes":output.stderr,
        "report":parsed.as_ref().ok(),"parseError":parsed.as_ref().err().map(|error|error.to_compact_string()),
            "collectorOnly":true,
        }),
    );
    let report = parsed.expect("complete CLI JSON");
    let messages = report
        .as_array()
        .expect("report array")
        .iter()
        .flat_map(|file| file["messages"].as_array().expect("complete messages"))
        .collect::<Vec<_>>();
    let runtime = messages
        .iter()
        .filter(|message| message["ruleId"] == "type/corsa-runtime")
        .collect::<Vec<_>>();
    if runtime.is_empty() {
        let at = source.find("v-if=\"count\"").unwrap() + "v-if=\"".len();
        let line = source[..at].bytes().filter(|byte| *byte == b'\n').count() + 1;
        let column = source[..at]
            .rsplit('\n')
            .next()
            .unwrap()
            .encode_utf16()
            .count()
            + 1;
        assert_eq!(
            report,
            json!([{"file":"Card.vue","messages":[{
            "ruleId":"type/strict-boolean-expressions","ruleDocsPath":"docs/content/rules/type-and-script.md","severity":2,
            "message":"[vize:type/strict-boolean-expressions] Unexpected nullable number value in conditional. Please handle the nullish and zero cases explicitly.",
            "line":line,"column":column,"endLine":line,"endColumn":column+5
        }],"errorCount":1,"warningCount":0}])
        );
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stderr.is_empty());
    } else {
        assert_ne!(
            variant, "original",
            "original complete nullable control must remain green"
        );
        assert!(
            !output.status.success(),
            "query refusal must not exit successfully"
        );
        assert!(runtime.iter().all(|message| message["severity"] == 2));
        assert!(
            runtime.iter().any(|message| message["message"]
                .as_str()
                .unwrap()
                .contains("symbol owner is unknown to this connection")),
            "unexpected runtime failure: {report:#}"
        );
    }
    assert_eq!(fs::read_to_string(root.join("Card.vue")).unwrap(), source);
    assert_eq!(
        fs::read_to_string(root.join("contracts.ts")).unwrap(),
        CONTRACT
    );
    assert_eq!(
        fs::read_to_string(root.join("vite.config.mjs")).unwrap(),
        config
    );
}
