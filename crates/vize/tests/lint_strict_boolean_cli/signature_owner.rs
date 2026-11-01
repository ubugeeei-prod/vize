//! A real imported validator remains typed, while an unsafe validator still errs.
use super::{RULE, STRICT, project, read_native_config, vite_config};
use serde_json::{Value, json};
use std::{fs, process::Command};

macro_rules! fixture {
    ($name:literal) => {
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/differential/linter/symbol-owner-probes-11267/",
            $name
        ))
    };
}

const SOURCE: &str = fixture!("App.vue.txt");
const TYPED: &str = fixture!("Required.ts.txt");
const UNTYPED: &str = fixture!("Untyped.ts.txt");
const EMITS: &str = "type/require-typed-emits";

fn diagnostic(rule: &str, message: &str, start: usize, len: usize) -> Value {
    let prefix = &SOURCE[..start];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let column = prefix.rsplit('\n').next().unwrap().chars().count() + 1;
    json!({"ruleId":rule,"ruleDocsPath":"docs/content/rules/type-and-script.md",
        "severity":2,"message":message,"line":line,"column":column,
        "endLine":line,"endColumn":column+SOURCE[start..start+len].chars().count()})
}

fn expected(unsafe_validator: bool) -> Value {
    let mut messages = Vec::new();
    if unsafe_validator {
        let call = "defineEmits({ submit: check })";
        let mut emit = diagnostic(
            EMITS,
            "[vize:type/require-typed-emits] Emit should have a type definition",
            SOURCE.find(call).unwrap(),
            call.len(),
        );
        emit["help"] =
            json!("Use defineEmits<...>() or a validator object with typed payload parameters.");
        messages.push(emit);
    }
    // Match the existing complete strict-boolean contract, at its authored span.
    let boolean = diagnostic(
        RULE,
        "[vize:type/strict-boolean-expressions] Unexpected number value in conditional. A number can be falsy (0, NaN) or truthy.",
        SOURCE.find("if (count)").unwrap() + "if (".len(),
        "count".len(),
    );
    messages.push(boolean);
    json!([{"file":"App.vue","messages":messages,"errorCount":if unsafe_validator {2}else{1},"warningCount":0}])
}

#[test]
fn imported_validator_types_and_real_warnings_survive_repairs() {
    let Some(project) = project(SOURCE, serde_json::from_str(STRICT).unwrap()) else {
        return;
    };
    let root = project.path();
    let path = root.join("vite.config.mjs");
    let mut config = read_native_config(&path);
    config["linter"]["rules"][EMITS] = json!("error");
    fs::write(
        &path,
        vite_config::source(serde_json::to_vec(&config).unwrap()),
    )
    .unwrap();
    let config_source = fs::read_to_string(&path).unwrap();
    let mut observations = Vec::new();
    for (phase, dependency, unsafe_validator) in [
        ("typed", TYPED, false),
        ("untyped", UNTYPED, true),
        ("repaired", TYPED, false),
        ("restored", UNTYPED, true),
    ] {
        fs::write(root.join("dependency.ts"), dependency).unwrap();
        let args = ["lint", "--format", "json", "App.vue"];
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(root)
            .args(args)
            .output()
            .unwrap();
        let report = serde_json::from_slice::<Value>(&output.stdout);
        observations.push(json!({"phase":phase,"dependency":dependency,
            "args":args,"status":output.status.code(),"stdout":output.stdout,"stderr":output.stderr,
            "report":match report {Ok(value)=>value,Err(error)=>json!({"parseError":error.to_string()})},
            "expected":expected(unsafe_validator),
            "sourceAfter":fs::read_to_string(root.join("App.vue")).unwrap(),
            "configAfter":fs::read_to_string(&path).unwrap(),
            "dependencyAfter":fs::read_to_string(root.join("dependency.ts")).unwrap()}));
    }
    let capture = json!({"schema":"vize.patina.signature-owner-cli.v1",
        "scope":"authored causal reduction, not the original 1027-file app",
        "source":SOURCE,"config":config,"configSource":config_source,
        "executable":env!("CARGO_BIN_EXE_vize"),"cwd":root,"observations":observations});
    let raw = serde_json::to_string_pretty(&capture).unwrap() + "\n";
    eprintln!("11267 complete CLI observations: {raw}");
    if let Some(path) = std::env::var_os("VIZE_SYMBOL_OWNER_CLI_CAPTURE") {
        fs::write(path, raw).unwrap();
    }
    for row in observations {
        assert_eq!(row["status"], 1, "{row:#}");
        assert_eq!(row["stderr"], json!([]), "{row:#}");
        assert_eq!(row["report"], row["expected"], "{row:#}");
        assert_eq!(row["sourceAfter"], SOURCE, "{row:#}");
        assert_eq!(row["configAfter"], config_source, "{row:#}");
        assert_eq!(row["dependencyAfter"], row["dependency"], "{row:#}");
    }
}
